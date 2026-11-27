# PROXIMITY-MAP-EDGE-ROUTING-01 — separate sampled topology ties

- **Task / owner:** Improve visual separation and pick accuracy for ties in the sampled hop-group topology; implementation session, with an independent topology audit.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-baseline` / `96c7ad4f758458958fa429d9cfec120bc2ffc305`.
- **Candidate source:** `58efaac921512a68f1565b537dd70c438a8c5db5`.
- **Benchmark harness / aggregate evidence commit:** `88f2517c7743348542b568d63ae80f5ce63bcba6`.
- **Evidence and workflow source:** `6da8b2e9962ac078fdaa4832ff7e3b1442c971b1`.
- **Scope:** `src/main.rs`, this report, and topology acceptance notes.

## Change

Each sampled group interaction now uses a deterministic, twelve-segment quadratic route with a stable lane offset. Reciprocal arrowheads use opposite small offsets so both directions remain legible. The nodes paint over route ends, visually clipping ties at node boundaries. Direction arrowheads follow the route tangent and are placed between the node boundaries. The selected tie highlight follows every route segment, and pointer hit testing measures distance to those same segments. Fixed-size route arrays are prepared once per frame and reused by drawing and hit testing. This replaces a lane counter that never separated aggregated endpoint pairs.

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| Route geometry | PASS | Synthetic regression confirms deterministic routes, distinct lanes, finite nonzero segments, exact hit distance against the rendered route, and separated reciprocal arrow offsets. |
| Production UI selection and edge picking | PASS | Headless production UI test confirms all 12 selected-route segments receive the selection stroke, exact-record detail remains available, and clicking a visible rendered route selects its pair. |
| Clinton real-corpus topology | PASS | Retained UI test renders the sampled map at 2,813 entities / 6,173 records and verifies omitted-group/pair details; the sampler test confirms 48 groups across 10 hops and 23→3 endpoint-segment crossings. |
| Independent topology spot-check | PASS | Reviewer confirmed reciprocal offsets, local-tangent arrows, and shared draw/pick geometry; no critical routing issue found. |
| Full offline Rust suite | PASS | 71 passed, 0 failed, 3 ignored. |
| Locked check/debug build and formatting | PASS | `cargo check --locked --offline`, `cargo build --locked --offline`, `cargo fmt --all -- --check`, and `git diff --check` exited 0. |
| Exact-head cross-platform CI | PASS | [Run 46082650685](https://github.com/pskeffington/arch_notebook/actions/runs/46082650685) completed successfully for `58efaac921512a68f1565b537dd70c438a8c5db5`; all six Linux, Arch, and macOS jobs passed. |
| Benchmark-harness exact-head CI | PASS | [Run 46084425580](https://github.com/pskeffington/arch_notebook/actions/runs/46084425580) completed successfully for `6da8b2e9962ac078fdaa4832ff7e3b1442c971b1`; all six Linux, Arch, and macOS jobs passed. |
| Clinton sampled-map CPU frame cost | MEASURED | Paired headless production frame A/B at 1440×1600, 48 sampled groups / 47 pairs / 1,315 records: median frame-build 216.5 ms without the topology panel and 218.1 ms with it (+0.7%); median egui tessellation 147.8 ms / 149.0 ms (+0.8%). The panel adds 937 clipped shapes. Build p95 was 226.6 ms / 272.1 ms, so tail latency is variable; these are machine-specific headless CPU timings, not an FPS claim. Full results are in [the raw aggregate](artifacts/PROXIMITY-MAP-EDGE-ROUTING-01-clinton-frame-ab.json). |
| Native visual/pointer review and dense complete-map performance | NOT TESTED | Headless checks do not prove native contrast, feel, or presentation. The 1,128-pair complete-graph ceiling and GPU/compositor/FPS remain unmeasured. |

No corpus-specific names, IDs, or relationship contents are included in repository evidence.
