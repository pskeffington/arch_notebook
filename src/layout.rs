//! Deterministic, community-aware chart layout.
//!
//! This is a display arrangement only. It uses a fixed occupancy grid to
//! approximate long-range repulsion, keeping each iteration bounded by the
//! number of entities times 12² rather than comparing every entity pair.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LayoutNode {
    pub x: f32,
    pub y: f32,
    pub pinned: bool,
    pub community: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LayoutSettings {
    pub iterations: usize,
    pub seed: u64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ConnectivityNode {
    pub id: uuid::Uuid,
    pub x: f32,
    pub y: f32,
    pub pinned: bool,
    pub degree: usize,
}

/// G0 connectivity arrangement retained as the production baseline layout.
pub(super) fn connectivity_layout(nodes: &[ConnectivityNode]) -> Vec<(f32, f32)> {
    let mut positions: Vec<_> = nodes.iter().map(|node| (node.x, node.y)).collect();
    let movable_count = nodes.iter().filter(|node| !node.pinned).count();
    let mut order: Vec<usize> = (0..nodes.len())
        .filter(|index| !nodes[*index].pinned)
        .collect();
    order.sort_by_key(|index| (std::cmp::Reverse(nodes[*index].degree), nodes[*index].id));
    for (rank, index) in order.into_iter().enumerate() {
        let angle = std::f32::consts::TAU * rank as f32 / movable_count as f32
            - std::f32::consts::FRAC_PI_2;
        positions[index] = (0.5 + angle.cos() * 0.36, 0.5 + angle.sin() * 0.36);
    }
    positions
}

const GRID: usize = 12;
const MIN_ITERATIONS: usize = 1;
const MAX_ITERATIONS: usize = 120;

pub(super) fn force_layout(
    nodes: &[LayoutNode],
    edges: &[(usize, usize)],
    settings: LayoutSettings,
) -> Vec<(f32, f32)> {
    let mut positions: Vec<(f32, f32)> = nodes.iter().map(|n| (n.x, n.y)).collect();
    if nodes.len() < 2 {
        return positions;
    }
    let iterations = settings.iterations.clamp(MIN_ITERATIONS, MAX_ITERATIONS);
    let valid_edges: Vec<(usize, usize)> = edges
        .iter()
        .copied()
        .filter(|(a, b)| *a < nodes.len() && *b < nodes.len() && a != b)
        .collect();
    let mut velocity = vec![(0.0f32, 0.0f32); nodes.len()];
    // Seed-derived perturbation breaks exact overlaps reproducibly.
    for (i, node) in nodes.iter().enumerate() {
        if !node.pinned {
            let h = mix64(settings.seed ^ (i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            let angle = (h as u32) as f32 / u32::MAX as f32 * std::f32::consts::TAU;
            positions[i].0 += angle.cos() * 0.0005;
            positions[i].1 += angle.sin() * 0.0005;
        }
    }

    for iteration in 0..iterations {
        let mut counts = vec![0usize; GRID * GRID];
        let mut sums = vec![(0.0f32, 0.0f32); GRID * GRID];
        for (x, y) in positions.iter().copied() {
            let cell = cell_index(x, y);
            counts[cell] += 1;
            sums[cell].0 += x;
            sums[cell].1 += y;
        }
        let mut community_sums = std::collections::BTreeMap::<usize, (usize, f32, f32)>::new();
        for (i, node) in nodes.iter().enumerate() {
            let entry = community_sums.entry(node.community).or_default();
            entry.0 += 1;
            entry.1 += positions[i].0;
            entry.2 += positions[i].1;
        }
        let mut community_counts = vec![0usize; GRID * GRID];
        let mut community_centroids = vec![(0.0f32, 0.0f32); GRID * GRID];
        for (count, sum_x, sum_y) in community_sums.values() {
            let center = (*sum_x / *count as f32, *sum_y / *count as f32);
            let cell = cell_index(center.0, center.1);
            community_counts[cell] += 1;
            community_centroids[cell].0 += center.0;
            community_centroids[cell].1 += center.1;
        }
        let cooling = 1.0 - iteration as f32 / iterations as f32;
        let mut forces = vec![(0.0f32, 0.0f32); nodes.len()];

        // Grid-centroid repulsion approximates distant node groups and handles
        // dense cells without quadratic all-pairs work.
        for i in 0..nodes.len() {
            if nodes[i].pinned {
                continue;
            }
            let (x, y) = positions[i];
            let own = cell_index(x, y);
            for cell in 0..counts.len() {
                let count = counts[cell];
                if count == 0 {
                    continue;
                }
                let (cx, cy, weight) = if cell == own {
                    if count <= 1 {
                        continue;
                    }
                    let others = (count - 1) as f32;
                    (
                        (sums[cell].0 - x) / others,
                        (sums[cell].1 - y) / others,
                        others,
                    )
                } else {
                    (
                        sums[cell].0 / count as f32,
                        sums[cell].1 / count as f32,
                        count as f32,
                    )
                };
                let dx = x - cx;
                let dy = y - cy;
                let d2 = dx * dx + dy * dy + 0.0004;
                let scale = 0.00018 * weight / d2;
                forces[i].0 += dx * scale;
                forces[i].1 += dy * scale;
            }
            // Translate each community as a unit away from other community
            // centroids. Unlike centroid attraction, this cannot collapse a
            // community's members onto one point.
            let (group_count, group_x, group_y) = community_sums[&nodes[i].community];
            let group_center = (group_x / group_count as f32, group_y / group_count as f32);
            let group_cell = cell_index(group_center.0, group_center.1);
            for cell in 0..community_counts.len() {
                let count = community_counts[cell];
                if count == 0 {
                    continue;
                }
                let (cx, cy, weight) = if cell == group_cell {
                    if count <= 1 {
                        continue;
                    }
                    let others = (count - 1) as f32;
                    (
                        (community_centroids[cell].0 - group_center.0) / others,
                        (community_centroids[cell].1 - group_center.1) / others,
                        others,
                    )
                } else {
                    (
                        community_centroids[cell].0 / count as f32,
                        community_centroids[cell].1 / count as f32,
                        count as f32,
                    )
                };
                let dx = group_center.0 - cx;
                let dy = group_center.1 - cy;
                let d2 = dx * dx + dy * dy + 0.0025;
                let scale = 0.00008 * weight / d2;
                forces[i].0 += dx * scale;
                forces[i].1 += dy * scale;
            }
        }
        for &(a, b) in &valid_edges {
            let dx = positions[b].0 - positions[a].0;
            let dy = positions[b].1 - positions[a].1;
            let distance = (dx * dx + dy * dy).sqrt().max(0.0001);
            let attraction = if nodes[a].community == nodes[b].community {
                0.018
            } else {
                0.0045
            };
            let scale = attraction * (distance - 0.13) / distance;
            if !nodes[a].pinned {
                forces[a].0 += dx * scale;
                forces[a].1 += dy * scale;
            }
            if !nodes[b].pinned {
                forces[b].0 -= dx * scale;
                forces[b].1 -= dy * scale;
            }
        }
        for (i, node) in nodes.iter().enumerate() {
            if node.pinned {
                continue;
            }
            velocity[i].0 = (velocity[i].0 * 0.72 + forces[i].0).clamp(-0.035, 0.035);
            velocity[i].1 = (velocity[i].1 * 0.72 + forces[i].1).clamp(-0.035, 0.035);
            let step = 0.35 + 0.65 * cooling;
            positions[i].0 = (positions[i].0 + velocity[i].0 * step).clamp(0.02, 0.98);
            positions[i].1 = (positions[i].1 + velocity[i].1 * step).clamp(0.02, 0.98);
        }
    }
    for (i, node) in nodes.iter().enumerate() {
        if node.pinned {
            positions[i] = (node.x, node.y);
        }
    }
    positions
}

fn cell_index(x: f32, y: f32) -> usize {
    let col = (x.clamp(0.0, 0.999_999) * GRID as f32) as usize;
    let row = (y.clamp(0.0, 0.999_999) * GRID as f32) as usize;
    row * GRID + col
}

fn mix64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::{ConnectivityNode, LayoutNode, LayoutSettings, connectivity_layout, force_layout};

    #[test]
    fn connectivity_layout_orders_by_degree_and_preserves_pins() {
        let pinned = uuid::Uuid::from_u128(1);
        let low = uuid::Uuid::from_u128(2);
        let high = uuid::Uuid::from_u128(3);
        let nodes = [
            ConnectivityNode {
                id: pinned,
                x: 0.17,
                y: 0.83,
                pinned: true,
                degree: 99,
            },
            ConnectivityNode {
                id: low,
                x: 0.0,
                y: 0.0,
                pinned: false,
                degree: 1,
            },
            ConnectivityNode {
                id: high,
                x: 0.0,
                y: 0.0,
                pinned: false,
                degree: 4,
            },
        ];
        let positions = connectivity_layout(&nodes);
        assert_eq!(positions[0], (0.17, 0.83));
        assert!(positions[2].1 < positions[1].1);
    }

    #[test]
    fn force_layout_is_repeatable_bounded_and_preserves_pins() {
        let nodes = [
            LayoutNode {
                x: 0.25,
                y: 0.5,
                pinned: false,
                community: 0,
            },
            LayoutNode {
                x: 0.5,
                y: 0.5,
                pinned: true,
                community: 0,
            },
            LayoutNode {
                x: 0.75,
                y: 0.5,
                pinned: false,
                community: 1,
            },
            LayoutNode {
                x: 0.5,
                y: 0.5,
                pinned: false,
                community: 1,
            },
        ];
        let edges = [(0, 1), (1, 2), (2, 3), (50, 0)];
        let settings = LayoutSettings {
            iterations: 40,
            seed: 7,
        };
        let a = force_layout(&nodes, &edges, settings);
        let b = force_layout(&nodes, &edges, settings);
        assert_eq!(a, b);
        assert_eq!(a[1], (nodes[1].x, nodes[1].y));
        assert!(a.iter().all(|(x, y)| x.is_finite()
            && y.is_finite()
            && (0.02..=0.98).contains(x)
            && (0.02..=0.98).contains(y)));
    }

    #[test]
    fn force_layout_keeps_empty_and_singleton_inputs_well_formed() {
        assert!(
            force_layout(
                &[],
                &[],
                LayoutSettings {
                    iterations: 20,
                    seed: 1
                }
            )
            .is_empty()
        );
        let node = LayoutNode {
            x: 0.3,
            y: 0.7,
            pinned: false,
            community: 0,
        };
        assert_eq!(
            force_layout(
                &[node],
                &[],
                LayoutSettings {
                    iterations: 0,
                    seed: 1
                }
            ),
            vec![(0.3, 0.7)]
        );
    }

    #[test]
    fn force_layout_keeps_a_single_community_spread_across_the_canvas() {
        let nodes: Vec<LayoutNode> = (0..30)
            .map(|index| LayoutNode {
                x: 0.12 + (index % 6) as f32 * 0.15,
                y: 0.12 + (index / 6) as f32 * 0.15,
                pinned: false,
                community: 0,
            })
            .collect();
        let edges: Vec<(usize, usize)> = (0..30)
            .flat_map(|index| [(index, (index + 1) % 30), (index, (index + 6) % 30)])
            .collect();
        let positions = force_layout(
            &nodes,
            &edges,
            LayoutSettings {
                iterations: 50,
                seed: 13,
            },
        );
        let min_x = positions
            .iter()
            .map(|(x, _)| *x)
            .fold(f32::INFINITY, f32::min);
        let max_x = positions
            .iter()
            .map(|(x, _)| *x)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = positions
            .iter()
            .map(|(_, y)| *y)
            .fold(f32::INFINITY, f32::min);
        let max_y = positions
            .iter()
            .map(|(_, y)| *y)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(max_x - min_x > 0.5, "single group collapsed horizontally");
        assert!(max_y - min_y > 0.5, "single group collapsed vertically");
    }
}
