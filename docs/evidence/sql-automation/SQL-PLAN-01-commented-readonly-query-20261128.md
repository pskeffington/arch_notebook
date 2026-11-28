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
| Locked offline release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0 in 2m50s. Release SHA-256: `52c2a22db4b6c762bb88103137bab7b8419bce8189d7765464c3727d4efd1339`. |
| Current-user launcher/icon install | PASS | `scripts/install-linux-user.sh /tmp/arch-notebook-native-install/arch-notebook-linux-x86_64` exited 0. Installed executable SHA-256 matches the release. Isolated installer acceptance exited 0: desktop menu and shortcut, SVG/PNG icons, MIME definition, and SQL automation helper all match package assets. Report: `/tmp/arch-notebook-native-install/acceptance.json`. |
| Native window creation | PASS, limited | `xwininfo` observed `Arch Notebook · Link Analysis` at 1440×900. The X11 connection then became unavailable to screenshot/input tools; no visual or pointer/keyboard acceptance is claimed. |
