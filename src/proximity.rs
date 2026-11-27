//! Origin-centered contact groups derived from shortest-hop distance.
//!
//! Entities at the same hop distance are grouped into weakly connected
//! components of that layer. Cross-group summaries retain relationship-record
//! indices so analysts can return from an aggregate to the original records.

use std::collections::{HashMap, HashSet, VecDeque};

use uuid::Uuid;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Analysis {
    pub layers: Vec<Vec<Uuid>>,
    pub groups: Vec<ContactGroup>,
    pub interactions: Vec<GroupInteraction>,
    pub proxies: Vec<GroupProxy>,
    pub cross_group_relationship_record_count: usize,
    pub map_hops: Vec<usize>,
    pub map_groups_by_column: Vec<Vec<usize>>,
    pub map_group_indices: Vec<usize>,
    pub map_interaction_indices: Vec<usize>,
    pub reachable_entity_count: usize,
    pub unreachable_entity_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ContactGroup {
    pub hop_distance: usize,
    pub members: Vec<Uuid>,
    pub internal_record_count: usize,
    pub contains_cross_group_proxy: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct GroupInteraction {
    pub first_group: usize,
    pub second_group: usize,
    pub relationship_record_count: usize,
    pub distinct_endpoint_pair_count: usize,
    pub first_to_second_record_count: usize,
    pub second_to_first_record_count: usize,
    pub undirected_record_count: usize,
    pub link_indices: Vec<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct GroupProxy {
    pub entity_id: Uuid,
    pub distinct_group_contacts: usize,
    pub distinct_counterpart_entities: usize,
    pub relationship_record_count: usize,
}

fn sample_group_map(
    groups: &[ContactGroup],
    max_groups: usize,
    max_hops: usize,
) -> (Vec<usize>, Vec<Vec<usize>>) {
    if groups.is_empty() || max_groups == 0 || max_hops == 0 {
        return (Vec::new(), Vec::new());
    }

    let hop_count = groups.last().map_or(0, |group| group.hop_distance + 1);
    let displayed_hops: Vec<usize> = if hop_count <= max_hops {
        (0..hop_count).collect()
    } else if max_hops == 1 {
        vec![0]
    } else {
        (0..max_hops)
            .map(|column| column * (hop_count - 1) / (max_hops - 1))
            .collect()
    };
    let mut groups_by_hop = vec![Vec::new(); hop_count];
    for (group_index, group) in groups.iter().enumerate() {
        groups_by_hop[group.hop_distance].push(group_index);
    }

    let mut visible_groups_by_column = vec![Vec::new(); displayed_hops.len()];
    let mut group_cursors = vec![0; displayed_hops.len()];
    let mut visible_group_count = 0;
    loop {
        let mut found_group = false;
        for (column, hop) in displayed_hops.iter().copied().enumerate() {
            if visible_group_count >= max_groups {
                break;
            }
            if let Some(group_index) = groups_by_hop[hop].get(group_cursors[column]).copied() {
                visible_groups_by_column[column].push(group_index);
                group_cursors[column] += 1;
                visible_group_count += 1;
                found_group = true;
            }
        }
        if !found_group || visible_group_count >= max_groups {
            break;
        }
    }

    (displayed_hops, visible_groups_by_column)
}

/// Build a deterministic, undirected and unweighted contact roll-up from an
/// origin. Directions and parallel records do not alter hop distance or group
/// membership; cross-group summaries still preserve direction and every link
/// record. Unknown endpoints and self-links are excluded from the topology.
pub(super) fn analyze<'a, E, L>(entities: E, links: L, origin: Uuid) -> Analysis
where
    E: IntoIterator<Item = (Uuid, &'a str)>,
    L: IntoIterator<Item = (Uuid, Uuid, bool)>,
{
    let names: HashMap<Uuid, String> = entities
        .into_iter()
        .map(|(id, name)| (id, name.to_lowercase()))
        .collect();
    let entity_count = names.len();
    if !names.contains_key(&origin) {
        return Analysis {
            unreachable_entity_count: entity_count,
            ..Default::default()
        };
    }

    let mut adjacency: HashMap<Uuid, Vec<Uuid>> =
        names.keys().map(|id| (*id, Vec::new())).collect();
    let links: Vec<(Uuid, Uuid, bool)> = links.into_iter().collect();
    for (from, to, _) in &links {
        if from == to || !adjacency.contains_key(from) || !adjacency.contains_key(to) {
            continue;
        }
        adjacency.entry(*from).or_default().push(*to);
        adjacency.entry(*to).or_default().push(*from);
    }
    for neighbors in adjacency.values_mut() {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    let mut distances = HashMap::from([(origin, 0usize)]);
    let mut queue = VecDeque::from([origin]);
    let mut max_distance = 0;
    while let Some(current) = queue.pop_front() {
        let distance = distances[&current];
        for neighbor in adjacency.get(&current).into_iter().flatten() {
            if distances.contains_key(neighbor) {
                continue;
            }
            let next_distance = distance + 1;
            distances.insert(*neighbor, next_distance);
            max_distance = max_distance.max(next_distance);
            queue.push_back(*neighbor);
        }
    }

    let entity_order = |first: &Uuid, second: &Uuid| {
        names
            .get(first)
            .cmp(&names.get(second))
            .then_with(|| first.cmp(second))
    };
    let mut layers = vec![Vec::new(); max_distance];
    for (id, distance) in &distances {
        if *distance > 0 {
            layers[*distance - 1].push(*id);
        }
    }
    for layer in &mut layers {
        layer.sort_by(entity_order);
    }

    let mut ordered_by_distance = Vec::with_capacity(max_distance + 1);
    ordered_by_distance.push(vec![origin]);
    ordered_by_distance.extend(layers.iter().cloned());
    for members in &mut ordered_by_distance {
        members.sort_by(entity_order);
    }
    let mut groups = Vec::new();
    let mut group_by_entity = HashMap::new();
    for (distance, members) in ordered_by_distance.iter_mut().enumerate() {
        let allowed: HashSet<Uuid> = members.iter().copied().collect();
        let mut visited = HashSet::new();
        for seed in members.iter().copied() {
            if !visited.insert(seed) {
                continue;
            }
            let mut component = Vec::new();
            let mut queue = VecDeque::from([seed]);
            while let Some(current) = queue.pop_front() {
                component.push(current);
                for neighbor in adjacency.get(&current).into_iter().flatten() {
                    if allowed.contains(neighbor) && visited.insert(*neighbor) {
                        queue.push_back(*neighbor);
                    }
                }
            }
            component.sort_by(entity_order);
            let group_index = groups.len();
            for id in &component {
                group_by_entity.insert(*id, group_index);
            }
            groups.push(ContactGroup {
                hop_distance: distance,
                members: component,
                internal_record_count: 0,
                contains_cross_group_proxy: false,
            });
        }
    }

    let mut interactions: HashMap<(usize, usize), GroupInteraction> = HashMap::new();
    let mut endpoint_pairs: HashMap<(usize, usize), HashSet<(Uuid, Uuid)>> = HashMap::new();
    let mut proxy_groups: HashMap<Uuid, HashSet<usize>> = HashMap::new();
    let mut proxy_neighbors: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
    let mut proxy_records: HashMap<Uuid, usize> = HashMap::new();

    for (link_index, (from, to, directed)) in links.iter().copied().enumerate() {
        let (Some(&from_group), Some(&to_group)) =
            (group_by_entity.get(&from), group_by_entity.get(&to))
        else {
            continue;
        };
        if from_group == to_group {
            groups[from_group].internal_record_count += 1;
            continue;
        }

        let (first_group, second_group) = if from_group < to_group {
            (from_group, to_group)
        } else {
            (to_group, from_group)
        };
        let interaction = interactions
            .entry((first_group, second_group))
            .or_insert_with(|| GroupInteraction {
                first_group,
                second_group,
                ..Default::default()
            });
        interaction.relationship_record_count += 1;
        interaction.link_indices.push(link_index);
        if directed {
            if from_group == first_group {
                interaction.first_to_second_record_count += 1;
            } else {
                interaction.second_to_first_record_count += 1;
            }
        } else {
            interaction.undirected_record_count += 1;
        }

        let endpoint_pair = if from < to { (from, to) } else { (to, from) };
        endpoint_pairs
            .entry((first_group, second_group))
            .or_default()
            .insert(endpoint_pair);

        proxy_groups.entry(from).or_default().insert(to_group);
        proxy_groups.entry(to).or_default().insert(from_group);
        proxy_neighbors.entry(from).or_default().insert(to);
        proxy_neighbors.entry(to).or_default().insert(from);
        *proxy_records.entry(from).or_default() += 1;
        *proxy_records.entry(to).or_default() += 1;
    }

    let mut interactions: Vec<GroupInteraction> = interactions
        .into_iter()
        .map(|(key, mut interaction)| {
            interaction.distinct_endpoint_pair_count =
                endpoint_pairs.get(&key).map_or(0, HashSet::len);
            interaction.link_indices.sort_unstable();
            interaction
        })
        .collect();
    interactions.sort_by(|first, second| {
        groups[first.first_group]
            .hop_distance
            .cmp(&groups[second.first_group].hop_distance)
            .then_with(|| {
                groups[first.second_group]
                    .hop_distance
                    .cmp(&groups[second.second_group].hop_distance)
            })
            .then_with(|| first.first_group.cmp(&second.first_group))
            .then_with(|| first.second_group.cmp(&second.second_group))
    });

    let mut proxies: Vec<GroupProxy> = proxy_groups
        .into_iter()
        .filter_map(|(entity_id, contact_groups)| {
            (contact_groups.len() >= 2).then(|| GroupProxy {
                entity_id,
                distinct_group_contacts: contact_groups.len(),
                distinct_counterpart_entities: proxy_neighbors
                    .get(&entity_id)
                    .map_or(0, HashSet::len),
                relationship_record_count: proxy_records.get(&entity_id).copied().unwrap_or(0),
            })
        })
        .collect();
    proxies.sort_by(|first, second| {
        second
            .distinct_group_contacts
            .cmp(&first.distinct_group_contacts)
            .then_with(|| {
                second
                    .distinct_counterpart_entities
                    .cmp(&first.distinct_counterpart_entities)
            })
            .then_with(|| {
                second
                    .relationship_record_count
                    .cmp(&first.relationship_record_count)
            })
            .then_with(|| entity_order(&first.entity_id, &second.entity_id))
    });

    let proxy_entities: HashSet<Uuid> = proxies.iter().map(|proxy| proxy.entity_id).collect();
    for group in &mut groups {
        group.contains_cross_group_proxy = group
            .members
            .iter()
            .any(|member| proxy_entities.contains(member));
    }
    let (map_hops, map_groups_by_column) = sample_group_map(&groups, 120, 40);
    let map_group_indices: Vec<usize> = map_groups_by_column.iter().flatten().copied().collect();
    let map_group_set: HashSet<usize> = map_group_indices.iter().copied().collect();
    let map_interaction_indices = interactions
        .iter()
        .enumerate()
        .filter_map(|(index, interaction)| {
            (map_group_set.contains(&interaction.first_group)
                && map_group_set.contains(&interaction.second_group))
            .then_some(index)
        })
        .collect();
    let cross_group_relationship_record_count = interactions
        .iter()
        .map(|interaction| interaction.relationship_record_count)
        .sum();

    Analysis {
        layers,
        groups,
        interactions,
        proxies,
        cross_group_relationship_record_count,
        map_hops,
        map_groups_by_column,
        map_group_indices,
        map_interaction_indices,
        reachable_entity_count: distances.len(),
        unreachable_entity_count: entity_count.saturating_sub(distances.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::analyze;
    use super::{ContactGroup, sample_group_map};
    use uuid::Uuid;

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    #[test]
    fn origin_layers_condense_same_layer_contacts_and_rank_cross_group_proxies() {
        let entities = [
            (id(1), "Origin"),
            (id(2), "Peer A"),
            (id(3), "Peer B"),
            (id(4), "Outer A"),
            (id(5), "Outer B"),
            (id(6), "Isolate"),
        ];
        let links = [
            (id(1), id(2), true),
            (id(1), id(3), false),
            (id(2), id(3), true),
            (id(2), id(4), true),
            (id(2), id(4), false),
            (id(2), id(5), true),
            (id(3), id(5), false),
            (id(5), id(5), true),
            (id(5), id(999), true),
        ];

        let analysis = analyze(entities, links, id(1));
        assert_eq!(analysis.reachable_entity_count, 5);
        assert_eq!(analysis.unreachable_entity_count, 1);
        assert_eq!(
            analysis.layers,
            vec![vec![id(2), id(3)], vec![id(4), id(5)]]
        );
        assert_eq!(analysis.groups.len(), 4);
        assert_eq!(analysis.groups[0].members, vec![id(1)]);
        assert_eq!(analysis.groups[1].members, vec![id(2), id(3)]);
        assert_eq!(analysis.groups[1].internal_record_count, 1);
        assert_eq!(analysis.groups[2].members, vec![id(4)]);
        assert_eq!(analysis.groups[3].members, vec![id(5)]);

        let contact_pair = analysis
            .interactions
            .iter()
            .find(|interaction| interaction.first_group == 1 && interaction.second_group == 2)
            .expect("peer group contacts outer group");
        assert_eq!(contact_pair.relationship_record_count, 2);
        assert_eq!(contact_pair.distinct_endpoint_pair_count, 1);
        assert_eq!(contact_pair.first_to_second_record_count, 1);
        assert_eq!(contact_pair.undirected_record_count, 1);
        assert_eq!(contact_pair.link_indices, vec![3, 4]);

        let proxy = analysis
            .proxies
            .first()
            .expect("cross-group proxy is ranked");
        assert_eq!(proxy.entity_id, id(2));
        assert_eq!(proxy.distinct_group_contacts, 3);
        assert_eq!(proxy.distinct_counterpart_entities, 3);
        assert_eq!(proxy.relationship_record_count, 4);
        assert!(analysis.groups[1].contains_cross_group_proxy);
        assert_eq!(analysis.map_group_indices, vec![0, 1, 2, 3]);
        assert_eq!(
            analysis.map_interaction_indices.len(),
            analysis.interactions.len()
        );
        assert_eq!(
            analysis.cross_group_relationship_record_count,
            analysis
                .interactions
                .iter()
                .map(|interaction| interaction.relationship_record_count)
                .sum::<usize>()
        );
    }

    #[test]
    fn group_map_sampling_is_bounded_deterministic_and_covers_sampled_hops() {
        let groups_by_hop = [
            vec![0, 1, 2],
            vec![3],
            vec![4, 5],
            vec![6, 7],
            vec![8],
            vec![9, 10],
            vec![11],
            vec![12, 13],
        ];
        let groups: Vec<ContactGroup> = groups_by_hop
            .iter()
            .enumerate()
            .flat_map(|(hop, indices)| {
                indices.iter().map(move |index| ContactGroup {
                    hop_distance: hop,
                    members: vec![id(*index as u128 + 1)],
                    ..Default::default()
                })
            })
            .collect();

        let (displayed_hops, sampled) = sample_group_map(&groups, 5, 3);
        assert_eq!(displayed_hops, vec![0, 3, 7]);
        assert!(sampled.iter().all(|hop_groups| !hop_groups.is_empty()));
        assert_eq!(sampled.iter().map(Vec::len).sum::<usize>(), 5);
        for (column, hop) in displayed_hops.iter().copied().enumerate() {
            assert!(
                sampled[column]
                    .iter()
                    .all(|group_index| groups[*group_index].hop_distance == hop)
            );
        }
        assert_eq!(
            (displayed_hops.clone(), sampled.clone()),
            sample_group_map(&groups, 5, 3)
        );

        let (one_hop, one_column) = sample_group_map(&groups, 120, 1);
        assert_eq!(one_hop, vec![0]);
        assert_eq!(one_column, vec![vec![0, 1, 2]]);
        assert!(sample_group_map(&groups, 0, 3).1.is_empty());
        assert!(sample_group_map(&groups, 3, 0).0.is_empty());
    }

    #[test]
    fn missing_origin_produces_no_groups_or_proxies() {
        let analysis = analyze(
            [(id(1), "Only")],
            std::iter::empty::<(Uuid, Uuid, bool)>(),
            id(2),
        );
        assert!(analysis.groups.is_empty());
        assert!(analysis.interactions.is_empty());
        assert!(analysis.proxies.is_empty());
        assert_eq!(analysis.unreachable_entity_count, 1);
    }
}
