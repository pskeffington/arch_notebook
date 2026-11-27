# SQL-SCHED-08 · systemd calendar parser acceptance

- **Task / owner:** Validate generated Linux SQL automation timer cadence expressions with systemd's calendar parser; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `52ebc8e07836ffa8440e95851bd61aa6f653cbbc`.
- **Source candidate:** `d62e4a8ec0df84a00a2ed01e8aaeea8209dbbc5c`.
- **Scope:** Extend the Linux SQL scheduler acceptance harness to pass every generated `OnCalendar` expression to `systemd-analyze calendar` when available.
- **Runtime binary:** `/tmp/arch-notebook-target/release/arch-notebook`; SHA-256 `7f789d8fa53fb2fac444e2189767ffd266e3f49c537b2058d6175356fdf176c8`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Python syntax | **PASS** | `python3 -m py_compile scripts/test-linux-sql-scheduler-acceptance.py` exited 0. |
| Generated cadence expressions | **PASS** | `systemd-analyze calendar` accepted all five generated expressions: `hourly`, `*:0/15`, daily at 03:25, Tuesday at 15:45, and day 28 monthly at 23:59. Report records `/usr/bin/systemd-analyze` and five validated expressions. |
| Existing SQL scheduler acceptance | **PASS** | `python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-sql-sched-08-parser-20261127` exited 0. All five isolated cadence installs, preflight, service invocation, UTC half-open boundary, CSV export, invalid-query rejection, and per-case removal checks passed. |
| Exact-head cross-platform CI | **PASS** | Workflow run [45988117371](https://github.com/pskeffington/arch_notebook/actions/runs/45988117371) completed successfully on `94f6e9dadf8243db4a298067f02554125e1b38a8`. The Linux test job ran scheduler acceptance, including all five cadence parser checks. |
| Live Linux timer delivery | **NOT TESTED** | Harness uses a temporary `systemctl` stub. `systemctl --user is-system-running` cannot connect to the user bus in this execution environment (`Operation not permitted`). Calendar syntax validation does not establish timer installation, dispatch, restart recovery, or journal behavior. |
| IBM interoperability and native Automation UI | **NOT TESTED** | No IBM schema/application or native desktop interaction was exercised. |

## Evidence artifact

- [SQL-SCHED-08 acceptance JSON](artifacts/SQL-SCHED-08-linux.json), SHA-256 `1a33e369a2b08b0f7872647323a22b055b93fc052c4c50883770119b456db827`.

The new parser check is skipped with `NOT AVAILABLE` if `systemd-analyze` is absent. It supplements the generated-unit assertions; it is not a substitute for a live scheduler run.
