# SQL schema adaptation and reviewed repair acceptance

- **Result:** PASS for the disposable Linux CLI workflow described below.
- **Source commit:** `86e2be5ac631730b577f52837795a493ff4a4287`
- **Binary:** `/tmp/arch-notebook-target/release/arch-notebook`
- **Binary SHA-256:** `858106a15a79657ac335e60f39782d7d5542c81f7936f89aadb2ccbdb83dc14a`
- **Fixture:** `archive/research.databases/sample_connections.sql` (fictional records)
- **Acceptance artifacts:** `/tmp/arch-notebook-sql-automation-adaptation-acceptance-86e2be5/`
- **Acceptance report:** `acceptance-report.json` in that directory

## Acceptance results

The full `scripts/test-sql-automation-acceptance.py` run passed with 23 expected CLI operations: 20 returned exit 0 and three expected failure probes returned exit 1. It retained the existing baseline/current capture, parameter override isolation, comparison, schema-stability rejection, failed capture retention and CSV/TSV chart-import round trips. The acceptance report and command output are in `/tmp/arch-notebook-sql-automation-adaptation-acceptance-86e2be5/acceptance-report.json`.

For schema adaptation, the harness renamed `event_date` to `observed_date` in its disposable SQLite database. SQLite inspection reported the new column. Running the existing job returned exit 1 and saved a failed run; the diagnosis was `missing_column` with reference `event_date`, `schema_changed=true`, `query_changed=false`, and `parameters_changed=false`. The schema-adaptation history filter returned that failed run both before and after repair.

The harness then updated the saved query explicitly, as an analyst-reviewed change. Query-only validation passed without changing the case file. The repaired capture passed and saved four connections. Hash checks confirmed that application inspection, validation, failed capture and repaired capture did not modify the SQLite source. The harness itself performed the deliberate schema rename before those checks.

The existing interchange acceptance also passed: CSV and TSV each exported and imported four connections, five entities, and four relationships with zero skipped rows; canonical row values matched between formats, including stable endpoint identifiers and custom source/relationship properties.

## Build and test evidence

All commands ran from the repository root after the implementation was committed:

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS: 14 passed, 0 failed, 1 ignored |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | PASS |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | PASS |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS |
| `PYTHONPYCACHEPREFIX=/tmp/arch-notebook-pycache python3 -m py_compile scripts/test-sql-automation-acceptance.py` | PASS |
| `python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-automation-adaptation-acceptance-86e2be5` | PASS |
| `git diff --check` | PASS |

The ignored test is the already documented UI-01 reversed-parallel-record lane collision, outside this SQL change. This evidence covers a local synthetic CLI flow; it does not establish native UI, scheduled execution, IBM application interoperability, or release acceptance.
