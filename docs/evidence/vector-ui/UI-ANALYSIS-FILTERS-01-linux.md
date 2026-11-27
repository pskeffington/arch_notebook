# UI-ANALYSIS-FILTERS-01 · compact analysis filter layout

- **Task / owner:** Improve Most connected filter readability in the chart analysis pane; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline / product source:** `487473d0ff655e0b9fc0584fcd8dd59a7e320dbb` / `1dd4cdcb204f20ba610e8b4dedf2f776744aae9b`.
- **Changed product path:** `src/main.rs`.
- **Release / installed binary SHA-256:** `7bb98beff1db4321fabec1258ddfc67574deb1a44dfb0cee45d1ee522b36840f`.
- **Host:** Linux x86_64; current-user launcher only.

## Change

The **Rank by** and **Connected by** filters now use separate rows in the Analysis pane. Selected relationship labels longer than 20 Unicode characters display as a bounded prefix plus an ellipsis; the full value remains available in the control's hover tooltip and in the explanatory line below it. Ranking semantics and available options are unchanged.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Unicode-safe label truncation | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline analysis_filter_label_truncation_is_unicode_safe_and_stable -- --nocapture` exited 0. It checks short values, the exact character bound, and a multibyte Unicode label. |
| Default and minimum window layout | **PASS, headless UI** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline headless_analysis_filter_controls_stack_and_fit_default_pane -- --nocapture` exited 0. It asserts row order, selected-label clipping bounds, full-label disclosure, and fit at 1440×900/default width and 1000×650/minimum window width. |
| Full Rust suite | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 60 passed, 0 failed. |
| Locked check, debug/release builds, formatting and diff validation | **PASS** | The repository-standard offline check and both builds exited 0; `cargo fmt --all -- --check` and `git diff --check` exited 0. Build artifacts are in `/tmp/arch-notebook-target`. |
| Per-user launcher and graphics | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-sync-ui-filters/arch-notebook-linux-x86_64` exited 0. Installed executable matches the release hash above; menu and Desktop entries validate and match byte-for-byte. Installed SVG/PNG match repository artwork; automation helper matches `scripts/install-linux-automation.sh`; the case MIME default is `arch-notebook.desktop`. |
| Exact-candidate native window | **BLOCKED** | Launch command `env -u WAYLAND_DISPLAY DISPLAY=:0 /tmp/arch-notebook-target/release/arch-notebook /home/sysadmin/github/arch_notebook/fixtures/chart/correctness.case.json` exited 1 with Winit `XOpenDisplayFailed`. Earlier screenshots belong to older product sources and are not evidence for this change. |
| Exact-source GitHub Actions | **NOT RUN** | No Actions run for this source commit was present at evidence capture. The workflow does not run on feature-branch pushes; no PR currently targets this branch. |
| IBM Analyst's Notebook comparison | **NOT TESTED** | This source and headless UI verification do not establish visual parity or IBM interchange compatibility. |

The focused tests validate the rendered headless widget layout and text clipping, but not native pointer, keyboard, tooltip timing, accessibility, or visual comparison with Analyst's Notebook. Those remain open gates.
