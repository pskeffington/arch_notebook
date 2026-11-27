# SQL-DRILLBACK-02 — joined-query source record drillback

- **Task / owner:** Extend read-only live SQL drillback regression coverage to joined analyst queries; implementation session.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-baseline` / `17ed57eb14f50b7277285766aeecd7d1e24999cc`.
- **Candidate source:** recorded after commit.
- **Changed paths:** `src/main.rs`, this evidence file, `docs/analyst-notebook-parity.md`.

## Change

Extended the existing source-scoped live lookup regression to use a qualified `LEFT JOIN` between observations and a source-context table. The query constructs composite source/target identity aliases, projects aliased observation fields and a joined context value, and includes a row with no joined context. It runs against two separate tenant-scoped SQLite databases that reuse the same local numeric IDs.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Focused joined-query regression | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline sql_source_record_lookup_keeps_source_scope_and_is_read_only -- --exact tests::sql_source_record_lookup_keeps_source_scope_and_is_read_only` exited 0: 1 passed. |
| Assertions | PASS | Aliased joined columns map correctly; the matched `LEFT JOIN` value is displayed, an unmatched join remains empty, source-scoped composite identities stay separate, typed parameters still bind, and BLOB/NULL displays remain bounded. |
| Read-only and immutable snapshot checks | PASS | Both SQLite files are byte-identical before/after lookup, and both serialized snapshots are unchanged. |
| Full offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 69 passed, 0 failed, 3 ignored (two local-corpus tests and the opt-in benchmark). |
| Locked compile check | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native inspector workflow | NOT TESTED | This test calls the lookup path directly; no native pointer/render acceptance is claimed. |

The fixture uses generated temporary databases and fictional rows. It does not read or modify the retained Clinton data. No IBM database connector compatibility is implied.
