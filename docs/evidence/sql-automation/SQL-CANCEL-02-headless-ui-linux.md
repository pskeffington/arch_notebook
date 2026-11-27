# SQL-CANCEL-02: Headless Automation cancel-button workflow

- **Task / owner:** Verify the desktop SQL Automation cancellation event path; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `4a53052013cb7e8b1d64dd7fc924aff8adfc26ea`; test-only follow-up in `src/main.rs`.
- **Fixture:** disposable SQLite database and fictional case created by the regression test under the system temp directory.
- **UI harness:** `egui::Context::run` at 1440×900 with synthetic pointer input; no native window or compositor was used.

## Acceptance path

The headless test starts a two-job desktop automation batch. Job 1 saves a successful SQL snapshot; job 2 runs a cancellable recursive SQLite query. While job 2 is active, the test renders the running Automation screen and clicks the visible **Cancel automation** button through the same `App::update` event path used by the desktop UI. It confirms the cancellation state, waits for the worker result through subsequent app frames, then verifies:

- The active query is recorded as cancelled and both job outcomes remain in run history.
- Job 1's completed snapshot and batch checkpoint remain available; the pending job is not marked complete.
- The disposable SQLite source bytes are unchanged.
- The running screen displayed job progress and the cancel action.

## Commands and results

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | **PASS**, exit 0 |
| `git diff --check` | **PASS**, exit 0 |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline headless_ui_cancels_desktop_sql_automation_and_keeps_prior_checkpoint -- --nocapture` | **PASS**, 1 passed |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, 55 passed, 0 failed |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | **PASS**, exit 0 |

## Limits

This verifies the egui event path and worker persistence contract, not native pointer behavior, visual quality, accessibility, display responsiveness, systemd timer dispatch, or macOS LaunchAgent dispatch. Native Automation review remains **NOT RUN** because this session cannot access a compositor or X11 display. The test-only change received implementation-owner review; no separate independent review handoff is recorded.
