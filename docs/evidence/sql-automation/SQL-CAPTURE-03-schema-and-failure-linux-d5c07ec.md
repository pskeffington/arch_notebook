# SQL schema-drift and failed-run acceptance evidence

- **Task / owner:** Extend the fictional SQL automation acceptance harness to cover schema-stability rejection and persisted failure records; Codex, Linux Operations.
- **Repository / branch:** `pskeffington/arch_notebook`, `codex/ui-00-chart-baseline`.
- **Harness source commit:** `d5c07ecac4b967606bdac69575da35b01e80d59a`.
- **Product binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `460ad736650ca8288279cf2a0f36ddc28a6436340c160a588dc5a465cbc6ae8b`. Product Rust source was unchanged from its previous release build; this pass changes only the Python acceptance harness.
- **Harness:** `scripts/test-sql-automation-acceptance.py`, SHA-256 `30b0f66d27570b15e03d1fc5ed1a41cb210ad9678d16479e5570dece1e4e992b`.
- **Host:** Linux x86_64, kernel `7.3.5-zen1-1-zen`; `rustc 1.100.0`, `cargo 1.100.0`.
- **Fixture SQL:** `archive/research.databases/sample_connections.sql`, SHA-256 `5bf58753d01310afa53366098c9ee6c5535f4ea74a5b4e5e84708275a133e9df`.
- **Acceptance report:** `/tmp/arch-notebook-sql-automation-final-93582e5/acceptance-report.json`, SHA-256 `243b64d4623bb5e59b951ff49297b2c96c3423f23eee01e9347f02776b591162`.
- **Generated case:** `/tmp/arch-notebook-sql-automation-final-93582e5/failure.case.json`, SHA-256 `ed3ef75c8ff436a5ff60c43d80a35fa2ad85514f485b81694541d018432e9352`.
- **Generated SQLite source after intentional test schema change:** SHA-256 `8d0f987b984d5ec24ae63efc9e37a14200f3f3dab66e3060391d216d4a66a460`.

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| Python harness syntax | PASS | `PYTHONPYCACHEPREFIX=/tmp/arch-notebook-pycache python3 -m py_compile scripts/test-sql-automation-acceptance.py`, exit 0. |
| Baseline/current capture flow | PASS | Each capture saved a four-record snapshot; the one-run parameter override left the saved job configuration unchanged. |
| Snapshot comparison | PASS | History retained expected added, removed, confidence-declined and source-note changes; CSV export contained the expected three changed records. |
| Read-only capture behavior | PASS | SQLite source SHA-256 was identical before and after both captures. |
| Schema-stability gate | PASS (expected rejection) | Harness deliberately added a column to the disposable source DB, then ran `--check-enabled-job-queries --require-unchanged-schema`. CLI returned exit 1 with schema drift reported. The case was not saved; the case-file SHA-256 and modified DB SHA-256 were each identical before and after preflight. |
| Failed capture persistence | PASS (expected capture failure) | Harness set maximum regressions to 0 and ran a baseline recapture. CLI returned exit 1 and `FAIL`, while saving the snapshot and failure run. The database hash stayed unchanged. The saved case parsed successfully; `--list-runs` returned all three records, including the failure. |
| Full acceptance harness | PASS | `scripts/test-sql-automation-acceptance.py` exited 0. Nine CLI operations ran: seven expected exit 0 and two expected exit 1 (schema gate, configured regression failure). |
| Native Automation UI and live scheduler firing | PENDING | This CLI run does not establish interactive review or execution by a live user-session scheduler. |

Exact acceptance command:

```text
python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-automation-final-93582e5
exit 0
```

This is synthetic local acceptance. It validates the CLI policy and persistence paths, not IBM application interoperability, real analyst data, native GUI review, crash-injection atomicity, or live scheduler timing.
