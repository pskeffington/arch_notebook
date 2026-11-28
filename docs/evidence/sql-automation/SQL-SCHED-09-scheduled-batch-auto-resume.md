# SQL-SCHED-09 scheduled SQL batch auto-resume

- **Task / owner:** Resume an interrupted scheduled SQL batch safely on the next Linux systemd or macOS LaunchAgent invocation; Codex implementation and local verification.
- **Baseline:** `b1d75f849cd8581bfdb700c0f2d84121e697de03` on `codex/ui-00-chart-baseline`.
- **Scope:** Rust batch checkpoint selection, Linux and macOS scheduled runner arguments, policy and interruption acceptance, operator documentation.
- **Changed paths:** `src/main.rs`, `scripts/install-linux-automation.sh`, `scripts/run-macos-automation.sh`, `scripts/test-linux-sql-scheduler-acceptance.py`, `scripts/test-macos-sql-scheduler-acceptance.py`, `scripts/test-scheduled-sql-batch-resume.py`, `.github/workflows/build.yml`, `docs/automation.md`, `docs/analyst-notebook-parity.md`, this evidence file.

## Behavior

New batch manifests record their origin (`desktop`, `cli`, or `scheduled`) and a persistent failed-attempt marker; older manifests deserialize with unknown legacy metadata. The scheduled runner switch is accepted only when `ARCH_NOTEBOOK_AUTOMATION_TRIGGER=scheduled` and the invocation has no manual job selection or SQL parameter overrides. It considers only the newest retained batch. A batch resumes only when it is scheduled-origin, the checkpoint proves no job attempt has failed, it has a recorded pending selection, it exactly matches the currently enabled job IDs and effective SQL configuration, and its checkpoints validate. A completed, failed, changed, malformed, older, non-scheduled, or pre-change legacy batch starts a new scheduled batch (malformed eligible checkpoints fail closed); operators can explicitly resume an older batch with `--resume-batch`.

An automatic resume logs `AUTOMATION BATCH AUTO-RESUME · <batch-id>` to standard error and reports `auto_resumed` in JSON. Already checkpointed jobs are skipped; pending jobs run with the saved configuration. Ordinary CLI and desktop invocations keep their existing fresh-batch behavior.

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| Rust selection policy | PASS | Unit test covers eligible pending work, complete batches, failed captures, the persistent failure marker, legacy manifest decoding, latest overall batch precedence, and SQL configuration drift. |
| Real process interruption and scheduled recovery | PASS | Two-job fixture holds job 2's SQLite database exclusively, lets job 1 checkpoint, then SIGKILLs the scheduled process. After releasing the lock, the next scheduled invocation resumes the same batch, skips job 1, completes job 2, and saves exactly two scheduled runs/checkpoints. |
| Manual invocation boundary | PASS | The same acceptance script retries without the scheduled trigger, observes rejection, and confirms the case SHA-256 is unchanged. |
| Linux generated service and cadence acceptance | PASS | Five isolated cadence installations; service invocation includes the recovery flag; SQL preflight, case removal, paths, and all five calendar expressions pass with stubbed systemd commands. |
| macOS LaunchAgent runner acceptance | PASS | Five isolated cadence configurations; runner includes the recovery flag; the scheduled SQL capture and CSV export pass with stubbed launchctl commands. |
| Full Rust suite | PASS | 74 passed, 0 failed, 3 ignored. |
| Native systemd timer dispatch/reboot recovery | BLOCKED | `systemctl --user list-timers --all` exited 1: user scope bus denied with `Operation not permitted`. No live timer was installed or changed. |
| Native macOS launchd dispatch/reboot recovery | NOT RUN | This host is Linux; isolated runner simulation is not native launchd evidence. |

## Commands and observed exit codes

From `/home/sysadmin/github/arch_notebook`:

```text
cargo fmt --all
exit 0

env CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline
exit 0

env CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline scheduled_batch_auto_resume_requires_latest_matching_pending_batch_without_failures
exit 0; 1 passed

env CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline
exit 0; 74 passed, 0 failed, 3 ignored

env CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
exit 0

python3 scripts/test-scheduled-sql-batch-resume.py --binary /tmp/arch-notebook-target/debug/arch-notebook --work-dir /tmp/arch-notebook-scheduled-resume-final-20261128
exit 0; PASS; recovered batch aabea1df-f254-4e53-b48e-3d5f0488b7ff; one completed job skipped; manual invocation rejected without case changes

python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/debug/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-linux-scheduler-resume-final2-20261128
exit 0; PASS; five cadence configurations, 27 stubbed systemctl calls, five calendar expressions

python3 scripts/test-macos-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/debug/arch-notebook --installer scripts/install-macos-automation.sh --runner scripts/run-macos-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-macos-scheduler-resume-final2-20261128
exit 0; PASS; five cadence configurations, 23 stubbed launchctl calls

timeout 10s systemctl --user is-system-running
exit 1; degraded

systemctl --user list-timers --all
exit 1; Failed to connect to user scope bus: Operation not permitted
```

## Limits and next action

The process-interruption recovery path is exercised end to end against disposable SQLite files and the actual Rust CLI. Platform runner generation and invocation are validated with isolated stubs. Native Linux timer delivery and reboot/sleep persistence remain unverified because this user session cannot access the systemd user bus. Native macOS LaunchAgent behavior also remains unverified. No production scheduler state was changed.
