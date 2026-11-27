# SQL-SCHED-07 · current installed release scheduler-runner recheck

- **Task / owner:** Revalidate generated Linux systemd and macOS LaunchAgent SQL runners using the installed Rust release; implementation session.
- **Repository / branch / verification source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `179aff68d32e59e49f411a18f17e90144903712d`.
- **Runtime source / binary:** product behavior from `e27ad5e842dd1c6d503f05c51844f12790502fa6`; `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.
- **Host / fixture:** Linux x86_64; fictional `archive/research.databases/sample_connections.sql`; all scheduler files, service invocations, LaunchAgents, and databases were confined to the named `/tmp` work directories.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Linux installer and generated systemd service invocation | **PASS, stub harness** | `python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-sched-linux-179aff6-20261127 > /tmp/arch-notebook-sched-linux-179aff6-20261127.json` exited 0. Five cadences passed; the generated service command used the real Arch Notebook CLI and recorded a scheduled-trigger capture with one relationship and one chart CSV row. Invalid preflight installed no schedule; case-specific removal preserved other schedules; special-character paths and UTC rolling-window start-inclusive/end-exclusive behavior passed. |
| macOS installer and generated LaunchAgent runner | **PASS, Linux stub harness** | `python3 scripts/test-macos-sql-scheduler-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --installer scripts/install-macos-automation.sh --runner scripts/run-macos-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-sched-macos-179aff6-20261127 > /tmp/arch-notebook-sched-macos-179aff6-20261127.json` exited 0. Five cadences passed; the generated runner recorded a scheduled-trigger capture and one chart CSV row. Failed first install left no agent; failed replacement restored the prior plist/agent; the real macOS launchd service was untouched. |
| Live Linux systemd timer dispatch | **NOT TESTED** | `systemctl` was a temporary stub. This verifies generated unit/command behavior, not host timer delivery. |
| Native macOS LaunchAgent dispatch | **NOT TESTED** | `launchctl` was a temporary stub on Linux. No macOS host was used. |
| Native Automation UI and IBM compatibility | **NOT TESTED** | These harnesses do not exercise desktop interaction, IBM schemas, or Analyst's Notebook. |

## Reports

- Linux harness: [SQL-SCHED-07-linux.json](artifacts/SQL-SCHED-07-linux.json), SHA-256 `1eca563327eeb8814c82bff6a6c0cafa5549da82f4addef0ae825c1ac17e961e`.
- macOS harness: [SQL-SCHED-07-macos.json](artifacts/SQL-SCHED-07-macos.json), SHA-256 `218b211a7fa3242a3e7dfaa747d33238c10998529d4d9188ac715c44257a64e0`.

The report JSON files preserve cadence lists, generated-runner output, schedule-removal and rollback results, UTC boundary rows, stub call counts, and work-directory paths. Passing the stub harnesses does not imply live timer delivery, reboot/sleep recovery, or platform-native UI acceptance.
