# SQL selected-entity seeded query review — 2026-11-28

**Repository:** `arch_notebook`  
**Branch:** `codex/ui-00-chart-nested-conditions`  
**Source commit:** `04d7ee5d16f9f324b4e79f3432ddfbb36a4afc0e`

SQL Review binds a selected chart entity's SQL external ID to a one-off named parameter (default `seed_entity_id`). Seeded validation fetches 200 source rows per page through a read-only SQL wrapper, retaining the original query for display/provenance. Analysts can select rows across pages and add only those chart relationships. The review exposes up to 16 otherwise-unmapped SQL columns, capped at 256 bytes per cell and 4 KiB per row. These details stay transient and are omitted from case serialization. The action reuses entity IDs, applies SQL-owned properties, creates no snapshot or automation run, then invalidates the review. Saved job parameters remain unchanged. Queries should use deterministic `ORDER BY` clauses because pagination follows query row order.

| Gate | Result | Evidence |
| --- | --- | --- |
| Page wrapper, named parameter, ordering and boundary | PASS | In-memory SQLite regression covers 205 ordered rows, trailing semicolon/comment handling, named parameter binding, 200-row display boundary and next-page offset. A separate boundary assertion counts skipped blank-endpoint rows in the 201st-row probe. |
| Selective insertion | PASS, headless state regression | Selects one of two rows, reuses an existing identity, adds one relationship, creates zero snapshots, and invalidates the review. |
| Transient query-detail review | PASS, SQLite-backed regression | Unmapped investigator note and BLOB values appear in the preview, cell and row bounds hold, the `review_fields` payload is omitted from case JSON, and source database bytes remain unchanged. |
| Per-page search and visible-row selection | PASS, headless UI regression | Case-insensitive filter searches stable connection keys (including natural keys), IDs, endpoint names/types/representations, relationship, notes, properties, and transient query fields. “Select visible rows” adds only filtered rows while preserving prior selections across pages; regressions confirm filtered-out rows are excluded and the search term carries across page navigation when the query and parameters still match. Desktop-width headless render shows the search, selection, navigation, and add controls. |
| Focused SQL review tests | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline seeded_query_review` exited 0 (4 passed). |
| Full locked offline suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0: 89 passed, 0 failed, 3 ignored. |
| Formatting / diff / locked check | PASS | `cargo fmt --all`, `git diff --check`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Locked offline release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0 in 2m52s. Release SHA-256: `dbc6b600c0ef79a3e0876846b2a43fa314bf9c686826c6a02fc3800632c0003f`. |
| Per-user launcher install | PASS | `scripts/install-linux-user.sh /tmp/arch-notebook-current-install-query-fields/arch-notebook-linux-x86_64` exited 0. Installed executable at `/home/sysadmin/.local/bin/arch-notebook` matches the release hash. |
| Desktop/icon/MIME acceptance | PASS | `python3 scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-current-install-query-fields/arch-notebook-linux-x86_64/arch-notebook --report /tmp/arch-notebook-linux-user-install-query-fields.json` exited 0. Menu and desktop shortcut match; SVG/PNG icons, SQL helper, and MIME definition match package assets; desktop-file validation passed. |
| Native interaction / IBM compatibility | NOT TESTED | Headless state and SQLite tests do not prove native pointer behavior or i2 Analyze integration. |

Page totals are intentionally not computed with a second count query; a 201st probe row signals another page. Stable ordering is the analyst's responsibility.
