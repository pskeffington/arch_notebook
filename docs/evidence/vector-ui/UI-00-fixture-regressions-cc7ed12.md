# UI-00 chart fixture regression evidence

- **Task / owner:** Extend the checked-in fictional chart regressions; Codex, Linux Operations.
- **Repository / branch:** `pskeffington/arch_notebook`, `codex/ui-00-chart-baseline`.
- **Baseline commit:** `f0107627e65c100faed90bfc06b889e8157c4677`.
- **Initial candidate commit:** `cc7ed12268bb71b094d63d5feeb1150b3a31f730`.
- **Diagnostic follow-up commit:** `4ba30b0` (improved failure output and reran the default suite and targeted defect check).
- **Date / host:** 2026-11-25, Linux x86_64, kernel `7.3.5-zen1-1-zen`; `rustc 1.100.0`, `cargo 1.100.0`.
- **Changed source:** `src/main.rs` test module only. Product behavior is unchanged.
- **Fixture:** `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`; Cargo.lock SHA-256 `4e14e9b55f812bfb1af0d7687039d4bc266a76f4a6f584dee8d7caa1349c01a3`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `460ad736650ca8288279cf2a0f36ddc28a6436340c160a588dc5a465cbc6ae8b`.

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| Formatting | PASS | `cargo fmt --all -- --check`, exit 0. |
| Default Rust tests | PASS with a known issue isolated | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`, exit 0; 13 passed, 0 failed, 1 intentionally ignored. |
| Self-loop route regression | PASS (unit geometry) | The fixture's self-loop receives a finite nonzero lane; its sampled curve has finite points, returns to its endpoint and leaves the endpoint by more than one logical point. This does not establish native rendering or picking. |
| Parallel-record lane uniqueness | FAIL; retained as ignored UI-01 regression | On follow-up `4ba30b0`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline correctness_fixture_assigns_distinct_lanes_to_parallel_records -- --ignored` exited 101 and identified fixture link indices 2 and 5 sharing lane `-36`. The records can visually overlap. The test is ignored in the default suite so the known defect remains explicit without making the source gate falsely pass. |
| Offline check | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, exit 0. |
| Debug build | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline`, exit 0. |
| Release build | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline`, exit 0; optimized build finished in 1m 46s. |
| Diff whitespace | PASS | `git diff --check`, exit 0. |
| Native UI-00 interaction and visual gate | BLOCKED | This pass did not deliver input to the Wayland app or inspect a captured window. See [native Wayland evidence](UI-00-linux-wayland-a84a938.md). UI-01 remains gated by the missing native baseline. |

## Defect handoff

The existing lane grouping sorts records by direction, then assigns centered offsets and negates offsets for reverse records. With multiple forward records and a reversed record, those operations can produce duplicate signed offsets. The checked-in fixture reproduces the defect at indices 2 and 5 with lane `-36`; the ignored assertion provides a runnable acceptance check for its repair.

**Next action:** after UI-00 native interaction and visual acceptance, repair deterministic lane allocation and exact record picking under UI-01. Remove the ignore only when the same targeted command passes; retain all records, stable identity and direction. Do not claim chart usability or IBM parity from this unit-level result.

## Repair follow-up — 2026-11-26

The lane collision was repaired after this historical failure report. `link_lane_offsets` now assigns each parallel record a lane in canonical endpoint order, then converts that lane to the record's local direction. The fixture assertion compares routes after direction normalization, so reversing a relationship cannot make it overlap a forward record. The check is active in the default test suite.

- **Changed source:** `src/main.rs` (`link_lane_offsets`); detailed current acceptance is recorded in [`UI-01-parallel-lanes-linux.md`](UI-01-parallel-lanes-linux.md).
- **Follow-up result:** targeted collision regression and full offline Rust suite PASS; native interaction and visual inspection remain BLOCKED.
