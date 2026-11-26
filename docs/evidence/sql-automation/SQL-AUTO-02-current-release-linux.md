# SQL-AUTO-02: Current-release SQL ingestion and scheduler harnesses

- **Result:** PASS for synthetic SQL capture, schema/failure/recovery workflow, chart exchange, generated Linux service invocation, and macOS runner simulation. **BLOCKED / not tested** for live systemd timer and macOS launchd dispatch.
- **Task / owner:** Continue Arch Notebook SQL automation acceptance; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source tip:** `bc536880d29075234525c0fde4f6a860caf71101`.
- **Product source commit:** `a5208a0e3aa8c0fe3b3a1e3f02986e45e97b919a` (`feat: add analysis method card`; later commits are documentation-only).
- **Installed binary:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `765f8fa2e6384a2df4747890a9b954920a1668dc35557068f749f89afcea1fc7`.
- **Fixture scope:** Synthetic SQLite data only; the 7,121-row workload contains joined email headers and no message bodies or operational records.

## Acceptance

| Check | Result | Evidence |
| --- | --- | --- |
| 7,121-row SQL-scale capture | PASS, exit 0 | `python3 scripts/test-sql-scale-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --work-dir /tmp/arch-notebook-sql-scale-evidence-bc53688/work` captured 7,121 distinct composite relationship IDs into two snapshots, 1,500 endpoint entities, and 0 skipped rows. Query-only preflight passed; NULL attributes stayed absent rather than becoming text markers; the confidence decline compared across snapshots; each capture left its SQLite source unchanged. JSON report SHA-256 `8d242884f1f6e2a1df75d1a8271917b50e0dd85334c5df9ab93d6e12f9c094c4`. |
| SQL automation and recovery acceptance | PASS, exit 0 | `python3 scripts/test-sql-automation-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-acceptance-current-bc53688` passed preflight, baseline/current captures, run and batch history, comparison export, schema-stability rejection, failure retention, CSV/TSV round trips, schema diagnosis and reviewed query repair, and batch resume. The changed-configuration resume was rejected with exit 1 and left the case unchanged; its successful-checkpoint resume skipped one completed job and reran the repaired job. JSON report SHA-256 `4bbf885d86d2e0d4ca3cc1baba8306cb89ed4732c6d1ed10c895afdd72f5d55d`. |
| Linux scheduler harness | PASS, harness-scoped | `python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-linux-sched-bc53688` tested five cadences and case-isolated units. A generated service command exited 0, saved one `scheduled` capture and one CSV row. The `systemctl` executable was a temporary stub (27 calls); no live user manager/timer dispatch was tested. Report SHA-256 `c7db7a2f822498382ae711c7e786eb5e85a1245207f2daf10e577e97acf30882`. |
| macOS LaunchAgent runner harness | PASS, simulation-scoped | `python3 scripts/test-macos-sql-scheduler-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --installer scripts/install-macos-automation.sh --runner scripts/run-macos-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-macos-sched-bc53688` tested five cadences, failure rollback, and case isolation. The bundled runner produced the expected scheduled capture and CSV row while the source DB stayed unchanged. `launchctl` was a temporary Linux stub (23 calls); real macOS launchd was untouched. Report SHA-256 `05d899681bba55a061ad7f1e88ff5492b9aa2ef0716d11532c2618d65d5f23e8`. |

The scale harness's planned fixture mutation creates the second snapshot state; its “source unchanged” assertion applies to each application capture, not the full harness lifetime. The scheduler reports exercise the generated service command and bundled runner, not live OS dispatch. No IBM application, IBM XSD, external data source, or real schedule was used.

## Handoff

The current installed Rust release passes the SQL ingest/recovery and generated-scheduler harnesses. Continue with native Automation panel interaction, long-running progress/cancel review, and live dispatch on supported Linux/macOS hosts. Keep those results separate from the passing harness evidence above.
