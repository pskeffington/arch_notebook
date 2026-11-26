# SQL-SCHED-05: Current-branch rolling SQL window acceptance

- **Result:** PASS for Linux generated-service and macOS packaged-runner harnesses on the current installed release; actual systemd timer and macOS `launchd` dispatch remain unverified.
- **Task / owner:** Verify SQL scheduling behavior against current Arch Notebook product source; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source:** `a09b49c6faddbd0401c9ac59b191e8151a830e70`; documentation-only branch head during this acceptance: `cb0ff938ae1328b3f9a9859b19edae874f82138c`.
- **Binary SHA-256:** release and installed binary both `54d57b05cd9ebb66de4b60274fb4062e06a96705f4a68f80eabb7649e249bdb7`.
- **SQL fixture SHA-256:** `5bf58753d01310afa53366098c9ee6c5535f4ea74a5b4e5e84708275a133e9df`.
- **Linux work directory:** `/tmp/arch-notebook-linux-scheduler-window-acceptance-20261126c`.
- **macOS runner work directory:** `/tmp/arch-notebook-macos-scheduler-window-acceptance-20261126c` (Linux-hosted simulation).
- **Report SHA-256:** Linux `967b8e6056b47238620a02ed6602632dc6861fcb78b34d24da39aba2aaf7ba84`; macOS runner `39c7729aae7d3d8b803e0537a2792021e81399e46d3ec8e60afadaf36ea10a37`.
- **Post-run SQLite SHA-256:** `7c62cccec9459451a0c00da3cbb35265be7bf36f7ddb81c6bf18527c0d1a3071`.

## Commands and outcomes

| Command | Result | Evidence |
| --- | --- | --- |
| `python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-linux-scheduler-window-acceptance-20261126c` | PASS, exit 0 | Report says `PASS`; all five cadence installers and isolated case-specific unit pairs passed; generated service invocation saved one scheduled capture and one-row chart CSV. |
| `python3 scripts/test-macos-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-macos-automation.sh --runner scripts/run-macos-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-macos-scheduler-window-acceptance-20261126c` | PASS, exit 0 | Report says `PASS`; five case-specific LaunchAgent schedules, failed-install cleanup, rollback, and case-removal checks passed using a temporary `launchctl` stub. |
| Rolling previous-UTC-month row selection | PASS in both harnesses | SQLite interval was `[2026-10-01, 2026-11-01)`; only `WINDOW-START` was included. `BEFORE-WINDOW` and the exclusive `WINDOW-END` were excluded. |
| macOS-runner SQLite immutability | PASS | Report confirms source database unchanged; post-run digest is recorded above. |

Both harnesses used disposable synthetic SQLite databases. The Linux run used a temporary `systemctl` stub; the macOS runner test used a temporary `launchctl` stub and ran on Linux. These results do not prove live system scheduler dispatch. The SQL code path, rather than a production or real Clinton email data source, was exercised.
