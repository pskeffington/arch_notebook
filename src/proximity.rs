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
    pub reachable_entity_count: usize,
    pub unreachable_entity_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ContactGroup {
    pub hop_distance: usize,
    pub members: Vec<Uuid>,
    pub internal_record_count: usize,
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

    Analysis {
        layers,
        groups,
        interactions,
        proxies,
        reachable_entity_count: distances.len(),
        unreachable_entity_count: entity_count.saturating_sub(distances.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::analyze;
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
