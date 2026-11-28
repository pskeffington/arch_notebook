# SQL multi-entity seeded query review — 2026-11-28

**Repository:** `arch_notebook`  
**Branch:** `codex/ui-00-chart-nested-conditions`

The SQL Review workspace can bind multiple selected chart entities to one one-off SQLite parameter. It offers only a shared source for which each selected entity has exactly one SQL identity; sources with a missing or ambiguous identity are omitted. IDs are sorted, deduplicated, JSON-encoded, and bound as parameter text, so query authors can use SQLite `json_each(:seed_entity_ids)` without interpolating IDs into SQL. Binding also switches the SQL Review SQLite path to the source database that owns those IDs, preventing a mismatch with the current query path. The saved automation job and schedule parameters remain unchanged. This is a local read-only SQL workflow; it does not implement the i2 Analyze connector or administrator query catalog.

| Gate | Result | Evidence |
| --- | --- | --- |
| Shared-source intersection and ambiguous-ID exclusion | PASS | Regression checks two selected entities with one common source and confirms a duplicate source identity makes that source unavailable. |
| JSON parameter binding and SQLite expansion | PASS | Regression parses the one-off parameter and reads its values via bundled SQLite `json_each`; values are sorted/deduplicated and empty identities are rejected. |
| Seeded SQL Review end-to-end | PASS, SQLite-backed | The actual seeded validation path returns only the two IDs supplied by the JSON parameter, adds no snapshot, and leaves source database bytes unchanged. |
| SQL Review UI event | PASS, headless | The test renders the data workspace with two selected entities and a deliberately mismatched SQL path, clicks **Bind selected entity IDs**, and verifies the JSON parameter, seeded-review mode, and SQLite path switch to the source that owns the IDs. It then repeats the mismatch with one selected entity and verifies the single-ID control switches the path and binds the scalar ID. |
| Focused multi-seed tests | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline sql_multi_seed` and `... sql_review_ui_binds_selected_entities_and_matches_their_sql_source_path` exited 0 (4 passed across both commands). |
| Formatting, diff, locked check, full suite | PASS | `cargo fmt --all`, `git diff --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0; 94 passed, 0 failed, 3 ignored. |
| Locked offline release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0 in 2m54s. Release SHA-256: `8d1c1087cdbe126785ba6ca57721a3f2f704ab08e537d9dcd0a0b11d4b739c04`. |
| Current-user launcher and graphics | PASS | Installer and isolated acceptance exited 0. Installed executable matches the release; desktop entry, shortcut, SVG/PNG, MIME definition, and SQL helper match package assets. Report: `/tmp/arch-notebook-native-install/multi-seed-source-path-final-acceptance.json`. |
| IBM interoperability and native interaction | NOT TESTED | No IBM schema/application or live external search service was involved. Headless UI event coverage does not establish native pointer interaction. |
