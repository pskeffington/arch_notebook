# SQL-SCHED-03: Scheduled rolling SQL time window on Linux

- **Result:** PASS for SQL date-window selection through the generated service command; live systemd timer dispatch remains unverified.
- **Owner:** Arch Notebook implementation session.
- **Baseline:** `4494bf664693e6038de588c21f3a7aa8b5d4c06f` on `codex/ui-00-chart-baseline`.
- **Changed paths:** `scripts/test-linux-sql-scheduler-acceptance.py`, `docs/automation.md`, `README.md`, and this evidence record.
- **Binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `0614576294efd356f606081dc437d22672e0e0a48b4612f86b436a3e32f694b9`.
- **Harness SHA-256:** `595667daa10bc970048e519d8f42b450f2e1059cc361b80d5f7d1c49aad8e7c3`.
- **Input SQL fixture SHA-256:** `5bf58753d01310afa53366098c9ee6c5535f4ea74a5b4e5e84708275a133e9df`.
- **Work directory:** `/tmp/arch-notebook-linux-scheduler-window-acceptance-20261126a`; all cases, database changes, units, logs, and CSV output were isolated there.
- **Acceptance report:** `/tmp/arch-notebook-linux-scheduler-window-acceptance-20261126a/acceptance-report.json`, SHA-256 `82e111eb45355bbf82e8e222aaf8b08c4e2fe99e8289d072de7ff4fad1496a73`.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| `PYTHONPYCACHEPREFIX=/tmp/arch-notebook-pycache python3 -m py_compile scripts/test-linux-sql-scheduler-acceptance.py` | PASS, exit 0 | Scheduler harness syntax compiled. |
| `python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-linux-scheduler-window-acceptance-20261126a` | PASS, exit 0 | Full report says `PASS`; all five cadence installers, isolated units, CLI preflights, failed-preflight behavior, and the generated service invocation passed. |
| Rolling-window boundary selection | PASS | The service query returned only `WINDOW-START` for `[2026-10-01, 2026-11-01)`. It excluded `BEFORE-WINDOW` and the row exactly at the exclusive `WINDOW-END`. |
| Scheduled snapshot and CSV | PASS | The real Arch Notebook CLI, invoked with the generated service `ExecStart`, saved one scheduled-trigger run, one connection snapshot, and a one-row chart CSV. |
| `git diff --check` | PASS, exit 0 | No whitespace errors in the candidate source and documentation changes. |

The harness computes the boundary rows using SQLite's own UTC `date('now', ...)` expressions, then runs the saved SQL through the generated service command. This checks the same calendar semantics users configure in a scheduled query. Systemd itself was stubbed for installation lifecycle checks, and the user service manager did not dispatch a real timer.

## Handoff

The scheduled query example is now covered by executable acceptance, and the Automation guide clarifies that saved parameter values remain fixed while SQLite date functions can roll scheduled time windows. Continue with native Automation interaction and real timer/restart acceptance when a stable desktop and user systemd manager are available. This does not close UI-00 or establish IBM application interoperability.
