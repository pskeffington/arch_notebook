# CHART-COORDINATE-REBASE-01 — explicit chart coordinate normalization

## Handoff

- **Task / owner:** Add a safe, analyst-invoked repair for anomalous saved chart coordinates; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `4b8d1cd59d46605f2717256089589d722f26dcde`.
- **Candidate product source Git blob:** `src/main.rs` `0313cf6f951678557ec14a7f8c5bfdfe77e3be17`.
- **Installed release binary:** SHA-256 `6ec6622ef41c14675a1e702185cc8b8548eaaad1eae860cffd063370c37a70a0`.
- **Data boundary:** The retained Clinton case was read-only in QA and was not changed. Evidence includes aggregate counts only.

## Change

When active-chart entity positions fall outside normalized `[0,1]` coordinates, **View… → Normalize chart coordinates** appears with the count of affected entities. Invoking it maps entity positions and chart-label positions through the entity-derived coordinate frame into normalized coordinates and stores explicit `[0,0,1,1]` bounds. For valid current bounds, the screen mapping is preserved; a regression verifies positions at non-default zoom and pan. The action changes the in-memory case only; the analyst must save to persist it. SQL snapshots and source databases are not written.

The current Clinton case has 2,793 out-of-range entity positions, mainly on the y-axis. The SQL QA uses a read-only case and does not invoke this action. A later analyst can open a writable copy, inspect the result, then choose Save if the normalized positions are desired.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Screen-preserving normalization | PASS | `rebase_chart_coordinates_preserves_the_displayed_layout_and_normalizes_labels` verifies every entity and label remains at the same screen position with existing zoom/pan, all coordinates normalize, and a second run is a no-op. |
| Analyst menu reachability | PASS, headless | `headless_view_menu_offers_normalization_for_out_of_range_chart_coordinates` finds and activates the View menu action and verifies normalized saved-state coordinates plus the save-to-keep status. |
| Full Rust suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target CARGO_BUILD_JOBS=1 cargo test --locked --offline` exited 0: 112 passed, 0 failed, 3 ignored. |
| Formatting, locked check, release build | PASS | `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target CARGO_BUILD_JOBS=1 cargo check --locked --offline`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target CARGO_BUILD_JOBS=1 cargo build --release --locked --offline` exited 0. Release SHA-256: `6ec6622ef41c14675a1e702185cc8b8548eaaad1eae860cffd063370c37a70a0`. |
| Launcher and install acceptance | PASS | Isolated install acceptance reported PASS; executable, menu/Desktop launchers, SVG/PNG artwork, MIME definition, and SQL helper matched the package. Aggregate report: [installer JSON](artifacts/CHART-COORDINATE-REBASE-01-install-acceptance.json), SHA-256 `b92db8366457ece01555632a5f885696ec2a8824afa645c0df323da509cfd98a`. |
| Clinton SQL primary regression on installed release | PASS | [CLINTON-CORPUS-QA-07](../sql-automation/CLINTON-CORPUS-QA-07-normalized-coordinates.md): query-only preflight and saved-run replay pass on 6,173 rows, zero skipped rows or deltas, with input hashes unchanged. The outlier count stays 2,793 because the case was not modified. |
| Native visual and pointer acceptance | NOT TESTED | The current session has no usable display/input path; the action was tested in egui headless interaction only. |
| IBM Analyst's Notebook visual parity | NOT TESTED | No IBM application was available. |

## Next action

Open the installed app on a writable copy of the Clinton case, use **View… → Normalize chart coordinates**, inspect the chart, and save only if the displayed arrangement is acceptable. Record native screenshot and pointer results separately; the existing read-only QA case and SQL source must remain unchanged.
