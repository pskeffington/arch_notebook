# Codex next implementation instructions

Task: implement the vector chart improvement queue in [ROADMAP.md](ROADMAP.md), beginning with **UI-00**. The current request authorizes the roadmap and Git handoff; this document supplies instructions for the next implementation session. Implementation results are not claimed here.

## Start here

1. Read `AGENTS.md`, `README.md`, `ROADMAP.md`, `docs/vector-ui-acceptance.md`, `docs/notebook-workflows.md` and `docs/research/vector-ui-literature-matrix.md`.
2. Inspect the current branch, working tree, remote state and any active implementation PR. The planning baseline is `a1cca830de5324080e15aa06ec8f2dc710eba3c0`; refresh the source map if newer code has landed. Preserve uncommitted work and coordinate with the other seat before editing shared files.
3. Use one feature branch/worktree for the pass, for example `codex/ui-00-chart-baseline`. Continue an existing compatible branch where one is active. Do not force-push, discard another seat's changes or treat the roadmap commit as a runtime acceptance result.
4. Linux Operations claims UI-00 and owns `src/main.rs`/shared chart changes. Foundation / Mac Operations validates on macOS and owns parallel R1 evidence work. Transfer exact SHAs, fixture hashes and remaining checks before changing shared-code ownership.

## Immediate assignment — UI-00

Implement this bounded result first: a repeatable fictional chart workload, opt-in local measurement, meaningful chart invariants and an executing CI test gate. Then collect the baseline on an available native host. Use [the UI-00 card](ROADMAP.md#ui-00--measure-the-current-chart-and-establish-regression-protection) and [measurement protocol](docs/vector-ui-acceptance.md) for the exact deliverables and exit gate.

- Run and retain the four existing tests. Add targeted chart and fixture regressions; there is no established chart test suite yet.
- Add the deterministic generator and manifest described in UI-00. Keep large generated cases outside the repository and record their hashes.
- Measure chart preparation, picking and navigator work separately. Keep diagnostics opt-in, local and bounded; do not serialize counters into cases.
- Add and execute `cargo test` in CI after dependencies are fetched, preserving the five supported build/package targets.
- Record an unoptimized release baseline before changing drawing, hit behavior or layout. If a native host is unavailable, complete the source work and record the exact host handoff; do not fabricate timings.

Once UI-00's source and available-host gates pass, continue UI-01 → UI-02 → UI-03 in separate reviewable passes. Keep the Mac-specific acceptance open until actual Mac evidence arrives. UI-04 is conditional; UI-05 accepts the integrated core workflow. R1 can proceed in parallel.

## Source map for implementation

All symbols below exist in `src/main.rs` at the planning baseline. Prefer symbol search over fixed line numbers.

| Concern | Existing symbols / location | Instruction |
| --- | --- | --- |
| Persistence and identity | `Entity`, `Link`, `CaseFile`, `refreshed_link_index` | Preserve UUIDs, source-scoped connection identity, empty-ID records and normalized positions. |
| Chart camera | `chart_fit_transform`; chart branch in `App::update`; `chart_zoom`, `chart_pan` | Consolidate forward/inverse transforms without changing saved coordinate semantics. |
| Layout and locks | `LayoutMode`, `App::auto_layout`, `position_locked` | Existing modes are Connectivity, Hierarchy and Grid. Retain them. |
| Routes and picking | `link_lane_offsets`, `connection_curve_points`, `polyline_distance`, `segment_distance` | Share route geometry; test loops, coincident endpoints, parallel and reversed records. |
| Drawing | `App::update`, `ui.painter_at(rect)`, entity/edge loops and navigator | Clipping exists. Target unnecessary preparation and text work with measured CPU culling. |
| Derived graph state | `invalidate_network_groups`, `connected_entity_counts`, `source_entity_ids`, `network_groups`, `network_bridges` | Extend explicit invalidation; avoid full rebuilds on camera-only changes. |
| Search and provenance | `focus_entity_from_chart_search`, `focus_link_from_chart_search`, `highlight_sql_source_on_chart`, `show_saved_selection_set` | Keep chart/table/inspector destinations consistent with active filters and selected records. |
| Path/time | `shortest_path`, `timeline_event_positions`, `timeline_date_domain`, `parse_iso_date_ordinal` | Preserve direction, date-order constraints and exact parallel-record selection. |
| Tests | `#[cfg(test)] mod tests` | Four tests cover connection identity and legacy snapshot keys; build a real chart regression seam. |

### Geometry and cache constraints

The baseline maps normalized case coordinates `(x, y)` into a chart rectangle, then applies zoom about the rectangle center and screen-space pan. Define a tested chart transform around that existing convention. `Entity.x/y` must not silently become absolute world coordinates.

Links lack a persistent UUID. A bare vector index becomes stale after insert/delete/reorder/import/merge; a `connection_id` may be empty or reused in another source. Use revision-aware handles and reconcile selection using existing identity rules. If anonymous duplicate records cannot be safely rematched after mutation, clear the stale selection and report the change instead of selecting another record.

Reuse normalized geometry or other derived state where valid; screen routes, label placement and hit bounds depend on viewport size, zoom and display/font/style settings. Cache invalidation tests must cover those dependencies. Existing network-group caches are not a substitute for a chart cache contract.

## Checks and evidence

Run from the repository root. These are required implementation commands, not results from the roadmap-writing pass:

```sh
git status --short --branch
git rev-parse HEAD
rustc --version
cargo --version
cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline
```

If dependencies are absent, record the offline failure. Fetch the locked dependencies only in an available permitted environment, then rerun the same commands; do not change the lockfile to make a check pass. Keep the existing CI packaging paths consistent with its configured target directory rather than changing them incidentally.

For each pass, write `docs/evidence/vector-ui/<pass>-<seat>-<candidate-sha>.md`. Include exact input hashes, commands and outputs, baseline/candidate measurements, inspected screenshots/recordings where available, unresolved defects and next action. The evidence record must distinguish `PASS`, `FAIL`, `PENDING` and `NOT APPLICABLE` for each check. Update the roadmap status only as far as those results support.

Commit each coherent implementation pass and push the feature branch. Keep the change reviewable with a PR/handoff describing the user problem, resulting behavior, measured checks and limitations. Do not advance a failed prerequisite by relabeling it complete. Publishing a GitHub Release still requires the explicit approval described in `AGENTS.md` and `RELEASING.md`.

## Preserve through every pass

Read-only SQLite access, immutable captures, SQL-managed versus analyst-owned properties, imported identity, merged history, evidence references, atomic writes, selection sets, saved-path recomputation, existing layout choices, timeline/source filters and the current branding. Benchmark with fictional inputs; keep operational documents and source payloads out of public fixtures. Retain egui/eframe 0.31.1 and Glow for the initial comparison.
