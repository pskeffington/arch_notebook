# UI-01 parallel relationship lane repair

- **Task / owner:** Resume vector organization work by repairing overlap among parallel and reversed relationship records; implementation session.
- **Repository / branch:** `pskeffington/arch_notebook`, `codex/ui-00-chart-baseline`.
- **Baseline commit:** `97d2c972f48369b793ba1b337219620e8466f52c`.
- **Changed paths:** `src/main.rs`, this evidence file, the UI-00 fixture regression history, and the social graphing roadmap.
- **Scope:** Keep all relationship records and identities. Assign lanes in canonical endpoint order and translate the lane into each record's local direction. No persistence or dependency changes.

## Acceptance

| Check | Result | Evidence |
| --- | --- | --- |
| Targeted reversed/parallel route regression | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline correctness_fixture_assigns_distinct_lanes_to_parallel_records -- --nocapture`, exit 0; 1 passed. |
| Rust unit suite | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`, exit 0; 27 passed, 0 failed, 0 ignored. |
| Formatting | **PASS** | `cargo fmt --all -- --check`, exit 0. |
| Locked offline compile check | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, exit 0. |
| Debug build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline`, exit 0. |
| Release build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline`, exit 0. |
| Whitespace check | **PASS** | `git diff --check`, exit 0. |
| Native chart interaction and visual inspection | **BLOCKED** | This code-level route test does not deliver input or inspect native rendering. Continue the UI-00 native sequence when the host input/capture path is available. |
| IBM Analyst's Notebook visual/format validation | **BLOCKED** | No IBM application validation was performed. |

## Result and remaining work

The known fixture collision is fixed and its test now passes as part of the normal suite. Canonical lane assignment ensures that a reversed relationship's rendered route is normalized before selecting its lane, preventing duplicate routes caused by negating an already-centered local lane. Native UI quality, exact record picking, broader chart usability, IBM parity, and release readiness remain unverified.

**Next action:** complete native UI-00 interaction and visual acceptance; then continue UI-01 chart organization and picking with the same fictional fixture and preserve the explicit pending gates.
