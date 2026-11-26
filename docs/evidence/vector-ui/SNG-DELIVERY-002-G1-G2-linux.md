# SNG-DELIVERY-002 G1/G2 implementation evidence

- **Task/owner:** Implement detected structural communities and cross-community record exploration; implementation backend by `/root/community_partition_backend`, UI/integration by `/root`.
- **Baseline:** `94c469316adc37403f5b88e396ea58171a24af05` on `codex/ui-00-chart-baseline`.
- **Changed paths:** `src/community.rs`, `src/main.rs`, `docs/notebook-workflows.md`, `docs/roadmaps/vector-community-exploration-roadmap.md`, and this evidence plus its packet acceptance JSON.
- **Scope:** G1 community membership and G2 cross-community count/drilldown in the Analysis pane. No source entity, relationship, SQL, or saved chart data are changed by analysis. Force layout and vector renderer optimization are not part of this implementation.

## Algorithm and semantics

The backend implements a dependency-free **single-level greedy modularity local-move heuristic**. It is not Louvain or Leiden. It uses resolution `γ=1`, at most 50 sweeps, and UUID traversal order for deterministic output. Input is a simple undirected unweighted graph: each unique unordered known-entity pair contributes one edge; reverse and duplicate records collapse; direction, date, type, and confidence do not affect membership; self-links and unknown endpoints are ignored; isolates remain singleton. Any disconnected provisional community is split after local moves so every returned non-singleton community is connected.

The Analysis pane computes on demand across all active-chart entities and relationship records, regardless of display selection or timeline filter. It labels the output as an exploratory structural partition and states it does not establish real-world group identity. All communities, cross-community pairs, and contributing records are accessible in virtualized scroll lists. Cross-group summaries separate relationship-record counts, distinct unordered endpoint pairs, forward/reverse directed record counts, and undirected records. Selecting an interaction exposes its exact chart records; selecting a record focuses it on the chart.

## Verification

| Check | Command | Result |
| --- | --- | --- |
| Formatting and whitespace | `cargo fmt --all -- --check`; `git diff --check` | **PASS**, exit 0 |
| Full application tests | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, exit 0; 41 passed, 0 failed |
| Offline type/check gate | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | **PASS**, exit 0 |
| Debug build | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | **PASS**, exit 0 |
| Release build | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | **PASS**, exit 0 |
| Packet regression harness | `python3 scripts/test-analysis-packet-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-packet-acceptance-20261126-g1g2-final2` | **PASS**, exit 0 |
| Independent static review | Review of current diff by `method_card_independent_review` | **PASS** after fixes for cache invalidation, complete list access, and method disclosure; reviewer did not rerun tests |

Community tests cover two cliques with a bridge and isolate, disconnected components and isolates, graph normalization for duplicates/reversed/self/unknown edges, cycles and deterministic ties, total membership, connected outputs, and rename-invariant partitioning. The integration test `detected_community_interactions_separate_record_pair_and_direction_counts` checks two four-node communities, five cross-community records, three distinct pairs, counts by direction, and the exact five source record indices. The method-card/cache test also checks that community analysis is cleared when analysis caches are invalidated. Relationship date/direction edits explicitly clear the cached analysis; names do not affect traversal because the algorithm orders by UUID.

## Durable artifacts

- Release binary used for packet regression: `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `361d849b3fada9797bbaa47dad2a785199492cdf5498cd4c39be019df7e82cab`.
- Packet acceptance report: [saved JSON](artifacts/SNG-DELIVERY-002-packet-acceptance.json), SHA-256 `bf929d6893cbb49315689bc69315c91a7003f91cbecda5641ed5cc35c0512588`.
- Fixture: `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.

## Limits and next gate

- Native visual and input acceptance is **BLOCKED**: this session could not access an input-capable desktop. Code/build/test evidence does not establish the actual Analysis pane layout, scrolling, selection, or chart focus behavior at runtime.
- The heuristic is single-level, has no user resolution control or multilevel aggregation, and inherits modularity's resolution limitations. Evaluate Leiden before describing or upgrading the algorithm further; do not read partition geometry as a social fact.
- Native graph performance baselines, force-directed layout optimization, and vector renderer optimization are **NOT STARTED**. G0 native measurements remain blocked by desktop access; G3 and G4 remain future gates in the roadmap.
- No release or deployment approval is implied.
