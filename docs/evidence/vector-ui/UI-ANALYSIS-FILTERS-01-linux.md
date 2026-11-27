# UI-ANALYSIS-FILTERS-01 · compact analysis filter layout

- **Task / owner:** Improve Most connected filter readability in the chart analysis pane; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline / product source:** `487473d0ff655e0b9fc0584fcd8dd59a7e320dbb` / `6dd04e4bfd16e3bb14eafa7b7634474ed3a51322`.
- **Changed product path:** `src/main.rs`.
- **Release / installed binary SHA-256:** `766229457ca4de9975ace1a044bdde1e8802b6bc9ccd4f5bb60bdf904a83274f`.
- **Host:** Linux x86_64; current-user launcher only.

## Change

The **Rank by** and **Connected by** filters now use separate rows in the Analysis pane. Selected relationship labels use a pane-width-based limit of 8–20 Unicode characters plus an ellipsis; both the selected filter and its explanatory note stay bounded, while the full value remains available through hover help. Ranking semantics and available options are unchanged.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Unicode-safe label truncation | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline analysis_filter_label_truncation_is_unicode_safe_and_stable -- --nocapture` exited 0. It checks short values, the exact character bound, and a multibyte Unicode label. |
| Default and minimum window layout | **PASS, headless UI** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline headless_analysis_filter_controls_stack_and_fit_default_pane -- --nocapture` exited 0. It asserts row order, width-scaled selected-label clipping, full-value hover disclosure, and fit at 1440×900/default 320px pane width and 1000×650/minimum 240px pane width, including wide Unicode text. |
| Full Rust suite | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 60 passed, 0 failed. |
| Locked check, debug/release builds, formatting and diff validation | **PASS** | The repository-standard offline check and both builds exited 0; `cargo fmt --all -- --check` and `git diff --check` exited 0. Build artifacts are in `/tmp/arch-notebook-target`. |
| Per-user launcher and graphics | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-sync-ui-filters/arch-notebook-linux-x86_64` exited 0. Installed executable matches the release hash above; menu and Desktop entries validate and match byte-for-byte. Installed SVG/PNG match repository artwork; automation helper matches `scripts/install-linux-automation.sh`; the case MIME default is `arch-notebook.desktop`. |
| Exact-candidate native process/window creation | **PASS, scoped** | The exact launch command remained live; `xwininfo -root -tree` listed an Arch Notebook X11 window at 1440×900. |
| Exact-candidate native visual review | **BLOCKED** | `spectacle --new-instance --activewindow --background --nonotify --no-decoration --output /tmp/arch-notebook-analysis-filters-6dd04e4.png` aborted with exit 134. No current-candidate screenshot or visual review is claimed. Host input injection also remains unavailable. |
| Exact-source GitHub Actions | **NOT RUN** | No Actions run for this source commit was present at evidence capture. The workflow does not run on feature-branch pushes; no PR currently targets this branch. |
| IBM Analyst's Notebook comparison | **NOT TESTED** | This source and headless UI verification do not establish visual parity or IBM interchange compatibility. |

The focused tests validate the rendered headless widget layout and text clipping, but not native pointer, keyboard, tooltip timing, accessibility, or visual comparison with Analyst's Notebook. Those remain open gates.
