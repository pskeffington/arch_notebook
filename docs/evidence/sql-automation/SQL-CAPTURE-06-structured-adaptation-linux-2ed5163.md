# Structured SQL schema adaptation acceptance

- **Result:** PASS for the disposable Linux CLI workflow below.
- **Source commit:** `2ed51632e2bbe501b2b288de84cc51491a4d042d`
- **Binary:** `/tmp/arch-notebook-target/release/arch-notebook`
- **Binary SHA-256:** `178554daa2fa799da921c51551e8d46796296a1ca5681237819f8002d27e0058`
- **Fixture:** `archive/research.databases/sample_connections.sql` (fictional records)
- **Acceptance artifacts:** `/tmp/arch-notebook-sql-structured-adaptation-acceptance-2ed5163/`
- **Acceptance report:** `acceptance-report.json` in that directory

## Acceptance results

The full SQL automation harness passed all 23 expected CLI operations: 20 returned exit 0 and three failure probes returned the expected exit 1. It exercised baseline/current capture, one-run parameter override isolation, run and batch history, comparison deltas, schema-stability rejection, failed capture retention, CSV/TSV chart-import round trips, schema inspection, missing-column diagnosis, explicit query repair, and a successful repaired capture.

The harness renamed `event_date` to `observed_date` in its disposable SQLite database. The failed run stored `schema_adaptation` as structured case data with `kind=missing_column`, `reference=event_date`, and a repair hint. The run-history filter retained and surfaced the diagnosis after the query was explicitly corrected and the subsequent four-record capture passed. Query-only validation did not alter the case file. Hash checks verified the application did not modify the SQLite source; the harness itself performed the deliberate test schema rename.

Older run JSON without the new optional field remains readable: a Rust regression test deserializes a legacy record and confirms the compatibility fallback derives the missing-column classification and reference from its existing error message.

CSV and TSV each round-tripped four relationship records, five entities, and four relationships with zero skipped rows; canonical record values matched across both formats, including stable endpoint identifiers and custom properties.

## Build and test evidence

All commands were run against the committed source tree:

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS: 15 passed, 0 failed, 1 ignored |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | PASS |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | PASS |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS |
| `PYTHONPYCACHEPREFIX=/tmp/arch-notebook-pycache python3 -m py_compile scripts/test-sql-automation-acceptance.py` | PASS |
| `python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-structured-adaptation-acceptance-2ed5163` | PASS |
| `git diff --check` | PASS |

The ignored test is the known UI-01 reversed-parallel-record lane collision documented in [the UI fixture regression report](../vector-ui/UI-00-fixture-regressions-cc7ed12.md). This evidence is synthetic CLI and source-level acceptance; native UI review, live scheduling, external IBM application validation, and release acceptance remain separate gates.
