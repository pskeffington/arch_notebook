# SQL automation current-branch acceptance

- **Task / owner:** Recheck SQL automation, recovery, and joined-header scale behavior on the synchronized feature branch; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source commit:** `750b6cb9f9214089678cc3341dae5ecd7c65005b`.
- **Date / host:** 2026-11-26, Linux x86_64; `rustc 1.100.0`, `cargo 1.100.0`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `f8c14cabd73164b27218068685d75c62ecd2e9d854287299b9aca534ec373032`.
- **Fixtures:** `archive/research.databases/sample_connections.sql` and the harness-generated fictional joined email-header schema; no operational messages or email bodies.

## Commands and results

| Check | Result | Evidence |
| --- | --- | --- |
| Full SQL automation acceptance | **PASS** | `python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-auto-20261126-0723`, exit 0. Report SHA-256 `7571c8a7a783022b2be9701f7a47be420ccaf97cd77a6880a0ed1a5902850fab`. |
| Joined-header scale acceptance | **PASS** | `python3 scripts/test-sql-scale-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --work-dir /tmp/arch-notebook-sql-scale-20261126-0724`, exit 0. Captured 7,121 rows and 1,500 entities in two snapshots. Captured stdout report `/tmp/arch-notebook-sql-scale-report-20261126-0724.json`, SHA-256 `e0a35c2c27771603a66b4d5f4f5ea038c828f615068783ead1be34947c14b6f0`. |
| Native UI-00 interaction and visual acceptance | **BLOCKED** | `wayland-info` exited 255 with `failed to create display: Operation not permitted`; `DISPLAY=:0 xrandr --query` could not open the display. No native input or screenshot review is claimed. |
| IBM XSD/application validation | **BLOCKED** | IBM schema/application was not available or exercised. Internal SQL acceptance does not establish IBM interoperability. |

## Verified behavior

The full workflow reports PASS for query-only preflight without case mutation; baseline and one-run override isolation; run/batch history and comparison export; schema-drift preflight; failed capture retention; structured missing-column diagnosis; reviewed query validation and repair; interrupted batch resume with one completed job skipped; configuration-mismatch rejection without case mutation; and CSV/TSV chart round-trips with identical values. The source SQLite database remained unchanged during all app query/capture/preflight and chart exchange steps. Expected failure commands exited 1 and were accepted by the harness.

The scale run reports PASS for a joined header-shaped workload with composite entity and relationship IDs, repeated endpoint pairs, nullable attributes, 7,121 relationship records, two immutable comparable snapshots, and detection of a deliberate confidence decline. Query preflight passed, no rows were skipped, and each app capture left its SQLite source unchanged. The harness itself mutates only its disposable database between baseline and current snapshots.

## Limits and next action

These are automated CLI and synthetic-database results. They do not prove interactive Automation panel behavior, live Linux systemd timer dispatch, live macOS LaunchAgent dispatch, production-schema suitability, IBM application interoperability, or release readiness. Continue SQL workflow review on native supported platforms and validate representative analyst-owned schemas; retain IBM and native UI gates separately.
