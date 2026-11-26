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
        let mut community_sums = std::collections::HashMap::<usize, (usize, f32, f32)>::new();
        for (i, node) in nodes.iter().enumerate() {
            let entry = community_sums.entry(node.community).or_default();
            entry.0 += 1;
            entry.1 += positions[i].0;
            entry.2 += positions[i].1;
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
                let scale = 0.000018 * weight / d2;
                forces[i].0 += dx * scale;
                forces[i].1 += dy * scale;
            }
        }
        for &(a, b) in &valid_edges {
            let dx = positions[b].0 - positions[a].0;
            let dy = positions[b].1 - positions[a].1;
            let distance = (dx * dx + dy * dy).sqrt().max(0.0001);
            let scale = 0.018 * (distance - 0.13) / distance;
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
            let (count, sx, sy) = community_sums[&node.community];
            if count > 1 {
                let cx = (sx - positions[i].0) / (count - 1) as f32;
                let cy = (sy - positions[i].1) / (count - 1) as f32;
                forces[i].0 += (cx - positions[i].0) * 0.006;
                forces[i].1 += (cy - positions[i].1) * 0.006;
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
    use super::{LayoutNode, LayoutSettings, force_layout};

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
}
