# SQL capture chart CSV/TSV round-trip evidence

- **Task / owner:** Verify the SQL capture → chart CSV/TSV → saved import design → chart import workflow; Codex, Linux Operations.
- **Repository / branch:** `pskeffington/arch_notebook`, `codex/ui-00-chart-baseline`.
- **Harness source commit:** `b365fdc093e6129d004a98467246c6cdd8c1839f`.
- **Product binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `460ad736650ca8288279cf2a0f36ddc28a6436340c160a588dc5a465cbc6ae8b`. Runtime product code is unchanged from its earlier Rust release build; this pass changes only Python acceptance infrastructure.
- **Harness:** `scripts/test-sql-automation-acceptance.py`, SHA-256 `9153a4e6e71bbbeb9efdfed64a75a8be5d5cc17a74c5e711575ea9e723f3f250`.
- **Host:** Linux x86_64, kernel `7.3.5-zen1-1-zen`; `rustc 1.100.0`, `cargo 1.100.0`.
- **Fixture SQL:** `archive/research.databases/sample_connections.sql`, SHA-256 `5bf58753d01310afa53366098c9ee6c5535f4ea74a5b4e5e84708275a133e9df`.
- **Machine report:** `/tmp/arch-notebook-sql-automation-acceptance-final/acceptance-report.json`, SHA-256 `8614da53dc4343d6ebd01e3264f64347d4ac7bfa2d3ee4995b50eb4ee6ff54b7`.
- **Captured source DB after the harness's intentional schema change:** SHA-256 `8d0f987b984d5ec24ae63efc9e37a14200f3f3dab66e3060391d216d4a66a460`.

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| Python harness syntax | PASS | `PYTHONPYCACHEPREFIX=/tmp/arch-notebook-pycache python3 -m py_compile scripts/test-sql-automation-acceptance.py`, exit 0. |
| SQL capture and comparison flow | PASS | Baseline/current each saved four records; one-run override did not alter the configured job; run history, batch checkpoints and three expected comparison rows were preserved. |
| Schema-stability preflight | PASS (expected rejection) | The harness added a column to its disposable SQLite DB; preflight returned exit 1 with `changed=true`, `passed=false`, `required=true`. Neither the case nor the database changed during preflight. |
| Failed capture retention | PASS (expected capture failure) | With maximum regressions set to zero, capture returned exit 1 and `FAIL`, but saved the case, snapshot and failed run. `--list-runs` returned the failure record. The SQLite source hash remained unchanged during the run. |
| SQL chart CSV export/import | PASS | Exported four connections; generated and imported a reusable design; imported into an empty case as four new connections, five entities, zero skipped. Record identities, endpoint names/types, relationship labels, date, confidence, direction, source note/reference, source-scoped endpoint IDs, endpoint properties, relationship properties and ingest source matched the export. |
| SQL chart TSV export/import | PASS | Same checks passed for tab-separated output. |
| CSV/TSV equivalence | PASS | Canonicalized row values from both exports matched exactly. |
| Export side effects | PASS | Source case SHA-256 stayed unchanged during chart export; SQLite source SHA-256 stayed unchanged during chart export/import. Imports modified only disposable copies. |
| Full acceptance harness | PASS | Command exited 0. Seventeen CLI operations ran: fifteen expected exit 0; schema-policy rejection and configured regression failure each returned their expected exit 1. |
| IBM Analyst's Notebook interoperability | PENDING | This proves Arch Notebook's own exporter/importer round-trip only. IBM schema and application validation still require a receiving installation. |
| Native Automation UI and live scheduler firing | PENDING | Not exercised by this CLI workflow. |

Exact command:

```text
python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-automation-acceptance-final
exit 0
```

The generated CSV, TSV and imported case files are under `/tmp/arch-notebook-sql-automation-acceptance-final`. Their hashes and per-format counts are recorded in the machine report. This synthetic local pass is not IBM application validation or production acceptance.
