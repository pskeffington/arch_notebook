# SQL-SCHED-10: Linux timer dispatch status

- **Task / owner:** Expose Linux scheduled-SQL timer timing in **Check schedule** / implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `e5b7ab35ca82ca124e4c50db66bbb285d06e9277`.
- **Scope:** Linux schedule status output and its scheduler acceptance harness. No saved SQL configuration, timer cadence, SQL capture behavior, or case data changed.
- **Runner binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `7a3dace1f04c4be70522b582c1d12a27cab47c4c85c60faff88bcab905abaf20`.

## Change

`arch-notebook-install-automation --status <case>` now asks the user systemd manager for `LastTriggerUSec` and `NextElapseUSecRealtime`. It reports the last timer dispatch and the next scheduled dispatch, or says timing is unavailable when the manager does not return those properties. A dispatch starts the service; it is not evidence of a successful SQL capture. **Check schedule** displays the status output, and the guide directs analysts to batch history for job results. The systemd timer interface defines these as separate timer timestamps; see the [systemd D-Bus timer properties](https://freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html).

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Shell syntax | PASS | `bash -n scripts/install-linux-automation.sh`, exit 0. |
| Harness syntax | PASS | `python3 -m py_compile scripts/test-linux-sql-scheduler-acceptance.py`, exit 0. |
| Whitespace validation | PASS | `git diff --check`, exit 0. |
| Linux SQL scheduler acceptance | PASS, stub-scoped | All five cadences, case-specific unit pairs, generated service capture, chart CSV, failed-preflight rejection, removal isolation, five `systemd-analyze calendar` parses, and status variants for timestamps, no recorded dispatch, and an unavailable user manager passed. Command: `python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-scheduler-status-acceptance-20261128-v3`, exit 0. The generated service saved one synthetic connection and one CSV row. `systemctl` was a temporary stub (45 calls), so this does not establish live dispatch. |
| Aggregate acceptance artifact | PASS | [SQL-SCHED-10 report](artifacts/SQL-SCHED-10-timer-status-linux.json), SHA-256 `132a4bb94b2e49568977758413ecd0818a9b44302348c3e365ed498a123543d6`. |
| Live host systemd manager | BLOCKED | `systemctl --user is-system-running` exited 1: user-scope bus access returned `Operation not permitted`. A private `dbus-run-session` probe also could not bind its bus socket in this sandbox. No host timer was installed or fired. |
| IBM application/schema compatibility | NOT TESTED | This Linux schedule status change does not test IBM ANX or SQL compatibility. |

## Changed paths

- `scripts/install-linux-automation.sh`
- `scripts/test-linux-sql-scheduler-acceptance.py`
- `docs/automation.md`
- `docs/build-acceptance.md`
- `docs/evidence/sql-automation/SQL-SCHED-10-timer-status-linux.md`
- `docs/evidence/sql-automation/artifacts/SQL-SCHED-10-timer-status-linux.json`

## Remaining work

Verify the status output against a real per-user systemd manager and separately confirm that a dispatched SQL batch appears in run history. macOS LaunchAgent status is unchanged. The next owner is Linux Operations when a user-bus-capable host session is available.
