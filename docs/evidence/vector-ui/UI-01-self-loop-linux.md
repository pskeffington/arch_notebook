# UI-01 self-loop route and label repair

- **Task / owner:** Replace retraced self-link geometry with visible, selectable loops and keep loop labels off the route; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline / candidate:** `0d0fe0c4dd1347583d111362dd5d4771405ad235` → `e8f789fe9852f8130d45addb1ba9f4664002ee7e`.
- **Date / host:** 2026-11-26, Linux x86_64; `rustc 1.100.0`, `cargo 1.100.0`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`; SHA-256 `80c0f4af66b31d9814ac2931e8d22dba489bcb028d8f7f7998474bc1c4230e2c`.
- **Fixture:** fictional correctness chart copied to `/tmp/arch-notebook-ui01-fit-all.case.json`; only active zoom/pan changed for render review. The committed correctness fixture was not modified.
- **Screenshot:** [UI-01-self-loop-render-linux.png](UI-01-self-loop-render-linux.png), 1440×900, SHA-256 `e8315fe098fc53e1544cd19fcd2b7c2d0907c52bb43d617b5daee1985a215a08`.

## Source checks

| Check | Result | Evidence |
| --- | --- | --- |
| Targeted loop geometry and picking regression | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline correctness_fixture_routes_a_visible_self_loop -- --nocapture`, exit 0; 1 passed. It checks loop extent, route/label separation, an on-route arrowhead, exact self-loop selection, and distinct coincident endpoint routing/selection. |
| Full Rust suite | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`, exit 0; 27 passed, 0 failed, 0 ignored. |
| Formatting | **PASS** | `cargo fmt --all -- --check`, exit 0. |
| Locked check and debug build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline`, exit 0. |
| Optimized release build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline`, exit 0; binary digest above. |
| Diff whitespace | **PASS** | `git diff --check`, exit 0. |

The route function now distinguishes a self-link by endpoint identity. It samples a cubic loop instead of a quadratic path that retraced itself. Separate entities at coincident positions use a different visible route. Directed coincident routes place their arrowhead on the sampled curve. Self-loop and coincident-route labels are offset clear of route points. Both the renderer and hit-testing paths use the same curve helper.

## Native observation and limits

| Check | Result | Evidence |
| --- | --- | --- |
| Current release window launch and render | **PASS, render-only** | `env -u WAYLAND_DISPLAY XDG_DATA_HOME=/tmp/arch-notebook-ui01-xdg /tmp/arch-notebook-target/release/arch-notebook /tmp/arch-notebook-ui01-fit-all.case.json` remained live for capture. Spectacle saved the active window; image inspected at 1440×900. The self-loop label is placed beside its route, and the visible loop is no longer a single retraced stroke. The case copy used zoom 1.0; no Fit all action was performed, and several fixture items remain near or beyond the viewport edge. |
| Pointer/keyboard input | **BLOCKED** | `xdotool search --onlyvisible --name 'Arch Notebook'` failed with `Failed creating new xdo instance`. No event was delivered and no selection state was observed. |
| Screenshot transport stability | **PASS with retry** | The first Spectacle invocation aborted with exit 134; a subsequent invocation exited 0 and produced the retained screenshot. |
| IBM visual / ANX interoperability | **NOT TESTED** | No IBM application or schema was available for this check. |

This provides source-level geometry checks and one current-binary render observation. It does not complete the UI-00 native interaction baseline, verify actual pointer selection on the current window, establish IBM visual parity, or authorize release. Resume with an input-capable native session and perform the full UI-00 sequence; then inspect the self-loop selection in the Automation/Connections and chart views.
