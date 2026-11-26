# UI-01 parallel relationship lane repair

- **Task / owner:** Resume vector organization work by repairing overlap among parallel and reversed relationship records; implementation session.
- **Repository / branch:** `pskeffington/arch_notebook`, `codex/ui-00-chart-baseline`.
- **Baseline commit:** `17cc145541cc148ea84d44baaa14e5f42c3d57c4` (selection follow-up; lane repair baseline recorded below).
- **Changed paths:** `src/main.rs`, README, and this evidence file.
- **Scope:** Keep all relationship records and identities. Assign lanes in canonical endpoint order and translate the lane into each record's local direction. Resolve pointer hits against the rendered route and preserve exact record identity. No persistence or dependency changes.

## Acceptance

| Check | Result | Evidence |
| --- | --- | --- |
| Targeted reversed/parallel route and selection regression | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline correctness_fixture_assigns_distinct_lanes_to_parallel_records -- --nocapture`, exit 0; 1 passed. It checks route non-overlap and pointer-to-exact-record resolution for every parallel/reversed link in the fixture. |
| Rust unit suite | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`, exit 0; 27 passed, 0 failed, 0 ignored. |
| Formatting | **PASS** | `cargo fmt --all -- --check`, exit 0. |
| Locked offline compile check | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, exit 0. |
| Debug build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline`, exit 0. |
| Release build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline`, exit 0. |
| Whitespace check | **PASS** | `git diff --check`, exit 0. |
| Native chart interaction and visual inspection | **BLOCKED** | This code-level route test does not deliver input or inspect native rendering. Continue the UI-00 native sequence when the host input/capture path is available. |
| IBM Analyst's Notebook visual/format validation | **BLOCKED** | No IBM application validation was performed. |

## Result and remaining work

The known fixture collision is fixed and its test now passes as part of the normal suite. Canonical lane assignment ensures that a reversed relationship's rendered route is normalized before selecting its lane, preventing duplicate routes caused by negating an already-centered local lane. The same fixture now exercises the extracted hit-test helper at each route and verifies exact record identity. Native pointer input/visual quality, broader chart usability, IBM parity, and release readiness remain unverified.

**Next action:** complete native UI-00 interaction and visual acceptance; then continue UI-01 chart organization and picking with the same fictional fixture and preserve the explicit pending gates.
