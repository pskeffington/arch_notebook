# SQL-CAPTURE-07: Partial batch resume on Linux

- **Result:** PASS for the CLI recovery scenario; native scheduler and interactive cancellation remain unverified.
- **Owner:** Arch Notebook implementation session.
- **Baseline:** `f1df9b5b129cac4835bc07145335d836416c4223` on `codex/ui-00-chart-baseline`.
- **Scope:** Synthetic SQLite acceptance for durable job checkpoints, same-batch resume, completed-job skipping, and configuration mismatch rejection.
- **Changed paths:** `scripts/test-sql-automation-acceptance.py`, `README.md`, and this evidence record.
- **Fixture:** `archive/research.databases/sample_connections.sql`; a separate disposable database and case were created under `/tmp/arch-notebook-sql-resume-acceptance-20261125c`.
- **Application binary SHA-256:** `178554daa2fa799da921c51551e8d46796296a1ca5681237819f8002d27e0058` (`/tmp/arch-notebook-target/release/arch-notebook`).
- **Acceptance script SHA-256:** `7d342e2e0f0c0fcea3dcf375d6f185e2590e8131f12edf996ae92579c4afbc47`.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| `PYTHONPYCACHEPREFIX=/tmp/arch-notebook-pycache python3 -m py_compile scripts/test-sql-automation-acceptance.py` | PASS, exit 0 | Script syntax compiled. |
| `python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-resume-acceptance-20261125c` | PASS, exit 0 | Full SQL acceptance report: `/tmp/arch-notebook-sql-resume-acceptance-20261125c/acceptance-report.json`. Existing capture, schema, failure, CSV/TSV, and query-repair checks passed alongside recovery. |
| `git diff --check` | PASS, exit 0 | No whitespace errors in the candidate patch. |

The recovery subcase first exited 1 as expected with one successful run and one missing-view failure. The successful job was the only checkpoint. After adding the view to the disposable SQLite database, resuming the same batch exited 0, reported one prior successful job skipped, and added one successful run. A resume with a changed `snapshot_id` parameter exited 1 with a configuration diagnostic; the case SHA-256 remained unchanged across that rejected attempt.

The overall report has SHA-256 `b3c7fa958189d2df4cbb009a28acf4e260352181a26db8ba718cbec769085f7e`. The synthetic resumed case has SHA-256 `8ef5d35dfaad360b855c4ca70ccf314204832654014f4238da4d2918d5dbaebb`; the recovery SQLite file has SHA-256 `4832c32bffe6289037444901835faa3c9efdfac28a19c40a9e3acaa6bbd4f490`.

## Limits and next action

This does not exercise a native systemd timer, sleep/reboot recovery, a running long query cancellation, or desktop automation UI. It makes no IBM application interoperability claim. Next: run the native Linux scheduler and interactive UI acceptance when a usable display and scheduler host are available, then continue the UI-00 navigation and analyst workflow work.
