# SQL selected-entity seed parameter helper — 2026-11-28

**Repository:** `arch_notebook`  
**Branch:** `codex/ui-00-chart-nested-conditions`  
**Baseline:** `aaee4db63c31542551aac7517b2e0db91da96164`

SQL Review now lets an analyst bind a selected chart entity's external ID from an SQL source on the active chart to a named one-off query parameter. It defaults to `seed_entity_id`, and changes only the current SQL Review parameter draft. Saved automation job parameters are not edited. The analyst still writes the SQL predicate and must validate the query and explicitly choose any later ingest.

| Gate | Result | Evidence |
| --- | --- | --- |
| Formatting and whitespace | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Locked offline Rust suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0: 84 passed, 0 failed, 3 ignored. The two new tests cover parameter replacement/validation and active-chart SQL identity filtering. |
| Locked offline check | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Release build | BLOCKED | `cargo build --release --locked --offline` did not return a cargo result in the execution session; the process disappeared while the session remained pending. The existing release executable predates this change and was not installed. |
| Installed launcher / visual interaction | NOT RUN | No fresh release binary was available to install; tests exercise helpers but not native pointer interaction. |
| IBM External Search compatibility | NOT TESTED | This is a local SQL parameter helper, not an i2 Analyze integration or selective result browser. |

The picker filters identities to SQL snapshot sources linked to the active chart and distinguishes IDs from the same source. The query result preview and existing snapshot ingest behavior remain unchanged; selective result-by-result chart insertion is still missing.
