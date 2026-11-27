# SNG-DELIVERY-002 G1/G2 headless UI action evidence

- **Task/owner:** Exercise community and contact-analysis user actions through Arch Notebook's actual `App::update`; owner `/root`.
- **Source baseline:** `c362190c7fba5e2933a6f053577b26671c5510e3` on `codex/ui-00-chart-baseline`.
- **Scope:** `src/main.rs`, `src/proximity.rs`, this evidence note, and the phased roadmap. The code candidate is not yet committed at the time this evidence was recorded.

## Fixture and interaction

The test creates a deterministic fictional chart with two four-entity cliques, a bridge between the cliques, and 150 isolates. It uses stable UUIDs, real `egui::Context::run` frames, `eframe::App::update`, and synthetic pointer/wheel events at a 1440×900 logical viewport. It activates **Measure hop layers**, scrolls the Analysis pane to **Detect communities**, detects the partition, scrolls the virtualized list to the final singleton community and selects **Show**, opens a cross-community interaction, then inspects an exact contributing record. It seeds an unrelated fit selection before Inspect and verifies that the resulting fit set contains exactly the inspected record's two endpoints.

The pass exposed a layout defect: the Analysis sidebar had no outer scroll area, preventing lower controls from being reached at this viewport. The sidebar now scrolls as a whole while retaining the nested virtualized community list. Proximity analysis also stores bounded contact-map sampling indices and proxy flags once per analysis result rather than rebuilding those collections during every UI frame. Sampling remains deterministic and covered by unit tests; no timing improvement is claimed for that change.

## Verification

| Check | Command | Result |
| --- | --- | --- |
| Focused headless user-action path | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline headless_ui_drives_geodesic_and_detected_community_actions -- --nocapture` | **PASS**, exit 0; the real app update path completed geodesic, community, list scrolling, group selection, interaction selection, and exact-record inspection |
| Full Rust suite | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | **PASS**, exit 0; 52 passed, 0 failed |
| Formatting and whitespace | `cargo fmt --all`; `cargo fmt --all -- --check`; `git diff --check` | **PASS**, exit 0 |
| Offline type check | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | **PASS**, exit 0 |
| Offline debug build | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | **PASS**, exit 0 |
| Offline release build | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | **PASS**, exit 0; optimized build completed in 1m58s |
| Independent code review | `vector_patch_review` reviewed the diff against the source baseline | **PASS**; no correctness findings. Reviewer noted no UI test clicks a contact-map node and cap behavior is verified by the sampler unit test rather than an end-to-end large analysis assertion. |

## Limits and next gate

This is headless egui event-path evidence. It does not establish a native window, real pointer or keyboard behavior, visual quality, compositor performance, GPU/render time, or user acceptance. Native visual/input acceptance remains **BLOCKED** because this session cannot access an input-capable display. Earlier attempts exited with `XOpenDisplayFailed` under X11 and `NoCompositor` under Wayland. Do not use the older render-only screenshot as evidence for this candidate. No release or deployment approval is implied.

Next: independent review of this patch, prescribed offline type/check and debug/release builds, then native review when an input-capable desktop is available.

**Follow-up:** an input-capable X11/XWayland desktop later became available on 2026-11-26. Scoped native G1/G2 actions were then exercised with actual pointer and wheel input; see [native UI evidence](SNG-DELIVERY-002-G1-G2-native-ui-20261126.md). This historical headless report's blocked statement describes the earlier session state and is superseded for the tested G1/G2 path by that native evidence. Native G3/G4 gates remain open.

**Current-source contact-map click follow-up — 2026-11-26:** the regression `headless_ui_drives_geodesic_and_detected_community_actions` now scrolls the actual Analysis pane to the rendered **G2** map node, clicks its text through egui pointer events, and verifies that `chart_fit_entities` equals the complete member list for that exact group and that its first entity becomes selected. The focused test passed on source commit `f9fc22fa3b8e8ba8975505b58f5e77f3d25d91b8`; the full current-source suite is recorded as 57 passed in [latest branch acceptance](../build-acceptance.md). This closes the earlier reviewer-noted headless interaction gap only. Native pointer activation of a contact-map node remains unconfirmed, as stated in the native evidence.
