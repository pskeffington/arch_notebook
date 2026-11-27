# SQL-AUTO-07 · current installed release recheck

- **Task / owner:** Revalidate the current installed Rust release against SQL automation and joined-header scale acceptance; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Verification source:** `282f9de3a755557c856b16f50145d9f37936e882`.
- **Runtime source:** `e27ad5e842dd1c6d503f05c51844f12790502fa6`; later branch changes in `src/main.rs` are confined to `#[cfg(test)]` test coverage.
- **Binary:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`. The `/tmp/arch-notebook-target/release/arch-notebook` build has the same hash.
- **Host / data:** Linux x86_64; fictional `archive/research.databases/sample_connections.sql` plus a disposable synthetic joined email-header scale fixture. No real email content was used.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| SQL automation, diagnosis, recovery, and interchange | **PASS** | `python3 scripts/test-sql-automation-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-auto-282f9de-20261127 > /tmp/arch-notebook-sql-auto-282f9de-20261127.json` exited 0; report result is `PASS`. The report records successful preflight/capture/history/comparison, expected failures retained in history, schema diagnosis and reviewed repair, batch resume with one completed job skipped, changed-configuration resume rejection, and CSV/TSV design and data round trips with zero skipped records. |
| Read-only source and case-preservation checks | **PASS** | Report records identical SQLite hashes across successful captures and preflight, query preflight left the saved case unchanged, and configuration-mismatch rejection preserved the case. Failure fixtures deliberately mutate only disposable copies. |
| Joined-header scale capture | **PASS** | `python3 scripts/test-sql-scale-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --work-dir /tmp/arch-notebook-sql-scale-282f9de-20261127 > /tmp/arch-notebook-sql-scale-282f9de-20261127.json` exited 0: two snapshots, 7,121 records, 1,500 entities, 7,121 distinct composite record IDs, 4,121 repeated endpoint pairs, nullable attributes handled, and zero skipped rows. The harness reports the database unchanged during each capture. |
| GitHub Actions for the verification source | **NOT RUN** | GitHub Actions returned zero workflow runs and combined commit status returned an empty status list for `282f9de`. |
| Production schema, native Automation UI, live scheduler, IBM compatibility | **NOT TESTED** | Synthetic CLI acceptance does not establish these separate gates. |

## Reports

- Automation JSON: [SQL-AUTO-07-automation.json](artifacts/SQL-AUTO-07-automation.json), SHA-256 `1c46dc0a35ab6f010e123e74af49530433971490df3e5cbec3eae9a225cf8954`.
- Scale JSON: [SQL-AUTO-07-scale.json](artifacts/SQL-AUTO-07-scale.json), SHA-256 `ba9a16c114a8c068ff5d586586ae7955459a84779f81754cd45756363401ed28`.

These results revalidate the local installed release on the current branch's supported Rust automation behavior. They do not use the 7,000-email dataset and do not establish extraction completeness, email-level accuracy, Qwen analysis, analyst review, external data-source write-back, or IBM interoperability.
