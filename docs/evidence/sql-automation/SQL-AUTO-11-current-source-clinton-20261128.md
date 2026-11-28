# Current-source Clinton SQL automation revalidation

- **Task / owner:** Verify the current Rust candidate's SQL query preflight and saved-run replay against the retained Clinton QA case / orchestrator.
- **Repository / branch / baseline:** `pskeffington/arch_notebook`, `codex/ui-00-chart-nested-conditions`, source `54c1836b61f9d61aa6800d539d2b1a627c36871a`.
- **Platform:** Linux x86_64; locked offline development build.
- **Data source:** SQLite opened through the saved job's read-only query path. Database SHA-256 before and after: `fe9dcdf26f3810a0df3905ddf217cfd45704fd412f1509e1e379d2bb1483e071`.
- **Saved case:** SHA-256 before and after: `6143c16d3f66964991f80184d6e2a4d165553bf9443ac3cb6bfbeea930fa255b`.

## Acceptance results

| Gate | Result | Evidence |
| --- | --- | --- |
| Locked offline Rust debug build | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0. |
| Formatting / workspace tests / locked offline check | PASS | `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline` (78 passed, 0 failed, 3 ignored), and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline` all exited 0. |
| Locked offline optimized release build | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline` exited 0 in 2m 11s. Binary SHA-256: `d5fcb5b6bba6f6b53f1bddf3bca3105a0ae638555cb00931080bcd2131c706c3`. |
| Clinton query-only job validation | PASS | Current binary returned `ok=true`, `status=PASS`, 6,173 connections, 0 skipped rows, and `saved=false`; command exited 0. |
| Saved-run replay | PASS | Current binary replayed the saved run on a disposable case copy; `ok=true`, 1 passed, 0 failed, 6,173 connections, 0 skipped rows, and all delta counters zero; command exited 0. |
| Source immutability | PASS | The original case and SQLite hashes match the recorded hashes before/after. Replay wrote only to `/tmp/clinton-sql-acceptance/preflight-case.json`. |
| Human review and extraction completeness | NOT ESTABLISHED | All records remain review-required candidates. This SQL replay does not verify source PDF completeness or email meaning. |

Exact commands:

```sh
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline
cargo fmt --all -- --check
/tmp/arch-notebook-target/debug/arch-notebook --validate-job /tmp/clinton-sql-acceptance/preflight-case.json adbaf7d1-2fed-4dc2-8e14-dec6d1e2a28b --query-only --json
/tmp/arch-notebook-target/debug/arch-notebook --rerun-run /tmp/clinton-sql-acceptance/preflight-case.json 710d9e87-8945-4a6e-833f-e2cad4b93ca6 --json
```

The disposable case copy and full command outputs are local `/tmp` artifacts and are not committed because they contain row-level source values. This acceptance is limited to deterministic SQL capture/replay behavior; it does not validate semantic extraction, IBM interoperability, or scheduled desktop service dispatch.
