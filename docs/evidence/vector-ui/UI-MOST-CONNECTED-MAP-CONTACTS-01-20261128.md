# UI-MOST-CONNECTED-MAP-CONTACTS-01 — map contacts from a ranked entity

- **Task / owner:** Add a direct route from the Most Connected ranking to nearest-hop grouped contact topology; Rust implementation and verification.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `c249cfc7e1203e93326cd34aeb847fa4b7a6e1e0`.
- **Changed path:** `src/main.rs`.
- **Scope:** Each ranked entity now has a named **Map contacts · <entity>** action. It computes origin-centered undirected shortest-hop layers, grouped contact components, cross-group contact counts and proxy summaries, resets stale group selection, focuses the selected origin, and displays the existing topology view. The existing **Measure hop layers** action uses the same helper.

## Results

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Ranked action headless interaction | **PASS** | `headless_ui_drives_geodesic_and_detected_community_actions` clicks the entity-specific action, checks the origin/group state and visible `GROUP CONTACT TOPOLOGY` panel, then verifies the existing geodesic flow. |
| Rust formatting and diff check | **PASS** | `cargo fmt --all -- --check` and `git diff --check`, exit 0. |
| Locked offline check and workspace tests | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline` exit 0; test suite: 75 passed, 0 failed, 3 ignored. |
| Locked offline debug build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --workspace --locked --offline`, exit 0. Binary SHA-256: `25d6a9bfd421a9b0babde8eed1bcc10c6c83c0340b46bf115b1ba5050bf22505`. |
| Locked offline release build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --bin arch-notebook --locked --offline` completed successfully in 1m54s after a longer build wait. Release SHA-256: `f5ad767d85b6c4d4384a199f689cb896b2b414105c5bf8a8a2f1b2264bb39dd1`. The same binary passed isolated installation and release-binary Clinton acceptance; see [LOCAL-INSTALL-11](../deployment/LOCAL-INSTALL-11-linux.md). |
| Clinton SQL-backed corpus acceptance | **PASS, bounded** | Current debug binary acceptance exit 0: 6,173 relationship rows, zero skipped rows or deltas; 2,813 entities; two saved batches and runs pass; source inputs unchanged. Raw report: `/tmp/clinton-corpus-qa-map-contacts-20261128.json`. This validates the SQL/corpus regression path; it is not topology UI acceptance on the Clinton graph. |
| Native pointer/visual review | **NOT RUN** | Headless widget acceptance does not establish native desktop visual or pointer acceptance. |
| Qwen and human content review | **NOT RUN** | No model or content review was part of this topology change. |

## Follow-up

Perform native pointer/visual review on the installed Clinton graph. This action uses the existing topology calculation; it does not establish IBM Analyst's Notebook parity or PDF ingestion support.
