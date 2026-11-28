# Commented read-only SQL query-plan acceptance — 2026-11-28

**Repository:** `arch_notebook`  
**Branch:** `codex/ui-00-chart-nested-conditions`

Query-plan inspection now normalizes the query's terminal statement separator with the same SQL-aware parser used by seeded result paging. This handles a semicolon followed by line or block comments without changing quoted semicolons or executing source rows. The regression uses a typed named parameter and verifies the SQLite file bytes remain unchanged.

| Gate | Result | Evidence |
| --- | --- | --- |
| Parameterized query with terminal semicolon and line comment | PASS | `sql_query_plan_accepts_parameterized_query_with_trailing_comment_read_only` returns SQLite plan steps for `SELECT ... WHERE id = :record_id; -- reviewed plan`. |
| Source database read-only boundary | PASS | The regression compares the SQLite file bytes before and after plan inspection. |
| Focused regression | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline sql_query_plan_accepts_parameterized_query_with_trailing_comment_read_only` exited 0 (1 passed). |
| Formatting, diff, locked check, full suite | PASS | `cargo fmt --all`, `git diff --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0; 90 passed, 0 failed, 3 ignored. |
| Release build and native interaction | NOT RUN | This acceptance is a focused query-plan regression and does not establish release installation or native UI behavior. |
