# SQL-SCALE-01 joined header capture acceptance

- **Task / owner:** SQL automation scale and identity acceptance / Arch Notebook maintainer
- **Source commit:** `620e733978d0d508c6467d78d1e2e74b98ac90d8`
- **Branch:** `codex/ui-00-chart-baseline`
- **Scope:** Add a continuous integration acceptance harness for joined relational SQL capture with composite identities, nullable properties, duplicate endpoint pairs, and a 7,121-row result.
- **Data:** Fully synthetic header-only data. No real people, message bodies, or operational data are included.
- **Binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `178554daa2fa799da921c51551e8d46796296a1ca5681237819f8002d27e0058`. The source commit changes the acceptance harness and workflow only; the Rust product source is unchanged from the tested build.
- **SQLite fixture before the deliberate harness update:** SHA-256 `54f6ec697d50b4340d0b78cfbb48b3c6acab659631cc4d42d1946f66ad3058e6`.
- **SQLite fixture after the deliberate harness update:** SHA-256 `37fe6a12f6e1d44d692f0fa21b539beb09cff57d51b8c84e9afb9709d7b47fb8`.
- **Result:** **PASS** for the synthetic workflow below.

## Acceptance results

The schema contains contacts keyed by `(tenant_id, person_id)`, messages, and recipient rows keyed by `(message_id, recipient_ordinal)`. The query joins sender and recipient contacts, constructs stable composite entity and connection IDs, and returns nullable department, thread, and source-reference fields.

Query-only preflight did not change the case or SQLite source. Two CLI captures retained 7,121 relationships and 1,500 entities per snapshot; all 7,121 connection IDs remained distinct. The graph retained 4,121 repeated endpoint pairs as separate relationship records. Nullable attributes were accepted without converting SQL NULL to a string marker, and no rows were skipped. Run history reported the deliberate confidence decline in the second snapshot.

Between captures, the Python acceptance harness intentionally lowered the confidence of one fixture message. The application left the database hash unchanged during each preflight/capture operation; the second hash above reflects that intentional harness edit, not an application write.

The final recorded capture durations were 254 ms and 125 ms for this host and fixture. These are single-run observations, not a performance target or service guarantee. The harness verifies 1,200 contacts with a non-null department value and 6,105 relationships with a non-null thread token; SQL NULLs are omitted from entity/link properties rather than stringified.

## Commands and results

| Command | Result |
| --- | --- |
| `PYTHONPYCACHEPREFIX=/tmp/arch-notebook-pycache python3 -m py_compile scripts/test-sql-scale-acceptance.py` | PASS |
| `python3 scripts/test-sql-scale-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --work-dir /tmp/arch-notebook-sql-scale-acceptance-7121-final` | PASS; 7,121 rows, two captures, zero skipped rows |
| `python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-existing-acceptance-7121` | PASS; 23 expected operations (20 exit 0, 3 expected failure probes exit 1) |
| `git diff --check` | PASS |

The build workflow now runs the large joined SQL acceptance on its Linux test job after building the CLI. GitHub workflow status for the feature branch must be checked after the CI-triggering pull request or workflow run is available.

## Limits and next work

This is synthetic SQL CLI acceptance, not validation against the local Clinton email candidate case or a production database. Native scheduler firing/restart recovery, long-running interactive cancellation, representative user query drillback, and IBM Analyst's Notebook compatibility remain separate gates.
