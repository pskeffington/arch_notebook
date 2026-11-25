# Vector UI measurement and acceptance protocol

Updated: 2026-11-25. Applies to [UI-00–UI-05](../ROADMAP.md). This is a prospective protocol; no measurements or completed user study are recorded here.

## Separate the evidence levels

| Level | Required evidence | What it establishes |
| --- | --- | --- |
| Source | Candidate diff, meaningful tests, formatter/type/build results | Implemented behavior and the invariants actually tested |
| CI | Candidate SHA and job URLs, including executed tests | Checks/package results on named runners |
| Native | Exact binary/SHA, host/display details, fixture hash, logs | Behavior on that native host |
| Visual and interaction | Inspected fictional screenshots/recordings and task observations | Observed layout, selection and navigation behavior |
| User evaluation | Actual participant/session information, tasks, errors and completion time | Only the usability conclusions supported by those sessions |

CI does not replace a native rendering run. A developer walkthrough is not a participant study. A reported successful TRANS workflow is retained as an observation; acceptance requires a traceable reproduction or an explicitly scoped observation record.

## Reproducible fixtures

Implement a deterministic generator, seed/configuration manifest and compact correctness fixtures during UI-00. Use fictional names and source references. Preserve duplicate records; do not deduplicate a fixture to improve timings. Hash each generated file and record actual `N` entities, `E` relationship records, direction distribution, component count, parallel-record count and self-loop count.

| Family | Sizes / bounded workload | Purpose |
| --- | --- | --- |
| Correctness | Empty case; one entity; two entities; explicit parallel/reversed/anonymous links; loops; coincident positions; long names; missing endpoint in a controlled error fixture | Deterministic geometry, identity, import-error and selection expectations |
| Sparse | N = 100, 1,000, 10,000; E = 3N | Representative scale progression with fixed edges per entity |
| Hub | Same N; E = 3N with a documented hub degree and remainder rule | High-degree neighborhoods and heavy local overlap |
| Disconnected | Same N; E = 2N, including isolated entities and several bounded components | Fit, navigator, component focus and layout behavior |
| Locally dense | Same N; E = 10N capped at 100,000, concentrated in documented clusters | Density stress without an unbounded all-pairs graph |

The above E values count relationship records in a multigraph. The generator must define direction, parallel/loop allocation and placement explicitly, emit the requested counts, and reject impossible configurations. Test 10,000-node cases as capacity probes; do not imply that every workload must achieve the interactive target. Record load failures, timeouts and memory limits as results. Generate large files outside Git; retain the generator, manifest, seed, hashes and small regression fixture in Git.

Include an offscreen-endpoint case whose curved edge crosses the viewport, a visible label or arrow on an otherwise offscreen route, a selected offscreen item, and two independent sources with the same `connection_id`. Include an insertion/deletion that shifts link indices and two anonymous identical links to expose false cache identity assumptions.

Use direct case JSON for deliberately anonymous duplicate links. Preserve current import rejection of ambiguous source-bearing CSV rows without distinct connection IDs; a rendering stress fixture is not authority to weaken ingestion identity rules.

## Measurement procedure

1. Record baseline and candidate SHAs, binary and fixture hashes, exact lockfile hash, Rust versions, OS/kernel, CPU, memory, GPU/driver, renderer, X11/Wayland or macOS session, window dimensions in logical points and physical pixels, display scale, refresh rate and power settings. Do not combine measurements from different machines as a paired comparison.
2. Build release binaries with the same profile, lockfile and diagnostic configuration. Retain the unchanged baseline binary separately. Declare the required workload subset before taking timings; begin with the small correctness fixtures and representative 1,000-node sparse/hub cases. Optimization passes rerun affected representative cases; reserve the complete host/capacity matrix for UI-05, using the preserved baseline for matched comparisons. Describe instrumentation overhead or compare a diagnostics-disabled run when the diagnostic cost could affect the conclusion.
3. Run a documented action sequence: load, Fit all, active pan, pointer-anchored zoom, drag an unlocked entity, try a locked entity, select parallel records, search/focus an item, navigate the overview and apply a layout. Separate load, idle, interaction and layout phases.
4. Proposed sampling procedure: warm up for 5 seconds, capture a 30-second interaction trace, and repeat at least three times per required configuration. Record actual durations/sample counts and repeat the same action trace. Native input automation is useful where available; a scripted manual walkthrough must be labeled manual.
5. Report per-run p50/p95/p99 for each measured span, plus memory and count summaries. Declare the percentile calculation; retain raw or adequately summarized local measurements. Do not pool incompatible workloads or conceal failed runs.
6. Compare correctness and performance on equivalent fixtures, camera states and display settings. If run variation obscures the effect, report that uncertainty and investigate before claiming an improvement. Stop broad reruns once the concrete risk and required gates are resolved.

### Metrics and limits

| Metric | Definition / reporting rule |
| --- | --- |
| Chart preparation | CPU time spent deriving visible geometry, labels and shapes; name the instrumented boundaries. |
| Whole update | CPU time for the application's update path; include sidebar/navigator work. Keep separate from chart-only time. |
| Frame interval/presentation | Measure with an appropriate native/backend method and document it. App update time alone does not establish displayed FPS. |
| Input-to-visible | Time from input event to observed corresponding display change; state how measured. Mark pending if unsupported. |
| Picking | Candidate-query and exact-distance work, result identity and latency for the same clicks. |
| Layout | Time/iterations, cancellation latency, overlap/crossings and displacement of persistent entities; keep separate from pan/zoom. |
| Work/memory | Total/visible/culled entities and links; labels; submitted shapes or meshes if observable; cache hits/rebuilds; process/cache memory. Distinguish submitted shapes from measured GPU draw calls. |
| Task result | Correct record/evidence reached, completion time and errors for actual observed sessions. |

**Provisional interaction objective:** p95 frame interval at or below 16.7 ms on a declared workload and reference host. This approximates a 60 Hz budget; it is not a verified capability or a promise for every fixture. Label a CPU-time comparison as CPU time if presentation cannot be measured. Do not call a failed objective passed by changing the workload after seeing the result; document any revised scope and its rationale.

UI-00 passes when the baseline is reproducible, even if it misses that objective. UI-02 requires a measured improvement for the targeted workload and no unexplained correctness or performance regression beyond observed variability. UI-05 publishes only the capacity and host claims supported by actual measurements.

## Correctness gates

### Coordinates, routes and picking

- Preserve the existing normalized chart-coordinate serialization. For positive viewport dimensions and supported zoom, screen → case → screen error should be at most 0.1 logical point in the test domain; document the coordinate range and any justified tolerance adjustment.
- Pointer-anchored zoom preserves the case position under the pointer. Pan, zoom and resize do not mutate persisted entity coordinates or evidence. Dragging changes only the intended unlocked entity coordinates; layout preserves every locked coordinate.
- Card hit regions match the rendered card; overlays take input precedence. Curved edges, self-loops, reversed directions and parallel records have finite bounds and deterministic record selection. Distinguish actual self-links from different entities at coincident positions.
- Culling never removes a primitive intersecting the viewport, including curved crossings with both endpoints outside, arrowheads, strokes and relevant labels. False-positive candidates are acceptable; false-negative visible objects are not.
- Cached and corrected UI-01 reference paths agree on visible record identities and picking outcomes for the correctness fixtures. Do not use the pre-fix loop/picking behavior as the parity oracle. Apply a documented geometric tolerance where sampled curves differ; record identity agreement is exact.
- Empty cases, degenerate sizes, maximum/minimum zoom, long text and actual HiDPI settings do not panic or lose selection. Preserve keyboard shortcuts, search focus, saved selection sets, timeline filters and path direction/date constraints.

### Invalidation contract

Complete this table with actual cache names and tests as UI-02 is implemented. Derived caches stay outside serialized cases.

| Change | Required affected state |
| --- | --- |
| Open/new case, topology replacement | All entity/link lookup, route, spatial, filter and selection-derived state; case generation changes. |
| Add/delete/import/SQL refresh/merge/reorder | Topology revision, entity maps, link handles/lane groups, neighborhoods, route bounds, source/path views; remap or clear selection. Never retain an index into the old vector. |
| Drag/auto-layout/accepted layout result | Position revision, incident routes and bounds, spatial candidates, navigator positions and label collisions; preserve locked coordinates. |
| Lock/unlock or other layout-constraint edits | Constraint revision and affected worker/preview validity; recheck current locks before applying any result. |
| Pan/zoom/viewport resize | Camera/projection, visible candidates, screen geometry and label layout as needed; normalized graph positions and topology remain unchanged. |
| Display scale/font/text/theme changes | Affected text measurements, styling, projected bounds and spatial/hit tolerances; no stale font/texture references. |
| Entity rename/type/attributes or link label/direction/confidence/date/source edits | Affected text/style/route/filter/search state and identity reconciliation; invalidate path results according to existing rules. |
| Source/group/path/time focus and search/selection changes | Visible decoration, priority labels, hit policy and inspector content as applicable; do not alter underlying graph membership. |
| Worker layout completion or cancellation | Apply only to matching case/topology/position/constraint revisions and current locks. Discard stale results/previews; restore preview-owned positions only under the same revision guard, preserving intervening user edits. |

### Data and workflow invariants

Run on copied fictional fixtures. Verify source SQLite file hashes before/after, immutable snapshot records, stable endpoint/source identities, relationship counts/multiplicity/direction, source references, analyst versus source notes and merged history. Verify save/reopen plus supported CSV/TSV round-trip. Compare canonical values when order is immaterial; document expected position changes from deliberate dragging/layout. Use existing documented CLI/GUI flows and report unsupported operations rather than inventing a harness result.

Existing native capture/comparison, schema/property drift and scheduled execution acceptance remain required for the integrated workflow. Record those results independently from frame timings. No external IBM i2 or native ANX claim follows from an internal round-trip.

## Native interaction tasks

Use identical fictional targets across baseline and candidate. Record observed completion, errors, time and any assistance. A scripted developer run verifies the flow; an actual usability study needs its own participant/session design.

| Task | Observable outcome |
| --- | --- |
| Locate an actor | Search/focus finds the intended UUID; keyboard focus and a readable textual label identify it. |
| Trace a directed/time-constrained path | Exact eligible relationship records are highlighted; excluded alternatives remain inspectable with the appropriate reason. |
| Distinguish parallel links and a loop | Select the intended record individually and inspect direction, date, confidence and source. |
| Retrieve underlying evidence | Reach the selected record's source reference and relevant source/SQL view without a wrong-record handoff. |
| Return to investigation context | Use existing Fit all, navigator, search focus or saved selection/path to restore the intended area; record exactly which mechanism was used. |

At low zoom, selected/focused details must remain readable in the inspector and retrievable on the chart. Label policies must preserve positions and expose suppressed-detail counts. Test visual focus and written status; confidence/direction must have cues beyond color. Record actual keyboard and screen-reader checks separately. Do not claim general accessibility conformance from screenshots.

Linux Operations records native Linux behavior and the display protocol actually tested. Foundation records Mac behavior, display scaling, Command-key shortcuts and application launch/package behavior. The available architecture is named in each report; evidence for one architecture or display protocol does not imply the others were tested.

## Pass report template

Create one report per pass/seat/candidate at `docs/evidence/vector-ui/<pass>-<seat>-<sha>.md` with the following fields. Paths named here are prospective deliverables.

```text
pass:
seat / accountable owner:
status: READY | IN PROGRESS | BLOCKED | REVIEW | ACCEPTED | DEFERRED
baseline_sha:
candidate_sha:
fixture_manifest / hashes / N / E:
binary_hash / lockfile_hash:
host / renderer / display / power settings:
changed_paths:
commands_and_observed_results:
checks: PASS | FAIL | PENDING | NOT APPLICABLE, each with reason/evidence
measurement_boundaries / warmup / samples / repeats / percentile_method:
baseline_metrics:
candidate_metrics:
visual_and_task_observations:
defects / unavailable_checks / owner / reentry_condition:
recovery_or_rollback:
acceptance_decision_and_reviewer:
next_ready_pass:
```

Keep logs/results in the repository when compact and fictional; otherwise provide durable authorized artifact references and hashes. Technical reports describe observations and commands. Leave unrun fields pending, preserve failed results, and never substitute a commit ID for execution evidence.
