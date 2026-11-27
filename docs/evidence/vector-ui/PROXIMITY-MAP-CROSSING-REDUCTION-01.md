# PROXIMITY-MAP-CROSSING-REDUCTION-01 — order sampled groups by contact topology

- **Task / owner:** Reduce avoidable crossings in the sampled hop-group map; implementation session.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-baseline` / `25ff6b63e03b27fa98a31de0a00345f87c708847`.
- **Candidate source:** recorded after commit.
- **Scope:** `src/proximity.rs`, this report, and the topology roadmap.

## Method

The map keeps the existing proxy-prioritized, larger-group-first sample. Within the bounded sample, four deterministic forward/backward barycenter sweeps reorder each hop column according to neighboring group rows. This is a display-only ordering heuristic; it does not change group membership, shortest-hop distances, relationship direction, record counts, or persisted chart data.

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| Crossing reduction fixture | PASS | A four-by-four reversed bipartite fixture has 6 proper segment crossings before ordering and 0 afterward; repeated ordering yields the same result. |
| Production topology UI actions | PASS | Headless test for the Analysis pane passes group selection, line selection, exact-record navigation, measured node geometry, and sample-coverage labels. |
| Clinton topology scale and crossing count | PASS | Retained-corpus UI test and sampling test pass: 2,813 entities, 6,173 records, 10 displayed hops, 48 sampled groups. The sampled topology has 23 proper crossings before row ordering and 3 after. |
| Clinton primary corpus acceptance | PASS | `scripts/test-clinton-corpus-acceptance.py` passed with the current debug binary. SQL query-only preflight passed 1/1; saved run replayed 6,173 rows with zero skipped rows and zero deltas; the default-redacted packet rendered all 2,813 entity centers inside the SVG viewport. Read-only input hashes remained unchanged. Aggregate report is kept outside Git at `/tmp/clinton-topology-row-ordering-acceptance.json`. |
| Full Rust suite | PASS | 70 passed, 0 failed, 3 ignored. |
| Locked offline check/debug build and format | PASS | `cargo check`, `cargo build`, `cargo fmt --all -- --check`, and `git diff --check` pass. |
| Native visual review | BLOCKED | No native display access; reduced synthetic crossing count is not a substitute for visual acceptance. |
| Cross-platform CI | PENDING | Exact-head run must target the final committed candidate. |

The row-ordering heuristic improves a stated geometric metric on the fixture; it does not claim globally minimal crossings for arbitrary graphs.
