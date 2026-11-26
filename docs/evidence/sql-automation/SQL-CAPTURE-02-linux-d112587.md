# SQL automation capture acceptance evidence

- **Task / owner:** Re-run the scripted SQL capture/comparison workflow on the current notebook branch; Codex, Linux Operations.
- **Repository / branch:** `pskeffington/arch_notebook`, `codex/ui-00-chart-baseline`.
- **Repository source:** `d1125871334311324635f52e463bba834dac1c38`.
- **Binary build source:** product code at `cc7ed12268bb71b094d63d5feeb1150b3a31f730`; later commits touched only `#[cfg(test)]` regression assertions and documentation. Binary SHA-256: `460ad736650ca8288279cf2a0f36ddc28a6436340c160a588dc5a465cbc6ae8b`.
- **Host:** Linux x86_64, kernel `7.3.5-zen1-1-zen`; `rustc 1.100.0`, `cargo 1.100.0`.
- **Fixture SQL:** `archive/research.databases/sample_connections.sql`, SHA-256 `5bf58753d01310afa53366098c9ee6c5535f4ea74a5b4e5e84708275a133e9df`.
- **Generated source database:** `/tmp/arch-notebook-sql-acceptance-d112587/sample.sqlite`, SHA-256 before and after capture `cbb7a0a273b1b036145f89d652ab28adf24e1c8ac0177a98cf31f206c5b17d9c`.
- **Detailed machine report:** `/tmp/arch-notebook-sql-acceptance-d112587/acceptance-report.json`, SHA-256 `28bfe5d6c822f31dde85bc0fd0a8836a330ac4c08a187e9591a67c82b110ac01`.

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| Query-only preflight | PASS | Exit 0; report says the case was not saved or changed by preflight. |
| Baseline and current captures | PASS | Both batch commands exited 0; each produced a four-record snapshot and a saved successful CLI run. |
| One-run override isolation | PASS | Current capture used `snapshot_id=text:current`; saved job parameters remained `snapshot_id=text:baseline`. |
| Snapshot deltas and run history | PASS | History and batch listings exited 0. Expected `removed`, `confidence_declined`, `source_notes_changed`, and `added` kinds were present. Two batch checkpoints were recorded. |
| Comparison CSV | PASS | Export exited 0 and contained the expected removed C-104, changed C-102 and added C-105 rows; three data rows total. |
| Source database immutability | PASS | SHA-256 before and after was identical. |
| Script result | PASS | `scripts/test-sql-automation-acceptance.py` exited 0; all six reported CLI commands exited 0. |
| Native GUI, schema-drift and scheduled-host acceptance | PENDING | This scripted CLI pass does not establish interactive review, schema-policy behavior, or live scheduler firing on the host. Existing isolated installer tests remain separate evidence. |

Exact command:

```text
python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-acceptance-d112587
exit 0
```

This pass uses only the checked-in fictional SQLite fixture. It does not establish real-source acceptance, IBM application interoperability, native desktop behavior, or scheduler execution by a live user session.
