# Selected topology route layering — 2026-11-27

**Task / owner:** Keep a selected group contact legible where map routes cross; implementation session.
**Repository / branch / baseline / product source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `76d4ac5ada34b610b0cfba1c3f9ff94a4ad26872` / `ef304472a3fc21b297b20c306a41281824228501`.

The sampled topology map previously painted the selected route together with each route's base stroke. A later crossing route could cover the selected highlight. The renderer now paints ordinary routes first, redraws the selected route above crossings, restores its directional arrows above the highlight, and paints group nodes last so route strokes remain masked at node centers. Route viewport culling and node-first pointer interaction are unchanged. Independent topology review caught the arrow-occlusion case and a potentially empty test comparison; both were corrected before this evidence update.

| Gate | Result | Evidence |
| --- | --- | --- |
| Selected route and direction marker layer order | PASS, headless UI | `headless_ui_drives_geodesic_and_detected_community_actions` requires all 12 selected-route segments and at least six reciprocal arrow strokes after the selected highlight in the emitted paint order. |
| Focused headless topology test | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline headless_ui_drives_geodesic_and_detected_community_actions -- --nocapture` exited 0: 1 passed. |
| Full offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 73 passed, 0 failed, 3 ignored (including two retained-corpus opt-in checks). |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Locked offline check and debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` and `cargo build --locked --offline` exited 0. |
| Independent topology code review | PASS | Reviewer confirmed selected highlight → directional arrows → nodes ordering and verified the regression rejects an empty highlight. |
| Locked offline release build | BLOCKED | The release build command did not return a completion status in this runner after reporting the application crate compilation step. It was interrupted; no release build pass or updated release binary is claimed. |
| Native visual review and exact-head CI | NOT RUN | Native display capture was unavailable in the earlier review; no new screenshot or exact-head workflow result is claimed for `541259f`. |

This change only affects the ordering of map paint operations. It does not establish native contrast/feel, renderer presentation performance, Clinton-corpus visual acceptance, or IBM Analyst's Notebook parity. Related interaction and Clinton sampling evidence: [PROXIMITY-MAP-PICK-01](PROXIMITY-MAP-PICK-01-linux.md), [PROXIMITY-MAP-CROSSING-REDUCTION-01](PROXIMITY-MAP-CROSSING-REDUCTION-01.md), and [PROXIMITY-MAP-CLINTON-UI-01](PROXIMITY-MAP-CLINTON-UI-01-linux.md).
