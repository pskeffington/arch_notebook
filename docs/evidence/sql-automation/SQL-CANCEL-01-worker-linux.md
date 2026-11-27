# SQL-CANCEL-01: Cancelled desktop automation worker run

- **Result:** PASS for worker cancellation and persistence; native button interaction remains unverified.
- **Owner:** Arch Notebook implementation session.
- **Baseline:** `83807838e985d4cae549a801abd06ab7c6cde347` on `codex/ui-00-chart-baseline`.
- **Scope:** Regression coverage for cancelling the active SQL job in a desktop automation worker after an earlier job in the batch succeeded.
- **Changed paths:** `src/main.rs`, `README.md`, and this evidence record.
- **Fixture:** Fictional case and disposable SQLite database created under `/tmp` by the Rust test; the repository fixture is used only as a template.
- **Source SHA-256:** `a572a828f4fe705c5ad3091399c0fd1fb8d0e01aefec27de46b858c2a0e86aef`.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline cancelling_desktop_sql_automation_saves_run_and_keeps_prior_checkpoint -- --nocapture` | PASS, exit 0 | Focused worker cancellation regression passed. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS, exit 0 | All 18 tests passed; 0 failed and 0 ignored. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | PASS, exit 0 | Workspace check succeeded. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | PASS, exit 0 | Debug build succeeded. |
| `cargo fmt --all -- --check` | PASS, exit 0 | Formatting clean. |
| `git diff --check` | PASS, exit 0 | No whitespace errors. |

The regression executes one successful synthetic SQL job, starts a long-running SQLite query as the second job, and sets the same cancellation atomic used by the desktop control. It confirms that the second job is recorded as cancelled, the first job's snapshot and batch checkpoint remain, the case persists both run records, and the source database bytes do not change.

## Limits and next action

**Follow-up:** the cancellation regression now drives the actual Automation cancel button through a headless egui event test and verifies the running UI, cancellation state, retained checkpoint, and unchanged SQLite source. See [SQL-CANCEL-02 headless UI evidence](SQL-CANCEL-02-headless-ui-linux.md). Native pointer/visual acceptance and live systemd timer dispatch remain unverified; the user-scope systemd bus denies access as recorded in `SQL-SCHED-01-linux.md`.
