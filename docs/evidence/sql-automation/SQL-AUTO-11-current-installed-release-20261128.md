# SQL-AUTO-11 · current installed release SQL acceptance — 2026-11-28

**Task / owner:** Revalidate the installed Rust release with the SQL automation recovery, scale, and Linux scheduler harnesses / implementation session.

**Repository / branch:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions`.
**Product code commit:** `9293242198483edb977b7e0d49e778cfa149ad85`.
**Installed binary:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `4f37a78a91230b8c1aeafc16d0d3aedc18638dd83bb56eaf2172d504cf24c08d`.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| SQL automation, failure retention, reviewed schema repair, resume, CSV/TSV | PASS | `python3 scripts/test-sql-automation-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-auto-current-install-20261128-evidence` exited 0. Aggregate report: [SQL-AUTO-11 JSON](artifacts/SQL-AUTO-11-current-installed-release-20261128.json), SHA-256 `a348b239e8e67481e819724a2da908e4a6944fac0efb2ac8cc51f655c5a02713`. Query-only checks left the case unchanged; source hashes stayed unchanged during captures; expected failures were retained; reviewed schema repair and partial-batch resume passed; mismatched resume configuration was rejected without modifying the case. |
| Joined-header SQL scale | PASS | `python3 scripts/test-sql-scale-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --work-dir /tmp/arch-notebook-sql-scale-current-install-20261128-evidence` exited 0. 7,121 fictional rows captured as 7,121 distinct composite connection IDs, 1,500 endpoint entities, 4,121 repeated endpoint pairs, and zero skipped rows across two snapshots. Aggregate report: [SQL-SCALE-02 JSON](artifacts/SQL-SCALE-02-current-installed-release-20261128.json), SHA-256 `1c6dfde1f7a252238cc1abed3be279eb2897cb59322c838acc762bee9a0d04ea`. |
| Linux schedule installation / generated runner | PASS, stub-scoped | `python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-linux-scheduler-current-install-20261128-evidence` exited 0. Five cadences, isolated case schedule pairs, special-character paths, failed-preflight no-install behavior, dynamic UTC window, generated scheduled invocation, chart CSV, and timer-status variants passed. `systemctl` was a temporary stub. Aggregate report: [SQL-SCHED-11 JSON](artifacts/SQL-SCHED-11-current-installed-release-20261128.json), SHA-256 `481da061d8e5cb94e8e0ec796b4c97e28d544cc9c6e3be6b95c93a3079d7645c`. |
| Live Linux timer dispatch, sleep/reboot recovery | NOT TESTED | The harness does not install or await a live host timer. Earlier host user-manager access was denied by the sandbox. |

All suite reports are aggregate acceptance artifacts; source database fixtures were fictional and isolated under `/tmp`. Separately, [CLINTON-CORPUS-QA-05](CLINTON-CORPUS-QA-05-installed-release-20261128.md) passes read-only SQL preflight and replay against the primary real corpus. IBM's optional seeded External Searches workflow and the Arch Notebook product gap are summarized in the [parity map](../../analyst-notebook-parity.md#seeded-external-query-research).
