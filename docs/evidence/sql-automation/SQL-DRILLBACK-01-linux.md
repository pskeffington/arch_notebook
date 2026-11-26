# SQL-DRILLBACK-01: Per-entity live SQL source lookup regressions

- **Task / owner:** Verify source-scoped, read-only live SQL lookup used by the selected-entity inspector; test-verification session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline / candidate:** `08e1a17503674ec93ded6cde7c672c9d3d17f968` → `a542d6f7bcfb1def441a52c48ec6c58eba0de400`.
- **Changed path:** `src/main.rs` (test-only additions; production SQL lookup implementation unchanged).
- **Date / host:** 2026-11-26, Linux x86_64; `rustc 1.100.0`, `cargo 1.100.0`.
- **Fixture:** generated temporary SQLite databases and case snapshots; each test removes its fixture directory on success. No user database or message corpus was read.

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| Focused lookup regressions | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline sql_source_record_lookup -- --nocapture`, exit 0; 3 passed. |
| Full Rust test suite | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`, exit 0; 31 passed, 0 failed, 0 ignored. |
| Locked compile check | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, exit 0. |
| Formatting and whitespace | **PASS** | `cargo fmt --all -- --check` and `git diff --check`, exit 0. |
| Native inspector workflow | **PENDING** | Tests call the lookup function directly; no window, pointer interaction, or rendered result was exercised. |

The source-scoping test creates two separate SQLite files with tenant-composite identities. It verifies matching when the selected entity appears as either endpoint, repeated local IDs remain separate across the sources, typed SQL parameters are bound, NULL becomes an empty display cell, BLOBs are shown as bounded hexadecimal summaries, and each returned result identifies the matching snapshot and canonical database path. The test compares serialized snapshots and source database bytes before and after lookup; both remain identical.

The safety test rejects a saved write statement and duplicate case-insensitive source-identity columns before accepting results, then confirms the database bytes did not change. The bounds test verifies the lookup returns no more than 20 matching rows and marks the result truncated at the 21st match; a 4,096-character cell is clipped and marked as truncated.

This closes a source-level regression gap in the SQL drillback path only. It does not verify the selected-entity inspector in a native UI, actual user schemas, live scheduler dispatch, IBM connectors or database identities, IBM visual parity, or release readiness. IBM documents its own per-source chart data records and separately linked database identities in its [data records](https://docs.i2group.com/anb/10.1.2/data_records.html) guide; Arch Notebook's live lookup is SQLite-only and read-only.
