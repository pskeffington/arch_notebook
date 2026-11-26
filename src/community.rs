//! Deterministic structural community partitioning for chart exploration.
//!
//! This is a dependency-free, **single-level greedy modularity local-move**
//! heuristic. It is not Louvain or Leiden. It treats each distinct unordered
//! entity pair as one unweighted edge, ignores self-links and edges with an
//! unknown endpoint, uses modularity resolution `gamma = 1.0`, visits entities
//! in UUID order (so labels cannot affect membership), and performs at most 50
//! full sweeps.
//! Any disconnected fragments left with the same provisional label are split
//! into connected communities after optimization. Isolates remain singleton
//! communities. These are exploratory structural partitions, not ground-truth
//! social groups.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use uuid::Uuid;

const RESOLUTION: f64 = 1.0;
const MAX_SWEEPS: usize = 50;
const MOVE_EPSILON: f64 = 1e-12;

/// Partition the supplied entities using an undirected, unweighted simple
/// graph. Names are accepted for convenient callers but do not affect output.
/// Unknown edge endpoints, self-links, and repeated or reversed endpoint pairs
/// do not affect the partition.
pub(super) fn detected_communities<'a, E, L>(entities: E, edges: L) -> Vec<Vec<Uuid>>
where
    E: IntoIterator<Item = (Uuid, &'a str)>,
    L: IntoIterator<Item = (Uuid, Uuid)>,
{
    let mut ordered_entities: Vec<Uuid> = entities.into_iter().map(|(id, _)| id).collect();
    ordered_entities.sort_unstable();

    if ordered_entities.is_empty() {
        return Vec::new();
    }

    let id_to_index: HashMap<Uuid, usize> = ordered_entities
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index))
        .collect();
    let mut simple_edges = BTreeSet::new();
    for (from, to) in edges {
        let (Some(&from), Some(&to)) = (id_to_index.get(&from), id_to_index.get(&to)) else {
            continue;
        };
        if from == to {
            continue;
        }
        simple_edges.insert(if from < to { (from, to) } else { (to, from) });
    }

    let mut adjacency = vec![Vec::new(); ordered_entities.len()];
    for (from, to) in simple_edges {
        adjacency[from].push(to);
        adjacency[to].push(from);
    }
    // BTreeSet iteration already makes every neighbor list sorted.
    let degree: Vec<usize> = adjacency.iter().map(Vec::len).collect();
    let total_degree: usize = degree.iter().sum();

    let mut community: Vec<usize> = (0..ordered_entities.len()).collect();
    if total_degree > 0 {
        let mut community_degree: Vec<f64> = degree.iter().map(|value| *value as f64).collect();
        let twice_edges = total_degree as f64;

        for _ in 0..MAX_SWEEPS {
            let mut moved_any = false;
            for node in 0..ordered_entities.len() {
                let node_degree = degree[node] as f64;
                if node_degree == 0.0 {
                    continue;
                }

                let current = community[node];
                community_degree[current] -= node_degree;

                let mut links_by_community: HashMap<usize, usize> = HashMap::new();
                for neighbor in &adjacency[node] {
                    *links_by_community.entry(community[*neighbor]).or_default() += 1;
                }

                // After removing the node, the current community is the
                // incumbent. Candidate ties resolve to the smallest stable
                // community key (the traversal index of its original seed).
                let mut best = current;
                let mut best_score = modularity_move_score(
                    links_by_community.get(&current).copied().unwrap_or(0),
                    node_degree,
                    community_degree[current],
                    twice_edges,
                );
                let mut candidates: Vec<usize> = links_by_community.keys().copied().collect();
                candidates.sort_unstable();
                for candidate in candidates {
                    let score = modularity_move_score(
                        links_by_community[&candidate],
                        node_degree,
                        community_degree[candidate],
                        twice_edges,
                    );
                    if score > best_score + MOVE_EPSILON {
                        best = candidate;
                        best_score = score;
                    }
                }

                community[node] = best;
                community_degree[best] += node_degree;
                if best != current {
                    moved_any = true;
                }
            }
            if !moved_any {
                break;
            }
        }
    }

    let mut members_by_community: HashMap<usize, Vec<usize>> = HashMap::new();
    for (node, label) in community.iter().copied().enumerate() {
        members_by_community.entry(label).or_default().push(node);
    }

    // A local move can remove an articulation vertex and leave two fragments
    // with one label. Split such fragments so the result's connectivity claim
    // remains true.
    let mut groups = Vec::new();
    for members in members_by_community.values() {
        let allowed: HashSet<usize> = members.iter().copied().collect();
        let mut visited = HashSet::new();
        for seed in members {
            if !visited.insert(*seed) {
                continue;
            }
            let mut queue = VecDeque::from([*seed]);
            let mut group = Vec::new();
            while let Some(node) = queue.pop_front() {
                group.push(node);
                for neighbor in &adjacency[node] {
                    if allowed.contains(neighbor) && visited.insert(*neighbor) {
                        queue.push_back(*neighbor);
                    }
                }
            }
            group.sort_unstable();
            groups.push(group);
        }
    }

    groups.sort_by(|a, b| {
        b.len()
            .cmp(&a.len())
            .then_with(|| a.first().cmp(&b.first()))
    });
    groups
        .into_iter()
        .map(|group| {
            group
                .into_iter()
                .map(|index| ordered_entities[index])
                .collect()
        })
        .collect()
}

fn modularity_move_score(
    edges_into_candidate: usize,
    node_degree: f64,
    candidate_total_degree: f64,
    twice_edges: f64,
) -> f64 {
    edges_into_candidate as f64 - RESOLUTION * node_degree * candidate_total_degree / twice_edges
}

#[cfg(test)]
mod tests {
    use super::detected_communities;
    use std::collections::{HashMap, HashSet, VecDeque};
    use uuid::Uuid;

    fn id(index: u128) -> Uuid {
        Uuid::from_u128(index)
    }

    fn partition(nodes: &[(Uuid, &str)], edges: &[(Uuid, Uuid)]) -> Vec<Vec<Uuid>> {
        detected_communities(nodes.iter().copied(), edges.iter().copied())
    }

    fn assert_total_and_connected(
        nodes: &[(Uuid, &str)],
        edges: &[(Uuid, Uuid)],
        groups: &[Vec<Uuid>],
    ) {
        let known: HashSet<Uuid> = nodes.iter().map(|(id, _)| *id).collect();
        let flattened: Vec<Uuid> = groups.iter().flatten().copied().collect();
        assert_eq!(flattened.len(), nodes.len(), "every entity occurs once");
        assert_eq!(flattened.iter().copied().collect::<HashSet<_>>(), known);

        let mut adjacency: HashMap<Uuid, Vec<Uuid>> =
            known.iter().map(|entity| (*entity, Vec::new())).collect();
        for (from, to) in edges {
            if from != to && known.contains(from) && known.contains(to) {
                adjacency.entry(*from).or_default().push(*to);
                adjacency.entry(*to).or_default().push(*from);
            }
        }
        for group in groups {
            if group.len() < 2 {
                continue;
            }
            let allowed: HashSet<Uuid> = group.iter().copied().collect();
            let mut seen = HashSet::from([group[0]]);
            let mut queue = VecDeque::from([group[0]]);
            while let Some(entity) = queue.pop_front() {
                for neighbor in adjacency.get(&entity).into_iter().flatten() {
                    if allowed.contains(neighbor) && seen.insert(*neighbor) {
                        queue.push_back(*neighbor);
                    }
                }
            }
            assert_eq!(seen, allowed, "each community is connected");
        }
    }

    #[test]
    fn two_cliques_with_bridge_form_separate_connected_communities() {
        let nodes = [
            (id(1), "A1"),
            (id(2), "A2"),
            (id(3), "A3"),
            (id(4), "A4"),
            (id(5), "B1"),
            (id(6), "B2"),
            (id(7), "B3"),
            (id(8), "B4"),
            (id(9), "Isolate"),
        ];
        let mut edges = Vec::new();
        for start in [1, 5] {
            for left in start..start + 4 {
                for right in left + 1..start + 4 {
                    edges.push((id(left), id(right)));
                }
            }
        }
        edges.push((id(4), id(5)));

        let groups = partition(&nodes, &edges);
        assert_eq!(groups.iter().map(Vec::len).collect::<Vec<_>>(), [4, 4, 1]);
        assert!(
            groups
                .iter()
                .any(|group| group.contains(&id(1)) && group.contains(&id(4)))
        );
        assert!(
            groups
                .iter()
                .any(|group| group.contains(&id(5)) && group.contains(&id(8)))
        );
        assert_total_and_connected(&nodes, &edges, &groups);
    }

    #[test]
    fn disconnected_components_and_isolates_remain_separate() {
        let nodes = [
            (id(1), "A"),
            (id(2), "B"),
            (id(3), "C"),
            (id(4), "D"),
            (id(5), "Isolate"),
        ];
        let edges = [(id(1), id(2)), (id(3), id(4))];
        let groups = partition(&nodes, &edges);

        assert_eq!(groups.iter().map(Vec::len).collect::<Vec<_>>(), [2, 2, 1]);
        assert!(
            !groups
                .iter()
                .any(|group| group.contains(&id(1)) && group.contains(&id(3)))
        );
        assert!(
            !groups
                .iter()
                .any(|group| group.contains(&id(2)) && group.contains(&id(4)))
        );
        assert!(groups.iter().any(|group| group.as_slice() == [id(5)]));
        assert_total_and_connected(&nodes, &edges, &groups);
    }

    #[test]
    fn simple_graph_input_ignores_duplicate_reversed_self_and_unknown_edges() {
        let nodes = [(id(1), "A"), (id(2), "B"), (id(3), "C")];
        let simple_edges = [(id(1), id(2)), (id(2), id(3))];
        let noisy_edges = [
            (id(1), id(2)),
            (id(2), id(1)),
            (id(1), id(2)),
            (id(2), id(3)),
            (id(2), id(2)),
            (id(3), id(999)),
        ];
        assert_eq!(
            partition(&nodes, &simple_edges),
            partition(&nodes, &noisy_edges)
        );
    }

    #[test]
    fn cycles_and_equal_score_ties_are_deterministic_and_connected() {
        let nodes = [(id(4), "D"), (id(2), "B"), (id(1), "A"), (id(3), "C")];
        let edges = [
            (id(1), id(2)),
            (id(2), id(3)),
            (id(3), id(4)),
            (id(4), id(1)),
        ];
        let first = partition(&nodes, &edges);
        let mut reversed_input_nodes = nodes;
        reversed_input_nodes.reverse();
        let mut reversed_input_edges = edges;
        reversed_input_edges.reverse();
        assert_eq!(
            first,
            partition(&reversed_input_nodes, &reversed_input_edges)
        );
        let renamed_nodes = [
            (id(4), "Changed D"),
            (id(2), "Changed B"),
            (id(1), "Changed A"),
            (id(3), "Changed C"),
        ];
        assert_eq!(first, partition(&renamed_nodes, &edges));
        assert_total_and_connected(&nodes, &edges, &first);
    }
}
