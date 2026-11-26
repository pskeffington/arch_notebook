# SQL-AUTO-05: Current-branch SQL ingestion and automation recheck

- **Task / owner:** Continue the synchronized Arch Notebook build with emphasis on SQL ingestion and unattended automation; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source commit:** `8c08da9285c6104d35a66386676b244d44f1270e`.
- **Evidence-record baseline:** `991c4764715c71e682af784fbdbf9e2db3e833c7`.
- **Date / host:** 2026-11-26, Linux x86_64.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`; SHA-256 `1ef36e782956eaf1196a06d074ad8edf02591b45095d9acb805879b000cfe941`.
- **Fixtures:** fictional `archive/research.databases/sample_connections.sql` and generated joined email headers; no message bodies or operational records.

## Acceptance results

| Gate | Result | Evidence |
|---|---|---|
| Capture, longitudinal comparison, schema diagnosis, failure retention, batch recovery, CSV/TSV chart exchange | **PASS** | `python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-auto-991c476-20261126`, exit 0. Report SHA-256 `aa89d1a30d45abc02b34e87e16dfd69006bf2a842655732e2a3f670179be2724`. |
| Joined-header SQL scale | **PASS** | `python3 scripts/test-sql-scale-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --work-dir /tmp/arch-notebook-sql-scale-991c476-20261126`, exit 0. 7,121 rows, 1,500 entities, 7,121 composite record IDs, 4,121 repeated endpoint pairs, two snapshots, and zero skipped rows. Report SHA-256 `e1a0e72cc32a88e037c007aead1daf2be9ef64e2816bb5fdd3f06934987f5278`. |
| Linux systemd generation and service invocation | **PASS, stubbed** | Five cadences, invalid configuration rejection, install/removal safeguards, special-character paths, and a scheduled capture/export passed. `systemctl` was a temporary stub. Report SHA-256 `4a47af12766a675fdba5ee4cf2b7bee7b21756bd0bce26b558ca907928d47652`. |
| macOS LaunchAgent generation and runner simulation | **PASS, stubbed on Linux** | Five cadences, rollback, case isolation, special-character paths, rolling-window capture/export, and unchanged SQLite source passed. `launchctl` was a temporary stub; native macOS was untouched. Report SHA-256 `ffafa18f283d64beb161c240ecaa1aa25845ea6fd6f4d3ee451a8228e8ce178d`. |
| Native Automation UI | **BLOCKED** | Current release exits during Winit initialization with `WaylandError(Connection(NoCompositor))`; X11 reports `unable to open display :0`. `wayland-info` can enumerate compositor globals, but that does not prove the app opened or accepted input. |
| Live systemd/macOS scheduler dispatch | **NOT TESTED** | The harnesses replace host scheduler commands with temporary stubs. |
| IBM Analyst's Notebook interoperability | **NOT TESTED** | IBM's installed XSD and application were not available. Internal ANX checks remain separate. |

The SQL automation report records read-only source behavior during capture and preflight, one-run parameter isolation, retained failed runs, schema adaptation diagnostics and reviewed repair, interrupted batch resume without replaying successful jobs, configuration-mismatch rejection, and matching CSV/TSV round-trips. The scale fixture is synthetic; its deliberate baseline/current mutation occurs only in the harness-owned database.

All four compact JSON reports are committed under [`artifacts/`](artifacts/). These results support the current release's CLI and generated scheduler-runner behavior. They do not establish native UI usability, live timer delivery, suitability for production databases, IBM compatibility, or release readiness.
