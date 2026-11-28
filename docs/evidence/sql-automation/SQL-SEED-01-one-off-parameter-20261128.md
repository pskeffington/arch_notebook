# SQL selected-entity seeded query review — 2026-11-28

**Repository:** `arch_notebook`  
**Branch:** `codex/ui-00-chart-nested-conditions`  
**Source baseline:** `8628229b20644f35dbc2496e845620614d128cf5`

SQL Review binds a selected chart entity's SQL external ID to a one-off named parameter (default `seed_entity_id`). Seeded validation retains at most 200 query result rows for a transient checklist. The analyst can select rows and add only those chart relationships, reusing entity identities and applying SQL-owned source properties. The operation creates no snapshot or automation run; insertion invalidates the preview to avoid accidental repeat. Saved job parameters remain unchanged.

| Gate | Result | Evidence |
| --- | --- | --- |
| Seed binding and source filtering | PASS | Unit tests cover parameter replacement/validation and active-chart SQL identity filtering. |
| Selective result insertion | PASS, headless state regression | `seeded_query_review_adds_only_selected_rows_without_creating_a_snapshot` selects one of two rows, reuses the existing identity, adds one link, creates zero snapshots, and invalidates the result review. |
| Formatting and whitespace | PASS | Final `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Selective insert regression | PASS | `seeded_query_review_adds_only_selected_rows_without_creating_a_snapshot`: one of two rows is inserted, existing identity is reused, no snapshot is created, and review is invalidated. |
| Full locked offline suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0: 85 passed, 0 failed, 3 ignored. |
| Locked offline check | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Release build / launcher install | BLOCKED / NOT RUN | `cargo build --release --locked --offline` session stayed pending after the cargo process exited; existing release executable is stale and was not installed. |
| Native pointer interaction / IBM compatibility | NOT TESTED | Headless state coverage does not establish native pointer behavior or i2 Analyze integration. |

The 200-row cap bounds transient retained results; later rows are counted but not individually selectable. Narrow the query to reach later candidates. This is a local SQL workflow and does not provide IBM's server-configured external-search catalog or full result paging.
