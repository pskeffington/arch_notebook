# SQL selected-entity seeded query review — 2026-11-28

**Repository:** `arch_notebook`  
**Branch:** `codex/ui-00-chart-nested-conditions`  
**Source baseline:** `c4a6e1e9aa17681d6370bb7ae6f86322740a1590`

SQL Review binds a selected chart entity's SQL external ID to a one-off named parameter (default `seed_entity_id`). Seeded validation fetches 200 usable rows per page through a read-only SQL wrapper, retaining the original query for display/provenance. Analysts can select rows across pages and add only those chart relationships. The action reuses entity IDs, applies SQL-owned properties, creates no snapshot or automation run, then invalidates the review. Saved job parameters remain unchanged. Queries should use deterministic `ORDER BY` clauses because pagination follows query row order.

| Gate | Result | Evidence |
| --- | --- | --- |
| Page wrapper, named parameter, ordering and boundary | PASS | In-memory SQLite regression covers 205 ordered rows, trailing semicolon/comment handling, named parameter binding, 200-row display boundary and next-page offset. |
| Selective insertion | PASS, headless state regression | Selects one of two rows, reuses an existing identity, adds one relationship, creates zero snapshots, and invalidates the review. |
| Focused SQL paging tests | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline seeded_query -- --nocapture` exited 0 (2 passed). |
| Full locked offline suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0: 86 passed, 0 failed, 3 ignored. |
| Formatting / diff / locked check | PASS | `cargo fmt --all -- --check`, `git diff --check`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Release build / launcher install | BLOCKED / NOT RUN | In this environment the release build cargo process exits without returning a result in the execution session; the existing binary is stale and was not installed. |
| Native interaction / IBM compatibility | NOT TESTED | Headless state and SQLite tests do not prove native pointer behavior or i2 Analyze integration. |

Page totals are intentionally not computed with a second count query; a 201st probe row signals another page. Stable ordering is the analyst's responsibility.
