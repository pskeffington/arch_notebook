# SQL-AUTO-08 — current release with primary Clinton QA

- **Task / owner:** Revalidate SQL automation and the primary Clinton corpus on the current installed release; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source:** `30b8020` (`Restore topology line hover preview`); evidence-only follow-ups are on the branch tip.
- **Runtime binary:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `330bc4fe2446bf8c7bff685ac4e56ddc5ffabc7b82cb59199b2cf537bd0d09b0`.
- **Host:** Linux x86_64. All checks used the installed release binary; no production database or live schedule was modified.

## Results

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Clinton primary batch, saved SQL preflight and replay | **PASS** | `python3 scripts/test-clinton-corpus-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/clinton-corpus-qa-30b8020.json` exited 0. Query-only preflight passed; latest saved-run replay returned 6,173 rows, zero skipped rows, and zero deltas. Input hashes remained unchanged. The default-redacted packet includes all 2,813 entities, keeps all labels in bounds, and contains no names, relationship labels, IDs, or evidence references. |
| Synthetic SQL recovery, schema diagnosis, and interchange | **PASS** | `python3 scripts/test-sql-automation-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-auto-30b8020-20261127` exited 0. Query-only preflight, expected failure retention, diagnosed schema change, reviewed repair, resumable batch, changed-configuration resume rejection, and CSV/TSV round trips passed. Source DB hashes matched during captures; failed and mismatched preflights preserved their intended case state. |
| Joined-header scale workload | **PASS** | `python3 scripts/test-sql-scale-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook` exited 0. Two snapshots captured 7,121 of 7,121 fictional joined-header rows with zero skips; 7,121 composite record IDs and 4,121 repeated endpoint pairs were retained, nullable attributes were handled, and the fixture DB remained unchanged during each capture. Message bodies and real people were not used. |
| Linux scheduler generation and cadence parsing | **PASS, stubbed** | `python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-sql-sched-30b8020-20261127` exited 0. Five cadence expressions passed `systemd-analyze`; case isolation, failure preflight, half-open UTC window, generated service invocation, CSV output, and case-specific removal passed. `systemctl` was a temporary stub; live timer delivery remains untested. |
| Full Rust suite and optimized release build | **PASS** | 73 passed, 0 failed, 3 ignored; locked offline release build exited 0. Details are in [topology hover acceptance](../vector-ui/PROXIMITY-MAP-HOVER-02-linux.md). |
| Cross-platform CI / production-shaped schema review | **PENDING** | Feature-branch push does not trigger the workflow; production schema coverage remains an analyst-owned follow-up. |
| Qwen, human content review, PDF completeness, and IBM validation | **NOT TESTED** | These data-quality and interoperability gates are outside the SQL automation harness. No semantic interpretation is claimed. |

## Aggregate evidence artifacts

- Clinton corpus report: [CLINTON-CORPUS-QA-03 JSON](artifacts/CLINTON-CORPUS-QA-03-current-release-20261127.json), SHA-256 `67b7ac6dda638594a1ec1a4a3a461595fb09125a65777b6a6ce4b89c2f945ca2`.
- SQL automation report: [SQL-AUTO-08 JSON](artifacts/SQL-AUTO-08-current-release-20261127.json), SHA-256 `ea1887361896fc75d58c4381a92f5ddad510101f2017634dd0ff7dc655ac0d2b`.
- Joined-header scale report: [SQL-AUTO-08 scale JSON](artifacts/SQL-AUTO-08-scale-current-release-20261127.json), SHA-256 `a122fb599269bb67c8fa514d90c4e7e66f60e0acaa0ecd44720276d6d24cec0d`.
- Linux scheduler report: [SQL-SCHED-09 JSON](artifacts/SQL-SCHED-09-current-release-20261127.json), SHA-256 `09f3654fb2f2c6c6617fa3aa57b7dbd96fdef85dcfa2b4383617d9a0cf9d1176`.

The Clinton report contains aggregate counts and file digests only; it does not contain email names, bodies, subjects, or relationship rows. Qwen analysis and email completeness are still unverified.
