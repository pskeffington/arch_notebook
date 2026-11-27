# PROXIMITY-MAP-CLINTON-01 — bounded topology rendering on the retained Clinton case

- **Task / owner:** Clean up the hop-group topology overview and preserve full group/contact navigation; implementation session.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-baseline` / `ff106a669320c1e5e2325a39bd407da736a15ef5`.
- **Candidate source:** recorded after commit in the build acceptance record.
- **Candidate binary:** Linux debug, SHA-256 `a84eb32ae53a51f1d17bdab7e8d905d280318aa8105dd50fc8c91a6f1ffa7bb9`.

## Change

The hop-group summary now uses one compact row per layer instead of repeating every group in every layer. The visual topology sample is capped at 48 groups, balanced over the displayed hop layers, and prioritizes proxy-bearing and larger groups within each layer. Its map viewport remains scrollable. A separate virtualized full group index and full direct-contact rows retain every group and record-level drilldown. Per-group contact lookups are prepared in one pass; group selection exposes exact direct-contact counts and source records.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Full offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1`: 69 passed, 0 failed, 2 ignored (the opt-in benchmark and Clinton-local fixture). |
| Retained Clinton topology regression | PASS | With `ARCH_NOTEBOOK_CLINTON_CASE_PATH=/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json`, `cargo test --locked --offline clinton_case_topology_sampling_stays_bounded_and_covers_hops -- --ignored --nocapture` passed. Confirms 2,813 entities, 6,173 links, 2,128 reachable entities from the deterministic highest-degree origin, 10 displayed hop columns, 48 sampled groups, every hop represented, visible edges have visible endpoints, and the expanded index accounts for each group contact twice. |
| Headless production UI | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` verifies the compact overview and sampled topology are rendered, the complete selected-group direct contact row is selectable, and selection opens the exact-record detail. |
| Clinton corpus acceptance harness | PASS | `scripts/test-clinton-corpus-acceptance.py` on the candidate binary: query-only preflight 1 pass/0 fail; saved SQL replay 6,173 rows, 0 skipped, 0 deltas; redacted packet has 2,813/2,813 entities and no labels outside the SVG viewport. Read-only inputs remained unchanged. Aggregate report was kept at `/tmp/clinton-corpus-topology-final.json`. |
| Locked debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0. |
| Release build | NOT VERIFIED | Offline release compilation stopped progressing after entering the application crate; it was interrupted. No release result is claimed. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native Clinton topology visual review | NOT TESTED | The local display could not be opened from this session. Headless UI and corpus-data regressions do not establish native appearance, pointer feel, contrast, or render performance. |
| Qwen and human content review | NOT RUN | This is a structural topology/rendering check; it makes no semantic claims about the corpus. PDF extraction completeness is not established by this run. |
| Exact-head CI | PENDING | Dispatch and record the workflow for the committed candidate. |

The selection of a highest-degree origin is a deterministic structural fixture, not a semantic finding about an individual. No corpus names, entity IDs, relationship rows, or private input paths are added to this evidence.
