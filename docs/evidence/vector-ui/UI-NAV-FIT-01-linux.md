# Chart fit navigation acceptance

- **Task / owner:** Improve analyst navigation from an off-screen or over-zoomed chart; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source baseline:** `a356e0d06f9443aa3cb2f463392ebce8e7a16a18`.
- **Scope:** Add a searchable Quick open **Fit chart** command, a `Ctrl/Command+0` shortcut, and show that shortcut in **View…**. The action uses the existing chart-fit calculation and leaves a saved per-chart viewport intact until invoked.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Quick open command discoverability | PASS, source/test | The command is searchable by “fit” and “viewport”; `cargo test --offline quick_command_search_matches -- --nocapture` passed (1 test). |
| Rust regression suite | PASS | `cargo test --offline`: 34 passed, 0 failed. |
| Formatting and whitespace | PASS | `cargo fmt --all`; `git diff --check`; both exited 0. |
| Type/build checks | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0. |
| GitHub Actions for the pushed candidate | PENDING | GitHub Actions lookup returned no PR-triggered runs and the combined commit status had no contexts for `5cf1db5c573f69ce57f6322bb66bb2c958770de1`; no remote CI pass is claimed. |
| Current-candidate native window | BLOCKED | `env -u WAYLAND_DISPLAY /tmp/arch-notebook-target/release/arch-notebook --open-case "$PWD/fixtures/chart/correctness.case.json"` exited 1 with `XOpenDisplayFailed`. Current-candidate rendering, shortcut input, and viewport result were not observed. |
| Qwen visual review | BLOCKED | `qwen3.5:9b` appeared in one local Ollama model inventory response, but subsequent local `/api/chat` and `/api/tags` requests were refused. No Qwen critique is claimed. |

The available prior screenshot, [UI-00-native-window-20261126.png](UI-00-native-window-20261126.png), has SHA-256 `8396c9afdd7c42d7489941949201431fa9654aa9f12534f63fb68eb7e51ceb24`. Its fixture records a 200% zoom, which explains visible edge clipping in that image. The screenshot predates this change and is context only, not current-candidate acceptance. Native review should reopen the fictional correctness fixture, try Quick open search and `Ctrl/Command+0`, and verify that all entities fit at the target display size.
