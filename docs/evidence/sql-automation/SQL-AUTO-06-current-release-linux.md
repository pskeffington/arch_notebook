# SQL-AUTO-06: Current branch SQL and scale recheck

- **Task / owner:** Recheck SQL automation on the synchronized Arch Notebook release; implementation session.
- **Repository / branch / source:** `/home/sysadmin/github/arch_notebook`, `codex/ui-00-chart-baseline`, `e86814675eaee96ed6f20caff94d6d5b79b9a2c5`.
- **Release binary:** `/tmp/arch-notebook-release-verify/release/arch-notebook`, SHA-256 `d4bfcdc0e8f7fb3c374e92b4ebca2e21b2920e3211550a2b9536b6ac88d35676`.
- **Host:** Linux x86_64; Rust `1.100.0`.
- **Fixtures:** fictional `archive/research.databases/sample_connections.sql` plus the harness-generated joined email-header database. No real messages or email bodies were used.

## Acceptance results

| Gate | Result | Evidence |
| --- | --- | --- |
| Optimized offline build | **PASS**, exit 0 | `CARGO_TARGET_DIR=/tmp/arch-notebook-release-verify cargo build --release --locked --offline -j 1`. `--version --json` reports Arch Notebook `0.1.0`. |
| SQL capture, comparison, diagnosis, recovery, and chart exchange | **PASS**, exit 0 | `python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-release-verify/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-auto-e868146-20261126`. [JSON report](artifacts/SQL-AUTO-06-automation.json), SHA-256 `48b0e4574a695894f234abfc0b4d17e28654157764447138e02f197c7d670466`. |
| Joined-header SQL scale | **PASS**, exit 0 | `python3 scripts/test-sql-scale-acceptance.py --binary /tmp/arch-notebook-release-verify/release/arch-notebook --work-dir /tmp/arch-notebook-sql-scale-e868146-20261126 > /tmp/arch-notebook-sql-scale-e868146-20261126.json`. [JSON report](artifacts/SQL-AUTO-06-scale.json), SHA-256 `a7340078e39708f649a8da700e761c8086d874f9d717b4aac7cd39b6042f143c`. |
| Local executable and desktop sync | **PASS**, scoped | Installed `/home/sysadmin/.local/bin/arch-notebook` matches the release binary SHA-256; menu and desktop entries validate and compare byte-for-byte; icons and automation installer match repository assets; `gio mime` resolves case files to `arch-notebook.desktop`. See [LOCAL-INSTALL-02](../deployment/LOCAL-INSTALL-02-linux.md). |

The automation flow confirms preflight and capture do not modify the SQLite source; retains failed runs and structured missing-column diagnosis; validates a reviewed repair; resumes a partial batch without rerunning its successful job; rejects changed batch configuration without case mutation; and round-trips four chart relationships through both CSV and TSV. The scale fixture captures 7,121 relationship records with 1,500 entities, 7,121 distinct composite record IDs, 4,121 repeated endpoint pairs, two comparable snapshots, nullable attributes, and zero skipped rows. Source mutation occurs only inside the disposable scale harness between baseline and current snapshots.

## Limits

This evidence covers a current-source CLI release on fictional SQLite data. It does not establish native Automation UI behavior, live Linux systemd timer dispatch, native macOS LaunchAgent dispatch, production-schema suitability, or IBM Analyst's Notebook interoperability. The systemd user bus denied access in this session. No release was published.
