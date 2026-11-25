# Vector chart implementation roadmap

Updated: 2026-11-25. Planning baseline: [`a1cca830de5324080e15aa06ec8f2dc710eba3c0`](https://github.com/pskeffington/arch_notebook/commit/a1cca830de5324080e15aa06ec8f2dc710eba3c0).

## Objective and next action

Make the relationship chart easier to navigate, select and read as cases grow, while preserving every entity, relationship record and evidence reference. Deliver measured improvements to the existing native Rust vector renderer and retain the current branding and investigation workflows.

**Next implementation pass: `UI-00` — establish reproducible chart measurements and regression checks. Linux Operations owns shared chart implementation; Foundation / Mac Operations owns Mac validation and independent review.** Start with [CODEX_NEXT.md](CODEX_NEXT.md); record results using the [acceptance protocol](docs/vector-ui-acceptance.md).

This commit defines work. Runtime implementation and measurements remain **NOT STARTED**. The [literature matrix](docs/research/vector-ui-literature-matrix.md) supplies candidate methods and records unresolved source checks; its performance numbers are proposed targets.

## Current implementation and evidence

The following findings come from the pinned source, not a performance run. Refresh them against the implementation branch before editing.

| Area | Observed implementation | Consequence for the next pass |
| --- | --- | --- |
| Renderer | `Cargo.lock` pins egui, eframe, epaint and egui_glow to 0.31.1; `Cargo.toml` enables Glow. | Keep this backend and lockfile for baseline comparison. Validate APIs against the pinned version. |
| Chart code | Chart rendering, interaction and other application logic reside in `src/main.rs`, principally `App::update`. | Extract small pure chart helpers as needed; assign one writer to shared source. |
| Camera | `chart_fit_transform`, pointer-anchored zoom, pan, Fit all and navigator already exist. `Entity.x/y` are normalized chart coordinates. | Preserve saved coordinate semantics and navigation; share one tested transform between drawing and picking. |
| Layout | `App::auto_layout` already provides connectivity, breadth-first hierarchy and alphabetical grid; locked positions are respected. | Retain existing modes. An optional force layout is a later experiment. |
| Visibility | The chart already uses `ui.painter_at(rect)` clipping, but prepares every entity, edge and label. | Measure CPU preparation, then add conservative culling before shape/text work. |
| Repeated work | `App::update` rebuilds lane offsets, entity maps and filter sets; click handling reconstructs curves. Navigator rendering visits the full graph. | Profile these separately; cache only with explicit invalidation. These are bottleneck candidates, not measured results. |
| Connections | `link_lane_offsets`, `connection_curve_points` and `polyline_distance` provide parallel lanes and picking. The same-start/end quadratic retraces a segment. | Preserve parallel records; add a regression and true self-loop geometry before optimizing routes. |
| Identity | Entities have UUIDs. `Link` has no UUID; `connection_id` can be empty, and `selected_link` is a vector index. `refreshed_link_index` scopes nonempty IDs by source. | Never key a cache by `connection_id` alone. Invalidate/remap link indices on structural changes. |
| Tests and CI | Four identity/snapshot unit tests exist. `.github/workflows/build.yml` runs formatting, check, build and packaging, but has no `cargo test` step. | Run existing tests, add meaningful chart regressions and make CI execute them. |
| Workflow observation | TRANS → Arch Notebook PDF/email processing and actor-link display were reported successful in the project conversation. | Preserve that workflow; use fictional equivalents for committed checks. The report does not establish a measured UI benchmark. |

Source authority: [application](https://github.com/pskeffington/arch_notebook/blob/a1cca830de5324080e15aa06ec8f2dc710eba3c0/src/main.rs), [lockfile](https://github.com/pskeffington/arch_notebook/blob/a1cca830de5324080e15aa06ec8f2dc710eba3c0/Cargo.lock), [workflow](https://github.com/pskeffington/arch_notebook/blob/a1cca830de5324080e15aa06ec8f2dc710eba3c0/.github/workflows/build.yml), and [notebook workflows](docs/notebook-workflows.md).

## Priority queue and ownership

One shared-code pass is active at a time. Foundation may validate fixtures, collect Mac evidence and verify references in parallel without editing the same chart code. Hand off exact candidate SHAs. No dates or completion percentages are assumed.

| Order / pass | Outcome | Accountable owner | Preconditions | Status |
| --- | --- | --- | --- | --- |
| P0 / `UI-00` | Repeatable baseline, fictional fixtures and an executing regression gate | Linux Operations | Current instructions and source inspected | READY |
| P1 / `UI-01` | Consistent camera, visible geometry and record selection; correct loops | Linux Operations | UI-00 source checks and at least one native baseline recorded | NOT STARTED |
| P1 / `UI-02` | Reduced chart preparation with conservative culling, caching and indexed picking | Linux Operations | UI-01 geometry and selection gates pass | NOT STARTED |
| P2 / `UI-03` | Readable labels, predictable focus and keyboard access | Linux Operations | UI-02 equivalent-output checks pass | NOT STARTED |
| P2 / `UI-05` | Integrated Linux and Mac acceptance for the core UI improvements | Linux Operations; Foundation owns the Mac evidence | UI-00–UI-03 complete; include UI-04 only if delivered | NOT STARTED |
| P3 / `UI-04` | Evidence-based decision on an optional force layout | Linux Operations | UI-03 accepted on available host; R1 source gate for the selected method | NOT STARTED |
| Parallel / `R1` | Version-specific API and method evidence sufficient for implementation | Foundation / Mac Operations | Existing literature matrix and pinned lockfile | READY |

Critical path: UI-00 → UI-01 → UI-02 → UI-03 → UI-05. UI-04 is optional and must not delay acceptance of the core improvements. Missing Mac hardware blocks Mac acceptance only; it does not block pure tests or Linux implementation. Preserve the existing native ingestion/scheduler acceptance queue and record its results alongside UI evidence. External IBM i2 acceptance remains a separate work item.

## Pass execution cards

### UI-00 — Measure the current chart and establish regression protection

**Purpose:** produce a reproducible comparison baseline before changing chart behavior.

**Steps:**

1. Record branch, SHA, dirty state, toolchain, lockfile hash and host/display details. Reconcile newer work before editing; retain the reported TRANS workflow as context and leave its source data out of fixtures.
2. Run the four existing tests. Add fixture-driven invariants for normalized coordinate conversion, pinned positions, parallel-link identity, path/source selection and import/export preservation. Extract only the pure helpers required for a test seam. Keep a geometry defect reproduction with its corrective patch; do not merge a deliberately failing gate.
3. Implement a deterministic fictional fixture generator at `scripts/generate-chart-fixtures.py`, using the existing case/import schema. Commit a compact manifest and small correctness fixture under `fixtures/chart/`; generate larger workloads in a temporary directory. The generator and paths are new deliverables, not existing commands.
4. Add local, opt-in timing/counters for chart preparation, hit testing, navigator work and cache/shape counts. Keep instrumentation bounded and outside persisted case data; write results locally. Do not equate CPU spans with displayed FPS.
5. Add `cargo test --locked --offline` to a compatible native CI job after locked dependency fetch. Retain all existing build/package targets and artifact paths. Run the tests locally and record the resulting candidate CI job that executes them; neither a YAML edit nor a local test run alone establishes the CI gate.
6. Capture the unoptimized release baseline with the workloads and procedure in the acceptance protocol. Foundation can collect the Mac baseline at the same SHA in parallel. Run the compact import → chart → evidence → save/reopen smoke exercise on an available native host.

**Deliverables:** fixture generator/manifest, executing tests, CI test step, diagnostic measurement path, and `docs/evidence/vector-ui/UI-00-<seat>-<sha>.md` with linked results. Proposed evidence directories are created by the implementing pass.

**Gate:** deterministic fixture hashes and counts; existing tests plus added invariants execute; native baseline reports measured values or an explicit capacity failure; source and SQL integrity checks pass. Missing 16.7 ms does not fail baseline collection. Mark a host result BLOCKED if the host or dependencies are unavailable.

**Recovery:** retain the baseline branch/artifacts; fix instrumentation overhead or fixture errors before comparing optimization results. No source-model or dependency migration belongs in this pass.

### UI-01 — Share chart geometry and correct selection

**Purpose:** ensure that what the user sees is what they can select.

**Steps:**

1. Introduce small chart modules such as `src/chart/transform.rs` and `src/chart/geometry.rs` as needed. Preserve normalized `Entity.x/y` serialization, current zoom limits and established fit/navigator behavior. Name units and coordinate spaces in comments.
2. Use the same geometry description for painting, hit testing, bounds and arrow placement. Preserve confidence dash patterns and source/path/time highlighting. Keep pan/zoom transient; layout and deliberate node dragging remain position edits.
3. Reproduce the self-link retracing behavior, then give loops actual nondegenerate routes and pickable bounds. Distinguish a self-link by endpoint identity from two different entities at coincident positions. Manual equal-endpoint creation is currently disabled, so cover imported records and direct test fixtures. Maintain distinct lanes for same-direction, reversed-direction and duplicate relationship records.
4. Resolve hit priority explicitly: navigator/timeline controls first; inside a visible entity card, select that card; outside cards, choose the nearest eligible visible relationship with deterministic ties. Replace radial card approximations with the actual card bounds plus documented tolerance. Keep overlapping records reachable from Connections.
5. Test extreme supported zoom, pointer-anchored zoom, resize, fit, drag, locked nodes and empty/single-node cases. Record visual observations on the native host.

**Deliverables:** shared geometry helpers and focused regressions; updated selection behavior in `docs/notebook-workflows.md`; UI-01 evidence.

**Gate:** transform round trips meet the protocol tolerance; loops have finite nonzero extent and pick the correct record; painting and picking agree; existing saved cases and navigation retain their meaning. No topology or provenance changes occur.

**Recovery:** preserve the previous renderer while extracting helpers, then switch one behavior at a time. Revert the affected implementation commit on the feature branch if correctness cannot be demonstrated; retain the failing fixture and defect record.

### UI-02 — Reduce unnecessary rendering and picking work

**Purpose:** improve responsiveness without changing chart meaning.

**Steps:**

1. Profile the baseline candidates and implement one measured improvement per commit. Separate graph topology, positions, label/style/filter state and camera changes; cache only derived state. Retain existing network-group invalidation for actual graph edits; it also clears path/group state and must not be called merely because the camera changed.
2. Cache entity lookup and lane assignments by case/topology revision. Build reusable route bounds and a spatial candidate index where measurements justify it. Use a graph-revision-plus-index handle for anonymous/duplicate links unless a separately reviewed identity design is required; never merge records to simplify caching.
3. Cull before constructing shapes or laying out text. Bounds must include curved paths, loops, strokes, arrowheads and relevant labels. Keep an edge crossing the viewport even when both endpoints are outside. Maintain painter clipping as the final guard.
4. Reuse route geometry for hit testing; query nearby candidates rather than rebuilding every route on a click. Batch compatible paths only where the pinned API and measurements support it. Preserve dashed confidence encodings.
5. Measure navigator and sidebar work independently so a full-graph redraw or ranking pass cannot hide the chart improvement. Cache or simplify the navigator's derived overview while keeping selection and viewport feedback current.
6. Exercise the full invalidation table in the acceptance protocol. Compare cached/culling results with the corrected UI-01 reference path on the small correctness fixtures, including selected records outside the viewport. Do not preserve pre-fix loop or picking defects in the parity reference.

**Deliverables:** measured caching/culling/index changes, invalidation tests, before/after results, and UI-02 evidence.

**Gate:** visible and picked record identities agree with the reference; every invalidation scenario passes; the targeted workload improves and other required workloads show no unexplained regression beyond measured run variability. Ordinary pan/zoom does not rebuild topology or trigger layout. Document bounded memory use and stale-cache prevention.

**Recovery:** disable or revert the individual optimization if parity fails; keep the correct reference path for diagnosis until parity and native inspection pass.

### UI-03 — Make dense views readable and navigation predictable

**Purpose:** reduce clutter while keeping the selected evidence accessible.

**Steps:**

1. Add explicit overview, neighborhood and detail label policies driven by projected size/space. Keep coordinates unchanged when crossing detail thresholds; add hysteresis or stable ranking to avoid flicker.
2. Prioritize selected, keyboard-focused and searched items, then path and neighborhood context. Use collision-aware label placement and compact entities at overview scale. Offscreen selections remain named in the inspector and reachable through the existing focus action.
3. Show visible/total counts and a clear indication of suppressed detail. Level of detail may change decoration, never delete relationships, alter path results or obscure source availability. Preserve the current meaning of source/group/time dimming.
4. Preserve workspace shortcuts and search-to-chart behavior; provide visible keyboard focus, a written selection/status description and a usable textual route through Entities/Connections to evidence. Do not rely on color alone for direction, confidence or focus. Test with long labels and display scaling.

**Deliverables:** label/focus policy, relevant interaction tests, native screenshots from fictional fixtures, updated workflow guide, and UI-03 evidence.

**Gate:** task checks in the acceptance protocol succeed; selected details remain readable through the inspector; zoom changes neither positions nor identities; labels do not flicker around a threshold; keyboard navigation can reach the required record and evidence. Record screen-reader behavior separately where actual assistive technology is tested.

**Recovery:** expose a full-detail fallback and preserve the previous label policy until the new policy passes the task checks. Log any unresolved overlap or access defect.

### UI-04 — Evaluate an optional layout using verified method details

**Purpose:** determine whether a new arrangement improves the user's actual tasks.

**Steps:**

1. Complete R1 for the selected method. Compare an optional force layout with the existing three layouts; retain exact repulsion as the small-fixture reference before considering Barnes–Hut approximation.
2. Implement deterministic settings, iteration/time bounds, pinned-node constraints and cancellation. Use a worker if measurement shows main-thread blocking; apply its result only while case, topology, position and constraint revisions still match. Recheck current locks before applying; locking or unlocking a node invalidates an older result even if coordinates are unchanged.
3. Adopt a snapshot/preview/apply contract and retain prior positions for an explicit guarded revert. Cancellation discards the preview. Restore preview-owned positions only under the same revision guard; preserve intervening user edits. Opening a case, panning and zooming must not start an automatic layout. Preserve the persisted normalized coordinate range and locked coordinates.
4. Compare overlap, edge crossings, displacement, layout time and the same user tasks. Record precision/approximation settings and counterexamples; name an implementation ForceAtlas2 only when the verified algorithm is actually followed.

**Deliverables:** experiment and method notes, measured comparison, and a retain/defer decision in UI-04 evidence. Shipping a new algorithm is not required to close the decision.

**Gate:** all locked coordinates, graph records and evidence survive; cancellation and stale-result rejection work; the proposed layout improves an identified task or metric without an unresolved correctness regression. Otherwise mark the feature DEFERRED and retain the existing layouts.

**Recovery:** discard the preview; restore preview-owned positions only when their revision is still current, preserving intervening drag/import/lock edits. Any future case-format change requires a separate migration and compatibility pass.

### UI-05 — Accept the integrated chart on Linux and Mac

**Purpose:** establish the actual supported workflow and workload on native hosts.

**Steps:**

1. Freeze a candidate SHA and fixture manifest. Both seats run the same core workload and correctness checks; record platform-specific DPI, input, window-resize and packaging behavior.
2. Execute import → graph → individual relationship selection → source inspection → save/reopen → export/reimport. Exercise source/path/time filters, navigator, all existing layouts and locked positions. Re-run the retained SQL capture/comparison and scheduler acceptance checks using fictional sources.
3. Compare baseline and candidate release measurements, then observe the five user tasks. Separate developer walkthroughs from any actual user study; retain exact defects and host limitations.
4. Update `README.md`, `docs/notebook-workflows.md` and the acceptance record only to the evidence level achieved. Check the candidate's own CI; historical green builds do not cover it.

**Deliverables:** Linux and Foundation reports, before/after measurements, fictional screenshots or recordings with captions, defect resolutions and an acceptance decision linked to the candidate SHA.

**Gate:** required source checks pass; both native reports satisfy the core correctness and interaction checks; performance claims state the measured workload and hardware. External IBM i2 validation and release/notarization remain separately tracked. This pass does not publish a release.

**Recovery:** retain the prior working binary/case copy and candidate evidence; fix failures on the implementation branch and rerun only affected checks plus the final workflow smoke.

### R1 — Close the evidence needed for algorithm implementation

Foundation owns this parallel pass. Resolve egui 0.31.1 APIs from matching documentation or pinned crate source; distinguish clipping from CPU culling. Reconcile V01–V08 with exact source sections/pages, publication versions and method limitations using the existing crosswalk skill. Record unavailable locators as pending with reasons.

Prioritize V08 for UI-00–UI-02, V04 for semantic-zoom rationale, and V01/V02 only before adopting their algorithm. V03/V05 dense matrix views and V07 tile/routing architecture remain candidates. Basic instrumentation and pure geometry tests can proceed while scholarly checks remain open. Deliver updated matrix rows and linked implementation decisions; do not certify a paper or claim an empirical replication from source review.

## Scope and decision register

| Decision | Rationale / re-entry condition | Owner |
| --- | --- | --- |
| Retain egui/eframe 0.31.1 and Glow initially | Comparable baseline; propose a backend/dependency change only when a measured limitation remains after bounded fixes. | Linux Operations |
| Correct loops and picking before caching | Cached geometry must represent the record the user is selecting. | Linux Operations |
| Preserve normalized case positions and current branding | Compatibility with saved/imported cases and existing UI work. | Linux Operations |
| Keep research verification parallel | It informs later method selection; it need not stall basic correctness or measurement. | Foundation |
| Defer tile pyramids, a new SVG renderer, dense adjacency view and wholesale application rewrite | Require a demonstrated task/capacity limit and a separate evidence-backed proposal. | Linux Operations |
| Retain local source and evidence boundaries | Read-only SQL; immutable snapshots; preserved IDs, merged history and atomic writes; fictional public fixtures. | Both seats |

## Status updates

Use `READY`, `NOT STARTED`, `IN PROGRESS`, `BLOCKED`, `REVIEW`, `ACCEPTED` or `DEFERRED`. Keep source checks, native execution, visual inspection and task acceptance distinct. A commit alone does not accept an implementation pass.

Each pass report must include baseline/candidate SHAs, changed paths, exact commands and outcomes, fixture hashes, observed behavior, unresolved defects, recovery action and the next ready pass. If a check is unavailable, write `PENDING` or `BLOCKED` with the prerequisite and owner. Update this queue in the same implementation PR or handoff; do not open a second shared-code pass while its prerequisite is unresolved.
