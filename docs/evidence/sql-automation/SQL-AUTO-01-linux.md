# SQL-AUTO-01 Linux acceptance evidence

- **Task / owner:** SQL automation acceptance; Codex implementation and local verification.
- **Baseline:** `6cca0fa309a1a783aa971084d0ed6f912494c63a` on `codex/ui-00-chart-baseline`.
- **Scope:** Exercise the CLI's query-only validation, SQL automation runs, per-run parameter override, persisted run and batch history, and snapshot comparison export against the fictional `sample_connections.sql` fixture. Wire the acceptance script into Linux CI.
- **Changed paths:** `.github/workflows/build.yml`, `scripts/test-sql-automation-acceptance.py`, this evidence file.

## Acceptance criteria and results

| Check | Result | Evidence |
| --- | --- | --- |
| Query preflight succeeds and leaves the case file unchanged | PASS | `query_preflight`, exit 0; harness compares case bytes before and after. |
| Baseline and one-run override each save a four-record snapshot | PASS | `baseline_batch` and `current_override_batch`, exit 0; two snapshots with four records each. |
| Run history contains successful CLI runs and expected deltas | PASS | `run_history`, exit 0; `removed`, `confidence_declined`, `source_notes_changed`, and `added` verified. |
| Per-run override does not rewrite saved job parameters | PASS | Saved parameter remains `snapshot_id=text:baseline`. |
| Batch checkpoints persist and comparison CSV exports expected changes | PASS | `batch_history` reports two batches; comparison export has three expected rows (removed C-104, changed C-102, added C-105). |
| Configured SQLite source remains unchanged | PASS | SHA-256 before and after: `cbb7a0a273b1b036145f89d652ab28adf24e1c8ac0177a98cf31f206c5b17d9c`. |
| Rust unit tests, formatting, diff check and debug build | PASS | Commands and exit codes below. |
| Remote CI execution | NOT RUN | No PR is open for this feature branch; the workflow's direct push trigger is limited to `main` and tags. The integration step is now in the workflow for PR or manual dispatch. |

## Commands and observed exit codes

From `/home/sysadmin/github/arch_notebook`:

```text
python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/debug/arch-notebook --source-sql archive/research.databases/sample_connections.sql
exit 0

cargo fmt --all -- --check
exit 0

git diff --check
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline
exit 0 (11 passed, 0 failed)

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
exit 0

python3 -m py_compile scripts/test-sql-automation-acceptance.py
exit 0
```

The harness report was generated under `/tmp/arch-notebook-sql-acceptance-t4m75try/acceptance-report.json`; its disposable directory is local evidence, not a repository artifact. The fixture SQL file SHA-256 is `5bf58753d01310afa53366098c9ee6c5535f4ea74a5b4e5e84708275a133e9df`.

## Limits and next action

This validates the fictional SQLite fixture and CLI path. It does not test live scheduling/service behavior, IBM application interoperability, Qwen analysis, or native visual baselines. Those checks remain pending. CI integration has been authored but is not reported as remotely executed. After commit, push the feature branch and retain the remote commit SHA as delivery evidence.
