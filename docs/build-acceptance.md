# Local build acceptance record

## Portable saved SQL search packages — 2026-11-28

Saved SQL search definitions can now be exported/imported as version-1 JSON packages with `--export-sql-search` and `--import-sql-search`. The package is definition-only; it carries no chart/case data or SQL results, omits bound seed IDs, rebases relative paths, and leaves imported SQL unexecuted. Export protects case files, automation locks, and referenced SQLite sources; imports reject duplicate names and bound chart-seed values.

| Gate | Result | Evidence |
| --- | --- | --- |
| Package construction, path rebasing, stale-seed and source-overwrite unit regressions | PASS | Two focused Rust tests pass. |
| Full Rust suite | PASS | 99 passed, 0 failed, 3 ignored. |
| Locked offline workspace check and formatting | PASS | Both exited 0. |
| Optimized locked offline release build | PASS | `TMPDIR=/home/sysadmin/.cache/arch-notebook-build-tmp CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline`, exit 0. |
| CLI saved-search package acceptance | PASS | Round-trip/override, new ID, duplicate-name no-write, bound-seed rejection, SQLite source preservation, no query execution, unchanged snapshot count. |
| Isolated installer acceptance and current-user install | PASS | Launcher/menu/shortcut, icons, MIME definition and SQL helper pass; installed binary SHA-256 `5872cb68ebebca61961397b566fab9944ca7f958210e5a1a5b8d8e29fc6cf135` matches the candidate. |
| Primary Clinton corpus QA | PASS | One query-only preflight passed; replayed 6,173 rows with zero skipped rows or deltas; source inputs unchanged. |
| Native UI review and IBM interoperability | NOT RUN | Native pointer/display review unavailable; this is not an i2 Analyze connector. |

Exact commands, aggregate report hashes, and scope: [SQL-SEARCH-PACKAGE-01](evidence/sql-automation/SQL-SEARCH-PACKAGE-01-current-source-20261128.md).

## SQL search package workflow in SQL Review — 2026-11-28

The SQL Review catalog now imports and exports the same definition-only packages as the CLI. Import loads a draft without query execution or snapshot creation; optional overrides adapt the package name, source database path, and series. The case must still be explicitly saved to retain the imported search.

| Gate | Result | Evidence |
| --- | --- | --- |
| Formatting, workspace check, and full Rust suite | PASS | 100 passed, 0 failed, 3 ignored; locked offline check and formatting exited 0. |
| Optimized release build | PASS | Locked offline workspace release build exited 0. |
| CLI package acceptance and isolated installer | PASS | Package safety checks passed; installed binary, launchers, icons, MIME definition, and helper synchronized. |
| Clinton corpus SQL regression | PASS, bounded | 6,173 relationship rows, zero skipped rows and deltas, inputs unchanged. Extraction completeness, Qwen analysis, and human content review remain unestablished. |
| Native UI appearance/input and IBM interoperability | NOT RUN | Headless checks do not establish native display acceptance; no IBM schema/application validation was performed. |

Exact commands, binary digest, and aggregate evidence: [SQL-SEARCH-PACKAGE-02](evidence/sql-automation/SQL-SEARCH-PACKAGE-02-desktop-ui-20261128.md).

## Seeded saved search to automation draft — 2026-11-28

SQL Review now provides an explicit path from a chart-seeded saved search to an unseeded automation draft. It removes only the bound ID parameter value, preserves the SQL predicate and other typed defaults, clears stale validation state, and requires the analyst to review/edit and revalidate before a disabled job can be created.

| Gate | Result | Evidence |
| --- | --- | --- |
| Formatting, workspace check, full Rust suite | PASS | 101 passed, 0 failed, 3 ignored. |
| Optimized release build and current-user installation | PASS | Release contains the new control/warning; installed executable matches the optimized build. |
| Saved-search package, installer, Clinton regression | PASS | Package and install checks pass; Clinton replay returned 6,173 rows, zero skipped and zero deltas, with unchanged inputs. |
| Native UI, live systemd dispatch, IBM interoperability | NOT RUN | Current sandbox cannot access desktop input or user systemd; IBM schema/application validation remains unavailable. |

Commands, binary digest, and report hashes: [SQL-SEARCH-AUTOMATION-DRAFT-01](evidence/sql-automation/SQL-SEARCH-AUTOMATION-DRAFT-01-20261128.md).

## Case-local saved SQL searches — 2026-11-28

**Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-nested-conditions` / `8037cbc2d0a94444aebecb9937710e34e52006de`.

SQL Review now stores named query definitions in the active case: database path, series, SQL text, and typed parameter defaults. Loading restores only the draft; it does not execute SQL or create a snapshot. Chart seed values are excluded and the parameter name is retained for rebinding. Definitions are local SQLite drafts, not IBM i2 Analyze External Searches or scheduled automation jobs. Existing case files default to an empty catalog on load.

| Gate | Result | Evidence / limit |
| --- | --- | --- |
| Saved search serialization, legacy case compatibility, seed omission | PASS | Focused regressions cover defaults, round-trip, legacy fixture, and dynamic seed rebinding. |
| SQL Review headless save/load/update/delete | PASS | UI regression verifies all catalog operations without creating snapshots or executing SQL. |
| Rust suite at candidate source | PASS | 96 passed, 0 failed, 3 ignored; locked offline full suite completed before the last documentation-only edits. |
| Formatting, diff validation, locked offline workspace check | PASS | All exited 0. |
| Locked offline release build / install / Clinton corpus rerun | BLOCKED | `/tmp` had 789 MiB free at first attempt. After clearing Cargo incremental artifacts (1.4 GiB free), the release build process exited without replacing the existing binary (hash remained `b9049120…f0fa80cd`). No runtime acceptance is claimed for this candidate. |
| Native display and IBM application interoperability | NOT RUN | Native display access is unavailable; no i2 Analyze connection is implemented. |

Exact source-level commands and acceptance limits: [SQL-SEARCH-CATALOG-01](evidence/sql-automation/SQL-SEARCH-CATALOG-01-20261128.md).

## Saved SQL search Quick Open — 2026-11-28

Named case-local SQL searches now appear in Quick Open and can be opened by typing their name. Selecting one opens SQL Review and restores its path, series, query, and parameters without query execution or snapshot creation. The `quick_open_loads_named_sql_search_without_running_it` headless interaction regression passes. After this follow-up, the full Rust suite passes (97 passed, 0 failed, 3 ignored), and locked offline workspace check, formatting, and diff validation pass. Full-suite, release-binary, installed-runtime, and Clinton rerun limits remain recorded in [SQL-SEARCH-CATALOG-01](evidence/sql-automation/SQL-SEARCH-CATALOG-01-20261128.md); this follow-up changes source after that release attempt and has not been installed.

## Current-source SQL search follow-up runtime checks — 2026-11-28

The current source built in the locked offline debug profile and passed installer and Clinton checks. A later locked offline **release** build also completed successfully and passed those same checks. Release binary SHA-256: `cf9948f467a6f620b17381724984fa131f82dbb8c2f09ad684f8b33b76a90aaf`. Clinton replay: one query-only preflight passed; 6,173 relationship rows replayed; zero skipped rows or deltas; source inputs unchanged. The release was installed to the current user's `~/.local/bin`, menu launcher, Desktop shortcut, icon theme, and MIME paths. Installed binary digest matches the release artifact; menu and Desktop launchers match each other; SVG/PNG icons and MIME definition match repository assets. Aggregate-only release reports and hashes are in [SQL-SEARCH-CATALOG-03](evidence/sql-automation/SQL-SEARCH-CATALOG-03-current-source-release-20261128.md). The earlier debug-only check remains documented in [SQL-SEARCH-CATALOG-02](evidence/sql-automation/SQL-SEARCH-CATALOG-02-current-source-runtime-20261128.md).

## Proximity-map encoding legend — 2026-11-28

The sampled topology map now explains its node colors and size, proxy ring, direction arrow, selected/hovered tie styling, and relationship-count line thickness with a compact wrapped legend. Headless production-panel checks pass on the synthetic UI and retained Clinton case (2,813 entities, 6,173 records, 48 sampled groups, 47 sampled contacts). The full Rust suite passes (94 passed, 0 failed, 3 ignored); the locked offline release built and was installed with synchronized launcher and icon assets (SHA-256 `b9049120…f0fa80cd`). Native screenshot/pointer review remains blocked because this session cannot access the X11 or Wayland display. Details: [PROXIMITY-MAP-LEGEND-01](evidence/vector-ui/PROXIMITY-MAP-LEGEND-01-20261128.md).

## Clinton primary corpus QA rerun — 2026-11-28

The installed Rust release passes the primary Clinton SQL batch again: one saved job passed query-only preflight; replay returned 6,173 relationship rows with zero skipped rows and zero added/changed/removed deltas. The default-redacted topology packet contains 2,813 entities and 6,173 relationship records, with all entity centers rendered and no labels outside the SVG viewport. The PDF, case, and SQLite input hashes remained unchanged. The run does not establish PDF extraction completeness, Qwen analysis, or human content review. Full aggregate evidence: [CLINTON-CORPUS-QA-resume](evidence/sql-automation/CLINTON-CORPUS-QA-resume-20261128.md).

## Proximity-map hover layering — 2026-11-28

**Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-nested-conditions` / `2b6cb00b59aaaf60ea05603314ab8c346e23f34c`.

Hovered group-contact routes now paint after base routes and before selected routes and group nodes. This keeps a hovered tie visible where routes cross while leaving group fills, proxy rings, and node interaction on top. Route hover suppression uses the same square hit rectangle as the group click target. A distinct hover color lets the headless regression prove the full 12-segment route is painted before node fills.

| Gate | Result | Evidence |
| --- | --- | --- |
| Hover paint order and route hit test | PASS | `headless_ui_drives_geodesic_and_detected_community_actions`: exactly 12 distinct hover-color segments paint before group nodes; clicking the line still selects the group interaction and exact-record drilldown remains available. |
| Dense topology rendering | PASS | 48 groups / 576 contact pairs; viewport culling retains 156 candidates, skips 420, and paints 1,872 base-route segments. |
| Clinton production Analysis-pane UI | PASS | Retained corpus has 2,813 entities / 6,173 relationship records; analysis shows 48 sampled groups, 47 pairs and 1,315 represented records; selected-group contact detail is reached by scrolling the panel. |
| Full offline suite | PASS | 78 passed, 0 failed, 3 ignored. |
| Formatting / diff | PASS | `cargo fmt --all` and `git diff --check` exited 0. |
| Independent topology source review | PASS | Reviewer confirmed hover geometry matches the node hit rectangle, distinct hover strokes are asserted, and node/proxy rendering remains above the route. |
| Native visual / pointer review | BLOCKED | No accessible native display/input path in this session; headless rendering does not prove visual contrast or pointer feel. |
| Exact-head CI | PASS | [Run 46111875129](https://github.com/pskeffington/arch_notebook/actions/runs/46111875129) passed all six jobs for source `36de1a1302d1775331e2a765e28b7003b3b850d8`: Linux tests/fixtures, Linux x86_64/ARM64, Arch package, macOS Intel/Apple Silicon. |

Full command/evidence notes: [PROXIMITY-MAP-HOVER-ORDER-01](evidence/vector-ui/PROXIMITY-MAP-HOVER-ORDER-01-20261128.md).

## Dense sampled topology map validation — 2026-11-27

**Repository / branch / baseline:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `2feca0710b564e28be72bc5f129e8223c2c8adf6`.

A deterministic fictional stress case creates 48 distinct hop groups and 576 valid cross-group contact pairs, then renders the production Analysis panel at 1440×1600. The emitted route shapes confirm a 291×193 map clip, which is asserted against the exhaustive culling model. The convex-hull culler retained 156 route candidates and skipped 420 before sampling. The UI emitted exactly 12 route segments per retained contact (1,872 segments), preserving every route whose sampled expanded bounds could touch the viewport. The synthetic fixture complements the retained Clinton corpus, which remains the primary real-corpus QA batch.

| Gate | Result | Evidence |
| --- | --- | --- |
| Dense production UI route rendering | PASS, headless | `dense_proximity_map_culls_before_sampling_without_losing_visible_routes`: 48 groups / 576 pairs / measured 291×193 viewport / 156 candidates / 420 culled / 1,872 painted segments. |
| Culling conservatism | PASS | Exhaustive assertion across all 576 generated routes verifies no sampled route whose expanded bounds intersect the viewport is rejected by control-hull culling. |
| Clinton corpus UI | PASS, headless | Retained case: 2,813 entities / 6,173 records / 48 sampled groups / 47 sampled pairs; details in [viewport culling evidence](evidence/vector-ui/PROXIMITY-MAP-VIEWPORT-CULL-01-linux.md). |
| Full suite / check / build / format | PASS | 73 passed, 0 failed, 3 ignored; locked offline check and debug build, formatting and diff checks pass. |
| Topology source review | PASS | Reviewer confirmed control-hull bounds, crossing retention, arrow/picking margin, and node click priority. |
| Native visual / CI | NOT RUN / PENDING | Native display review remains unavailable; this non-main branch does not trigger the configured build workflow. |

Exact command and fixture construction: [PROXIMITY-MAP-DENSE-STRESS-01](evidence/vector-ui/PROXIMITY-MAP-DENSE-STRESS-01-linux.md).

## Sampled topology viewport culling — 2026-11-27

**Repository / branch / baseline:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `8af57a12649dadd5478d08289caf2cdf6de2060b`.

The sampled contact map now culls against each quadratic route's convex-hull bounds before sampling, using the scroll area's clip rectangle and a 10-point margin for stroke, arrow, and picking reach. Routes crossing the viewport remain available even when both endpoints are outside. Route geometry is no longer stored in a frame-sized vector; on a click, the renderer tracks only the nearest visible route candidate, preserving node-click priority. A headless regression checks crossing, offscreen, non-finite, and near-edge routes.

| Gate | Result | Evidence |
| --- | --- | --- |
| Route culling regression | PASS | `proximity_map_route_culling_keeps_crossings_and_pickable_edges` passed; existing production UI route-picking regression remains in the full suite. |
| Full offline Rust suite | PASS | 72 passed, 0 failed, 3 ignored. |
| Locked offline check / debug build / formatting | PASS | All exited 0; `git diff --check` also passed. |
| Clinton real-corpus production UI | PASS, headless | 2,813 entities / 6,173 links; 48 groups / 47 contact pairs / 1,315 represented records. |
| Matched topology frame shape count | PASS, observed | With the topology panel, clipped shapes decreased from 88,792 on the immediately preceding product renderer (`89ddc99`) to 88,204; the without-topology control remained 87,855. Frame-time samples were noisy; no latency or GPU speedup is claimed. |
| Topology source review | PASS | Reviewer confirmed the quadratic control-hull bound, crossing preservation, nearest-visible-route picking, and node-click priority. Culling happens before route sampling. |
| Native visual / GPU review | NOT RUN | Headless evidence only. |
| Exact-head CI | PENDING | Workflow does not run on this non-main branch push; no PR was opened. |

Commands and full benchmark output: [PROXIMITY-MAP-VIEWPORT-CULL-01](evidence/vector-ui/PROXIMITY-MAP-VIEWPORT-CULL-01-linux.md).

## Main chart topology route rendering — 2026-11-27

**Repository / branch / baseline:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `0763a00dfa893d3b0e86bf27ee15e6844a23cb82`.

Solid confidence-3 chart relationships now paint as a native quadratic Bézier shape instead of 16 individual segment strokes. The control point is recovered from the shared route midpoint, so drawing follows the same quadratic used by route sampling, culling, picking, and arrowheads. Sampled route points now use a fixed 17-point array instead of a heap `Vec`; dotted, dashed, self-loop, and coincident-endpoint routes retain their existing segment rendering.

| Gate | Result | Evidence |
| --- | --- | --- |
| Focused route regressions | PASS | `connection_curve_has_finite_endpoints_and_bends_for_parallel_lanes` and `chart_link_route_culling_preserves_crossings_and_nearby_decorations` passed. |
| Full offline Rust suite | PASS | 71 passed, 0 failed, 3 ignored. |
| Locked offline check and debug build | PASS | Both exited 0. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` passed. |
| Clinton real-corpus production UI benchmark | PASS, headless | 2,813 entities / 6,173 records loaded; 48 groups / 47 sampled pairs / 1,315 represented records. The paired frame benchmark passed. It does not measure the solid-link Bézier path because the retained corpus ties use confidence 1; no renderer speedup is claimed from these timings. |
| Topology source review | PASS | Read-only review confirmed the midpoint-derived control point matches the sampled quadratic; native visual and GPU validation were not performed. |
| Exact-head CI | PENDING | The workflow runs on `main` pushes or pull requests, not this non-main branch push; no run is attached to this candidate commit. |

Exact command output and limits: [PROXIMITY-MAP-ROUTE-PAINT-01](evidence/vector-ui/PROXIMITY-MAP-ROUTE-PAINT-01-linux.md).

## Topology selection feedback — 2026-11-27

The sampled map now draws the selected group with a clear outer ring and the selected sampled contact pair with a contrasting wider stroke. Full-index selections outside the bounded sample are labeled accurately while exact-contact records remain available. The 70-test suite and retained Clinton topology UI/sampler pass, including omitted group/pair selections. Native pointer/contrast review remains blocked; exact-head CI is pending. See [PROXIMITY-MAP-SELECTION-FEEDBACK-01](evidence/vector-ui/PROXIMITY-MAP-SELECTION-FEEDBACK-01.md).

## Topology tie routing — 2026-11-27

Sampled contact ties now follow deterministic curved routes; reciprocal direction arrows are offset and tangent-aligned, foreground nodes mask route ends, and pointer picking follows the painted segments. Per-frame route geometry is reused between painting and picking. An independent topology spot-check passes. The 71-test suite, retained Clinton topology UI/sampling checks, locked offline check/debug build, and format checks pass. Exact-head CI [46082650685](https://github.com/pskeffington/arch_notebook/actions/runs/46082650685) passed all six Linux, Arch, and macOS jobs for `58efaac`; the benchmark/evidence commit `6da8b2e` also passed all six in [run 46084425580](https://github.com/pskeffington/arch_notebook/actions/runs/46084425580). Paired Clinton headless CPU measurements show about 0.7% median frame-build and 0.8% median egui-tessellation overhead from the sampled panel at 48 groups / 47 pairs; build p95 varies from 226.6 ms without to 272.1 ms with the panel. Native review and the 1,128-pair ceiling remain unverified. See [PROXIMITY-MAP-EDGE-ROUTING-01](evidence/vector-ui/PROXIMITY-MAP-EDGE-ROUTING-01.md).

## Topology pointer-target geometry — 2026-11-27

Named map-geometry helpers size group nodes and their proxy-ring hit targets, place rows at a measured pitch, and include the last target in the scroll content. The sampled map now reports group, pair, and record coverage plus omitted totals. Production UI regression checks geometry, calculated coverage, and navigation; retained Clinton production UI and sampling checks pass. Exact-head CI [46076464039](https://github.com/pskeffington/arch_notebook/actions/runs/46076464039) passed all six jobs for `25ff6b6`. Native review remains open. See [PROXIMITY-MAP-HIT-GEOMETRY-01](evidence/vector-ui/PROXIMITY-MAP-HIT-GEOMETRY-01.md).

## Sampled topology row ordering — 2026-11-27

The bounded hop-group sample now orders rows by neighboring group contacts using deterministic barycenter sweeps. A synthetic fixture decreases crossings from six to zero; on the sampled Clinton graph, proper segment crossings decrease from 23 to 3. The Clinton real-corpus headless UI and sample bounds tests pass. The read-only Clinton acceptance also passes: 2,813 entities, 6,173 relationships, query-only SQL preflight 1/1, replay with zero skipped rows/deltas, and default-redacted packet with all entity centers in view. Human review, PDF extraction completeness, and Qwen analysis remain unverified. Exact-head CI [46078780061](https://github.com/pskeffington/arch_notebook/actions/runs/46078780061) passed all six jobs for `155dde0`. See [PROXIMITY-MAP-CROSSING-REDUCTION-01](evidence/vector-ui/PROXIMITY-MAP-CROSSING-REDUCTION-01.md).

## Topology rendering cleanup — 2026-11-27

The hop-group contact map now spaces columns and rows further apart, scales and labels group nodes with member totals, and separates overlapping contacts with deterministic lanes. Production headless UI and both retained Clinton topology regressions pass. Native display review remains blocked by unavailable X11/Wayland access; IBM visual parity is not tested. See [PROXIMITY-MAP-RENDER-CLEANUP-01](evidence/vector-ui/PROXIMITY-MAP-RENDER-CLEANUP-01.md).

## Current topology renderer verification — 2026-11-27

## Selected topology route layering — 2026-11-27

**Repository / branch / baseline / product source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `76d4ac5ada34b610b0cfba1c3f9ff94a4ad26872` / `ef304472a3fc21b297b20c306a41281824228501`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Selected route and direction markers above crossing routes | PASS, headless UI and independent code review | Regression requires all 12 selected strokes and at least six reciprocal arrow strokes after the selected highlight; nodes remain above both. |
| Focused topology UI and full offline suite | PASS | Focused test: 1 passed. Full suite: 73 passed, 0 failed, 3 ignored. |
| Formatting, diff validation, locked offline check/debug build | PASS | All four checks exited 0. |
| Locked offline release build on this host | PASS | A retry of `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` completed; the candidate was installed and verified in [LOCAL-INSTALL-09](evidence/deployment/LOCAL-INSTALL-09-linux.md). |
| Exact-head cross-platform CI | PASS | Manual run [46095125130](https://github.com/pskeffington/arch_notebook/actions/runs/46095125130) passed all six jobs for `57a714e7fe1c909e1eeb10b66283262c140fbcb3`: Linux tests, Linux x86_64/ARM64, macOS Intel/Apple Silicon, and Arch package. |
| Native visual acceptance | NOT RUN | No screenshot or native pointer review for this candidate. |

See [PROXIMITY-MAP-LAYER-01](evidence/vector-ui/PROXIMITY-MAP-LAYER-01-linux.md).

## Current-user Linux deployment — 2026-11-27

**Repository / branch / source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `57a714e7fe1c909e1eeb10b66283262c140fbcb3`.
**Installed release binary SHA-256:** `b3456f71520b084234b281a9056cea74b94aa1d9da7527ac3de4a125d79f574a`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Locked offline release build and user installer | PASS | Release build and `scripts/install-linux-user.sh` exited 0. |
| Installed binary and launcher | PASS | Installed binary matches the release SHA; menu and Desktop entries match and validate. |
| Icon, MIME, and automation assets | PASS | Both installed icons, case MIME XML, and automation installer match repository assets. |
| Exact-head CI | PASS | [Run 46095125130](https://github.com/pskeffington/arch_notebook/actions/runs/46095125130) passed all six jobs for this source. |
| Native visual interaction | NOT TESTED | X11 access is unavailable in this session; install synchronization does not establish visual acceptance. |

See [LOCAL-INSTALL-09](evidence/deployment/LOCAL-INSTALL-09-linux.md).

### Current bounded group topology candidate

**Repository / branch / baseline:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `ff106a669320c1e5e2325a39bd407da736a15ef5`.
**Candidate binary SHA-256:** `a84eb32ae53a51f1d17bdab7e8d905d280318aa8105dd50fc8c91a6f1ffa7bb9`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Full offline Rust suite | PASS | 69 passed, 0 failed, 2 ignored. |
| Retained Clinton topology sampling regression | PASS | Bounded 48-group sample spans all 10 hop columns; full expanded-contact index validated against all analyzed groups. |
| Clinton SQL replay and redacted packet | PASS | 6,173 records, 0 skipped, 0 deltas; all 2,813 entities rendered, zero labels outside SVG viewport; read-only input hashes unchanged. |
| Production topology UI on real Clinton corpus | PASS, headless | At 1440×900, the production Analysis pane renders the 48-group sample across 10 hop columns and navigates to a selected group with all group rows indexed; focused test passes. |
| Debug build, formatting, diff validation | PASS | Locked offline build, `cargo fmt --all -- --check`, and `git diff --check` passed. |
| Release build | NOT VERIFIED | Offline release compilation stopped progressing in the application crate and was interrupted; no release result is claimed. |
| Native Clinton graph review / human content review | BLOCKED / NOT RUN | X11 and Wayland access are denied in this session; human semantic review remains outstanding. See [PROXIMITY-MAP-CLINTON-01](evidence/vector-ui/PROXIMITY-MAP-CLINTON-01-linux.md) and [PROXIMITY-MAP-CLINTON-UI-01](evidence/vector-ui/PROXIMITY-MAP-CLINTON-UI-01-linux.md). |
| Exact-head CI for product implementation | PASS | Workflow run [46072141894](https://github.com/pskeffington/arch_notebook/actions/runs/46072141894) for product source `8e796bc8794e4865a3ec509703ae7ca3d8b581f4` passed Linux tests, Linux x86_64, Linux ARM64, Arch package, macOS Intel, and macOS Apple Silicon. The additional opt-in corpus UI regression and its evidence/docs are a later follow-up and are not covered by that run. |
| Exact-head CI for corpus UI harness and evidence | PASS | Workflow run [46073375126](https://github.com/pskeffington/arch_notebook/actions/runs/46073375126) for source `d3ee05fda2277e76ed9a5cf66cb2e7b9408ee2c3` passed Linux tests, Linux x86_64, Linux ARM64, Arch package, macOS Intel, and macOS Apple Silicon. CI compiles the added opt-in regression but cannot execute it because the retained Clinton case is local-only; its local focused run passed separately. |

See [PROXIMITY-MAP-CLINTON-01](evidence/vector-ui/PROXIMITY-MAP-CLINTON-01-linux.md) for exact commands and topology-specific assertions.

## ANX external-shaped import coverage — 2026-11-27

**Repository / branch / baseline:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `1370e2172473dffbd1f4923d424bda6909292a25`.
**Candidate source:** recorded after commit.

| Gate | Result | Evidence |
| --- | --- | --- |
| Supported external-shaped UTF-8 import | PASS, internal subset | `ArrowOnTail` endpoint reversal, direction, LinkType name, Strong confidence, date, and custom relationship attribute are verified on the actual import CLI. |
| Existing ANX safeguards and round-trip | PASS | Arch Notebook field round-trip, unsupported-shape rejection, DTD/external-entity rejection, and failed-import atomicity remain in the same passing harness. |
| IBM XSD and Analyst's Notebook validation | NOT TESTED | No IBM schema or application fixture was available; the added XML is synthetic. |

See [ANX-ROUNDTRIP-04](evidence/anx/ANX-ROUNDTRIP-04-external-shaped-fixture-linux.md).

## ANX vendor schema discovery — 2026-11-27

**Repository / branch / research baseline:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `2fd13d71ef38d37f6b7294834d9a9554ed470548`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Authoritative schema/example location research | PASS | IBM Support identifies the installed Windows `i2 Shared\XML\Schema` and `XML\Examples` locations. Its 2018 note names an ANB 7.x schema and covers older 8.9/9.0 versions, so it is not assumed valid for newer installations. |
| Local vendor schema or chart fixture availability | BLOCKED | Searches of repository fixtures/docs, `/home/sysadmin/Research.Databases`, and the user's home found no `.xsd` or `.anx` files. |
| IBM XSD/application validation | NOT TESTED | Requires a schema and representative chart from the matching installed IBM product version. Keep vendor artifacts local unless redistribution is permitted. |

Exact Windows locations, version caveat, and the validation handoff: [Analyst's Notebook compatibility map](analyst-notebook-parity.md#interoperability-gate); primary reference: [IBM Support ANX format note](https://www.ibm.com/support/pages/node/546669).

**Repository / branch / source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `a9f56cced2ec12c2f0f787520942939a7603b006`.
**Candidate:** current-source Linux debug application, SHA-256 `74c900167f450e28f391506d043bfb17fe177382afe19d6f8fdd16bcab24e606`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Offline Rust suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0; 69 passed, 0 failed, 1 opt-in benchmark ignored. |
| Current-source Linux debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-current-target cargo build --locked --offline` exited 0. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native initial render on synthetic chart | PASS, render-only | Current candidate rendered all 30 entity cards inside the chart viewport with dense Auto relationship text suppressed; [G4-DENSE-LABEL-03](evidence/vector-ui/G4-DENSE-LABEL-03-current-topology-render-20261127.md). |
| Clinton corpus SQL and redacted graph packet on current-source binary | PASS | Read-only preflight and latest-run replay pass for 6,173 rows with zero skipped rows/deltas; packet renders all 2,813 entities with default redaction and no out-of-viewport labels. Inputs remained unchanged; [CLINTON-CORPUS-QA-02](evidence/sql-automation/CLINTON-CORPUS-QA-02-current-debug-20261127.md). |
| Clinton PDF text-layer coverage and semantic Qwen review | NOT ESTABLISHED / BLOCKED | `pdftotext` returned nonempty text for all 7,121 pages, but header fields on 4,658 pages do not establish unique email extraction or semantic completeness. A later local aggregate-only Qwen structural review ran; it did not inspect message content or establish extraction completeness. See [CLINTON-CORPUS-EXTRACT-01](evidence/sql-automation/CLINTON-CORPUS-EXTRACT-01-text-layer-coverage-20261127.md) and [CLINTON-QWEN-STRUCTURAL-01](evidence/sql-automation/CLINTON-QWEN-STRUCTURAL-01-20261128.md). |
| Current-source ANX round-trip and safe rejection | PASS, Arch Notebook subset | Current debug binary round-tripped the fictional 8-entity/8-relationship fixture and rejected unsupported-shape and DTD inputs without changing the case. IBM XSD/application compatibility remains NOT TESTED; [ANX-ROUNDTRIP-03](evidence/anx/ANX-ROUNDTRIP-03-current-debug-20261127.md). |
| Native interaction, Clinton visual review, renderer performance, IBM application parity | NOT TESTED | No native input/performance measurements, real corpus capture, or IBM application/schema validation in this run. |
| Exact-head cross-platform CI | PASS | Manual workflow run [46059559135](https://github.com/pskeffington/arch_notebook/actions/runs/46059559135) completed successfully for `ab6e29318fca406d748f6694c89b3e25cd06b347`: Linux tests/SQL/ANX/installer, Linux x86_64, Linux ARM64, Arch package, macOS Intel, and macOS Apple Silicon. |
| Hop-group contact direction rendering | PASS, headless UI | Reciprocal directed group contacts render opposing arrowheads; undirected ties remain unarrowed. Full Rust suite and debug build pass; exact-head CI [46062451089](https://github.com/pskeffington/arch_notebook/actions/runs/46062451089) passes all six jobs. Native visual review remains open; [PROXIMITY-MAP-DIRECTION-01](evidence/vector-ui/PROXIMITY-MAP-DIRECTION-01-linux.md). |
| Hop-group map legend and node metrics | PASS, headless UI and exact-head CI | Legend explains node-size and relationship-count line-width encodings; node label distinguishes internal records, and hover discloses exact group metrics. The exact candidate passed all six CI jobs in [run 46064081779](https://github.com/pskeffington/arch_notebook/actions/runs/46064081779); native visual review remains open; [PROXIMITY-MAP-LEGEND-01](evidence/vector-ui/PROXIMITY-MAP-LEGEND-01-linux.md). |
| Hop-group map row spacing | PASS, measured geometry and headless UI | 52 px row pitch is checked against egui's actual label galley height and maximum node radius, with at least 4 px required clearance. Full suite (69 passed, 1 ignored), warning-free locked debug build, formatting, and diff checks pass; native visual review remains open; latest source CI awaits exact-head results; [PROXIMITY-MAP-SPACING-01](evidence/vector-ui/PROXIMITY-MAP-SPACING-01-linux.md). |
| Hop-group contact-line preview, selection, and record navigation | PASS, headless UI | Hover previews exact direction, pair, and record counts in a reserved status line; clicking selects the nearest interaction, opens exact-record detail, and highlights the tie. Group-node click-to-focus remains intact. Full suite/build/format checks pass. Native pointer/contrast review and latest exact-head CI remain open; [PROXIMITY-MAP-PICK-01](evidence/vector-ui/PROXIMITY-MAP-PICK-01-linux.md). |
| Topology metric label grammar | PASS, headless UI | Singular and plural entity, record, endpoint-pair, group, and hop-layer counts are rendered with matching nouns in map labels and summaries. The focused production UI regression verifies singular group and interaction labels. Full suite: 69 passed, 0 failed, 1 ignored; debug build and formatting/diff checks pass. Exact-head CI and native visual review are pending. |

The native screenshot resolves the left-edge clipping observed in the prior candidate, for this synthetic startup view only. It does not close G3/G4 native interaction or broader UI-00 acceptance.

## Analysis pane filter readability — 2026-11-27

**Task / owner:** Improve chart-analysis navigation at the default pane width; implementation session.
**Repository / branch / baseline / product source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `487473d0ff655e0b9fc0584fcd8dd59a7e320dbb` / `6dd04e4bfd16e3bb14eafa7b7634474ed3a51322`.
**Candidate binary SHA-256:** `766229457ca4de9975ace1a044bdde1e8802b6bc9ccd4f5bb60bdf904a83274f`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Unicode-safe long label handling | **PASS** | `analysis_filter_label_truncation_is_unicode_safe_and_stable` covers short values, exact 20-character limit, and Unicode character boundaries. |
| Default-width analysis-pane control layout | **PASS, headless UI** | `headless_analysis_filter_controls_stack_and_fit_default_pane` verifies separate rows, width-scaled clipping, and full-value hover disclosure at 1440×900/default 320px pane width and 1000×650/minimum 240px pane width, including wide Unicode labels. |
| Full Rust suite | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 60 passed, 0 failed. |
| Locked compile and builds | **PASS** | `cargo check --locked --offline`, debug build, and release build exited 0 with `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target`. |
| Formatting and diff validation | **PASS** | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Per-user launcher sync | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-sync-ui-filters/arch-notebook-linux-x86_64` exited 0; installed executable matches the release SHA above, desktop entries validate and match, and installed icon and helper files match repository assets. |
| Exact-candidate native process/window creation | **PASS, scoped** | The exact launch command remained live; `xwininfo -root -tree` listed an Arch Notebook X11 window at 1440×900. |
| Exact-candidate native visual review | **BLOCKED** | `spectacle --new-instance --activewindow --background --nonotify --no-decoration --output /tmp/arch-notebook-analysis-filters-6dd04e4.png` aborted with exit 134. No current-candidate screenshot or visual review is claimed. Host input injection also remains unavailable. |
| GitHub Actions for product changes | **PASS, ancestor candidate** | Manual workflow run [45934212852](https://github.com/pskeffington/arch_notebook/actions/runs/45934212852) completed successfully for `61fb5ebbe731761b9f53e29c0a3f8c8429e18b9e`, which includes the UI filter product changes. It predates the benchmark test/script and this evidence-only update; those newer changes are not covered by that run. |

Detailed source semantics, headless assertions, host limitation, and residual gates: [UI-ANALYSIS-FILTERS-01](evidence/vector-ui/UI-ANALYSIS-FILTERS-01-linux.md).

## Headless egui CPU baseline — 2026-11-27

The eight-case 100/1,000-entity synthetic workload benchmark completed successfully. It records full-app frame-build and egui CPU tessellation separately; the normal offline Rust suite reports 60 passed, 0 failed, and one opt-in benchmark ignored. The matched route-culling A/B shows 20.9–45.1% lower frame-build medians and 29.8–77.3% lower CPU tessellation medians on offscreen-panned synthetic charts; neutral-view shape counts are unchanged. GPU/presentation results remain unmeasured. See [headless CPU baseline](evidence/vector-ui/G4-EGUI-TESSELLATION-01-linux.md), [matched culling evidence](evidence/vector-ui/G4-CULLING-AB-01-linux.md), and [raw paired results](evidence/vector-ui/artifacts/G4-culling-ab-20261126.json).

## Analysis packet CLI output privacy — 2026-11-27

The JSON and human-readable success summaries no longer echo the caller-supplied packet output directory. The full offline suite passed (61 passed, 0 failed, 1 opt-in benchmark ignored), the locked release build passed, and the packet acceptance harness passed redaction, selected-chart, source immutability, SVG, failure-safety, permission, and stdout path-omission checks. Exact-head cross-platform CI passed in [run 45970460460](https://github.com/pskeffington/arch_notebook/actions/runs/45970460460). See [UI-ANALYSIS-PACKET-OUTPUT-PRIVACY-01](evidence/vector-ui/UI-ANALYSIS-PACKET-OUTPUT-PRIVACY-01-linux.md) and its [acceptance report](evidence/vector-ui/artifacts/UI-ANALYSIS-PACKET-OUTPUT-PRIVACY-01-linux.json).

## Conditional formatting in chart templates — 2026-11-27

Chart templates now carry up to 96 conditional attribute-formatting rules and apply them through bounded, visible template-owned legend blocks. The full suite (63 passed, 1 opt-in benchmark ignored), locked check/debug/release builds, and current-user launcher/icon sync pass. The complete template-ownership follow-up passed exact-head cross-platform CI in [run 45988117371](https://github.com/pskeffington/arch_notebook/actions/runs/45988117371). The prior native capture showed left-edge clipping; a current-source synthetic startup render shows all 30 cards inside the viewport, while template-generated legend interaction and the real Clinton chart view remain untested. See [CHART-TEMPLATE-CONDITIONAL-RULES-01](evidence/vector-ui/CHART-TEMPLATE-CONDITIONAL-RULES-01-linux.md), [prior G4 render](evidence/vector-ui/G4-DENSE-LABEL-02-native-render-20261127.md), and [current G4 render](evidence/vector-ui/G4-DENSE-LABEL-03-current-topology-render-20261127.md).

## Current-user Linux Desktop shortcut — 2026-11-27

The Linux per-user installer now creates an executable Desktop shortcut byte-identical to the application-menu entry. An isolated temporary-HOME smoke passes for executable, desktop files, icons, SQL helper and MIME definition; the current-user release and icon assets are synchronized. Exact-head cross-platform CI passed in [run 45988117371](https://github.com/pskeffington/arch_notebook/actions/runs/45988117371), including the installer smoke and current application candidate. A later current-source debug candidate launched and rendered a synthetic chart, but native interaction and launcher-to-current-source synchronization remain unverified. See [LOCAL-INSTALL-08](evidence/deployment/LOCAL-INSTALL-08-linux.md) and [current topology render](evidence/vector-ui/G4-DENSE-LABEL-03-current-topology-render-20261127.md).


## SQL automation and scale recheck — 2026-11-27

**Task / owner:** Revalidate the installed release's SQL automation, recovery, interchange, and joined-header scale paths; implementation session.
**Repository / branch / verification source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `282f9de3a755557c856b16f50145d9f37936e882`.
**Runtime source / binary:** product behavior from `e27ad5e842dd1c6d503f05c51844f12790502fa6`; installed release SHA-256 `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.

| Gate | Result | Evidence |
| --- | --- | --- |
| SQL automation, failure retention, diagnosis/repair, batch recovery, CSV/TSV interchange | PASS | Current installed release harness exited 0; expected failure statuses were retained and verified. |
| Joined-header scale | PASS | 7,121 synthetic records, 1,500 entities, 4,121 repeated endpoint pairs, two snapshots, zero skipped rows; source remained unchanged during captures. |
| Exact-source workflow status | NOT RUN | Actions API returned zero runs and commit status had no contexts for `282f9de`. |
| Production schema, native Automation UI, live scheduler, IBM validation | NOT TESTED | These require separate production-shaped inputs, native host interaction, scheduler execution, or IBM schema/application evidence. |

Commands, fixture descriptions, result hashes, and scope limits: [SQL-AUTO-07](evidence/sql-automation/SQL-AUTO-07-current-installed-release-linux.md).

## Joined-query SQL source drillback regression — 2026-11-27

**Task / owner:** Extend read-only live source-record lookup acceptance to joined SQL queries; implementation session.
**Repository / branch / baseline:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `17ed57eb14f50b7277285766aeecd7d1e24999cc`.
**Candidate source:** recorded after commit.

| Gate | Result | Evidence |
| --- | --- | --- |
| Joined lookup focused regression | PASS | Qualified `LEFT JOIN`, composite source identities, aliased outputs, present/missing join values, typed params, and display conversion all pass. |
| Source and snapshot immutability | PASS | The two fixture SQLite files and serialized snapshots remain unchanged after both lookups. |
| Full offline suite | PASS | 69 passed, 0 failed, 3 ignored. |
| Locked compile, format, diff checks | PASS | `cargo check --locked --offline`, `cargo fmt --all -- --check`, and `git diff --check` passed. |
| Native inspector review | NOT TESTED | Lookup is directly exercised at source level; native interaction remains a separate UI gate. |

See [SQL-DRILLBACK-02](evidence/sql-automation/SQL-DRILLBACK-02-joined-query-linux.md).

## SQL scheduler runner recheck — 2026-11-27

**Verification source:** `179aff68d32e59e49f411a18f17e90144903712d`; installed runtime binary SHA-256 `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Generated Linux systemd service invocation | PASS, stubbed | Five cadences and scheduled capture/CSV invocation passed with a temporary `systemctl` stub. |
| Generated macOS LaunchAgent runner | PASS, stubbed on Linux | Five cadences, rolling UTC window, removal and rollback passed with a temporary `launchctl` stub; native macOS host untouched. |
| Live systemd/launchd delivery | NOT TESTED | Stub execution does not establish live scheduler dispatch. |

Exact commands, fixture boundaries, report hashes, and limits: [SQL-SCHED-07](evidence/sql-automation/SQL-SCHED-07-current-installed-release-linux.md).

## Current synchronized branch verification — 2026-11-26

**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source commit:** `d0ce0c77e9fcee445536d0f55aceebf70e5fbd84` (adds a headless viewport paint-submission regression to the `e27ad5e` product candidate).
**Release binary SHA-256:** `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Offline Rust suite, check and formatting | PASS | 57 tests pass, including `headless_chart_paint_culls_offscreen_relationship_labels`; `cargo check --locked --offline`, formatting, and diff validation exit 0. |
| Debug and release builds | PASS | Locked offline builds exit 0. |
| Remote CI for exact source commit | NOT RUN | GitHub check-runs endpoint for `e27ad5e` returned zero checks. The paint-submission test commit `d0ce0c7` has not yet been checked remotely. Local acceptance is recorded separately. |
| Current-user desktop/launcher install | PASS, scoped | Installed binary matches the release artifact. Menu entry and Desktop shortcut validate and compare byte-for-byte; MIME registration, automation helper, SVG and PNG match. See [LOCAL-INSTALL-04](evidence/deployment/LOCAL-INSTALL-04-linux.md). |
| Native window interaction and visual review | NOT RUN | `wayland-info` returns `failed to create display: Operation not permitted`; install checks do not establish a rendered or interactive window. |
| IBM ANX/XSD/application validation | NOT TESTED | The IBM schema and application were unavailable; internal interoperability checks do not replace vendor validation. |

## Latest pushed verification source — 2026-11-26

**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source commit:** `a2739d0114e742fd17d18866eb6fcbd250857ba1` (refines the headless regression to count offscreen `CONTACTS` label shapes).
**Product release binary SHA-256:** `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Exact-source offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0; 57 passed, 0 failed. |
| Formatting and whitespace | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Remote GitHub Actions and commit statuses | NOT RUN | Actions API returned `total_count: 0`; combined commit status returned an empty status list for this exact SHA. |
| Native UI and vendor compatibility | NOT TESTED | Native visual review and IBM ANX/XSD/application validation remain separate open gates. |

The change refines test coverage only; the optimized application binary remains byte-identical to the installed `4a373a…` release. Product behavior and install evidence are recorded under [LOCAL-INSTALL-04](evidence/deployment/LOCAL-INSTALL-04-linux.md), [COMMUNITY-PROXY-01](evidence/vector-ui/COMMUNITY-PROXY-01-linux.md), and [G4-VIEWPORT-CULLING-01](evidence/vector-ui/G4-VIEWPORT-CULLING-01-linux.md).

## Previous synchronized branch verification — 2026-11-26

**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source commit:** `e86814675eaee96ed6f20caff94d6d5b79b9a2c5`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Offline Rust suite and source check | PASS | 54 tests, formatting, diff validation, and offline `cargo check` pass for the graph-layout comparison change. |
| Release build and G0/G3 layout comparison | PASS, CPU only | Release build and 12 paired sparse/hub/disconnected/dense cases at 1,000–10,000 entities; [G3 release evidence](evidence/vector-ui/G3-connectivity-vs-force-release-20261126.md). Native arrange/restore remains unverified. |
| Current-release SQL automation and scale | PASS, synthetic | Current release passes capture, recovery, schema adaptation, CSV/TSV exchange, and 7,121-record joined-header scale checks; [SQL-AUTO-06](evidence/sql-automation/SQL-AUTO-06-current-release-linux.md). Live scheduling and native Automation UI remain unverified. |
| Local desktop/launcher installation | PASS, scoped | Binary, menu/desktop entries, MIME default, icons, and automation helper synchronized to the current build; [LOCAL-INSTALL-02](evidence/deployment/LOCAL-INSTALL-02-linux.md). Native window launch was not tested because the display is unavailable. |
| Independent review of the new Arrange/Restore test | NOT VERIFIED | This continuation performed implementation-owner review only; no separate reviewer handoff is checked in. |
| IBM ANX/XSD/application validation | NOT TESTED | The IBM schema and application were unavailable; internal interoperability tests are not a substitute. |

## Current-user desktop and launcher installation — 2026-11-26

**Task / owner:** Install the proximity-analysis candidate on this machine and synchronize its executable, application-menu entry, desktop shortcut, and icon art; implementation session.
**Product source commit:** `6b59e32524c1ee487394ca1235cef4716d8937ae`.
**Installed release binary SHA-256:** `bd80d18f428655b095484d2f84ddd29ce5c4d369b6169e77d6040766b11d6309`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Per-user executable, installer helper, menu entry, MIME association | PASS | Installed under `/home/sysadmin/.local`; binary hash matches the release artifact; registered case MIME handler is `arch-notebook.desktop`. |
| Desktop shortcut and launcher icon synchronization | PASS | `/home/sysadmin/Desktop/Arch Notebook.desktop` matches the menu entry byte-for-byte; SVG and high-resolution PNG match repository icon hashes. |
| Native application window | BLOCKED | Winit returned `NoCompositor`; X11 was unavailable. Installation checks do not establish a rendered or interactive window. |

Detailed paths, checks, and hashes: [LOCAL-INSTALL-02 evidence](evidence/deployment/LOCAL-INSTALL-02-proximity-linux.md).

## Origin-centered contact groups and proxy ranking — 2026-11-26

**Task / owner:** Combine selected-origin hop layers with contact groups, group-to-group record summaries, and direct cross-group proxy ranking; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline:** `941373e3b8fa537c7ee922c14ab0435e488a3009`.
**Product source commit:** `6b59e32524c1ee487394ca1235cef4716d8937ae`.
**Release binary SHA-256:** `bd80d18f428655b095484d2f84ddd29ce5c4d369b6169e77d6040766b11d6309`.
**Changed paths:** `src/proximity.rs`, `src/main.rs`, `docs/notebook-workflows.md`, `docs/analyst-notebook-parity.md`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting, diff validation, full Rust suite | PASS | Formatting and diff checks exited 0; 50 tests passed, 0 failed. |
| Locked offline compile and optimized release build | PASS | Both exited 0. |
| GitHub Actions for source commit | PASS | [Run 45909562127](https://github.com/pskeffington/arch_notebook/actions/runs/45909562127) completed successfully for the exact candidate SHA; Linux test/fixture, both Linux architectures, both macOS architectures, and Arch package jobs passed. Release attachment was skipped because no release was published. |
| Current-candidate native chart review | BLOCKED | Wayland launch failed with `NoCompositor`; no visual or pointer acceptance is claimed. |
| IBM application compatibility | NOT TESTED | No IBM application or XSD was used. |

See [origin-centered proximity groups evidence](evidence/vector-ui/PROXIMITY-GROUPS-linux-6b59e32.md) for the calculation, fixtures and limits. The follow-up [group contact map acceptance](evidence/vector-ui/PROXIMITY-MAP-01-linux-ffdd781.md) records the bounded visual overview, navigation behavior, current-user installation, local build results and remaining native UI gate. The main chart continues to draw its original entity and relationship records.

## Origin-centered group contact map — 2026-11-26

**Task / owner:** Add a bounded group-level contact overview to the proximity workflow; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source commit:** `26fe459dee702ab6c41da483f7f29d267c60fed7`.
**Release binary SHA-256:** `b6955b38b8af4596e0d88a9667d7d60637ffdd7536c63a806d2d8334e648172f`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Offline Rust tests, compile and optimized build | PASS | 51 tests passed; locked offline check and release build exited 0. |
| Local executable, desktop launcher and icon synchronization | PASS | Installed binary matches release hash; launcher entries validate; repository and installed icon hashes match. |
| GitHub platform/package matrix | PASS | [Run 45914257016](https://github.com/pskeffington/arch_notebook/actions/runs/45914257016) passed Linux tests/fixture determinism, both Linux architectures, both macOS architectures, and Arch packaging for the exact source commit. Release attachment was skipped because this was not a release event. |
| Native visual/input review | BLOCKED | Current Wayland diagnostic returned `Operation not permitted`; no native appearance or pointer acceptance is claimed. |

The bounded map is in the Analysis pane. It samples at most 120 groups and 40 hop layers on large cases; the complete group and exact interaction records remain listed below it. See the [detailed acceptance record](evidence/vector-ui/PROXIMITY-MAP-01-linux-ffdd781.md).

## UI-01 chart entity-card picking — 2026-11-26

**Task / owner:** Align entity picking and viewport transforms with the visible chart cards; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline:** `0c5cf7bd59b1a66713e165943962354a1c7f6ae0`.
**Product source commit:** `e7b64653d798faeb858d5fb34c61bfd96a2db336`.
**Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `docs/evidence/vector-ui/UI-01-card-hit-geometry-linux.md`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting, diff validation, and unit tests | PASS | Formatting and diff checks exited 0; 48 offline Rust tests passed, including new transform round-trip, zoom-anchor, degenerate viewport, card-edge, and overlap-order tests. |
| Offline compile check | PASS | `cargo check --locked --offline` exited 0. |
| Strict Clippy | FAIL | Existing warning-as-error findings prevent a clean repository-wide Clippy run; no unrelated lint cleanup was included. See the detailed evidence record. |
| Current-candidate native chart interaction | BLOCKED | Desktop pointer/capture access is unavailable in this session; no visual acceptance is claimed. |
| IBM Analyst's Notebook compatibility | NOT TESTED | No IBM application or XSD was used. |

Full commands, results, and scope limits: [UI-01 card hit geometry evidence](evidence/vector-ui/UI-01-card-hit-geometry-linux.md).

## Synchronized-branch SQL automation recheck — 2026-11-26

**Task / owner:** Continue the Rust notebook build with emphasis on SQL automation, acceptance evidence, and synchronized progress; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source commit:** `8c08da9285c6104d35a66386676b244d44f1270e`; evidence baseline `991c4764715c71e682af784fbdbf9e2db3e833c7`.
**Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `1ef36e782956eaf1196a06d074ad8edf02591b45095d9acb805879b000cfe941`.

| Gate | Result | Evidence |
| --- | --- | --- |
| SQL capture, comparison, failure retention, reviewed schema adaptation, batch resume, CSV/TSV exchange | PASS | Current release automation harness passed; source SQLite remained unchanged during app captures and preflight. |
| Joined-header SQL scale | PASS | Fictional 7,121-row joined workload; two snapshots, 1,500 entities, composite identities, repeated pairs, zero skipped rows. |
| Linux systemd schedule generation and invocation | PASS, stubbed | Five cadences and generated service invocation passed using a temporary `systemctl` stub. |
| macOS LaunchAgent generation and runner simulation | PASS, stubbed on Linux | Five cadences, rollback, rolling-window output, and unchanged SQLite source passed using a temporary `launchctl` stub. |
| Native Automation UI | BLOCKED | App exits with Winit `Connection(NoCompositor)` despite compositor globals being visible to `wayland-info`; X11 is inaccessible. |
| Live scheduler dispatch and IBM application/schema validation | NOT TESTED | No live timer/LaunchAgent run or IBM XSD/application was used. |

Exact commands, fixture scope, and report hashes are in [SQL-AUTO-05](evidence/sql-automation/SQL-AUTO-05-current-branch-linux.md). This recheck updates the earlier SQL baseline for the synchronized release; it does not claim native UI, live scheduling, production-schema, or IBM interoperability acceptance.

## Chart viewport navigation — 2026-11-26

**Task / owner:** Improve analyst recovery from an over-zoomed chart; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source baseline:** `a356e0d06f9443aa3cb2f463392ebce8e7a16a18`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Fit chart access | PASS, source/test | Added **Fit chart** to Quick open search and `Ctrl/Command+0`; **View…** lists the shortcut. Quick-command search regression passed. |
| Rust suite and build | PASS | Focused command search: 1 passed; full suite: 34 passed; offline `cargo check` and optimized release build exited 0. |
| GitHub Actions for the pushed candidate | PENDING | GitHub Actions lookup found no PR-triggered run and no combined commit status contexts for `5cf1db5c573f69ce57f6322bb66bb2c958770de1`; no remote CI pass is claimed. |
| Current-candidate native interaction and semantic Qwen review | BLOCKED | X display access failed. A local aggregate-only Qwen structural review ran separately; no current-candidate rendering, shortcut input, or semantic/content review is claimed. See [CLINTON-QWEN-STRUCTURAL-01](evidence/sql-automation/CLINTON-QWEN-STRUCTURAL-01-20261128.md). |

Detailed evidence: [UI-NAV-FIT-01](evidence/vector-ui/UI-NAV-FIT-01-linux.md). The previous 1440×900 screenshot is contextual only; repeat visual and keyboard acceptance in an input-capable desktop session.

## Current-release SQL automation recheck — 2026-11-26

**Task / owner:** Revalidate SQL capture, recovery, scale and scheduler runners on the pushed route-geometry candidate; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source commit:** `fae574dc74af78ae19a950641ba2b14e652901c2` (branch HEAD `e51c83a` contains documentation updates only).
**Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `63d008c9d5431390663da1f58f803534908e3e5d4904d67054895b3391093088`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| SQL automation, diagnostics, recovery and chart interchange | PASS | Preflight, immutable capture/comparison, one-run override isolation, schema drift rejection, failed-run retention, reviewed schema repair, resumable batches, mismatch rejection, and CSV/TSV round-trip. |
| Joined-header scale ingestion | PASS | 7,121 records, 1,500 entities, 7,121 composite IDs, 4,121 repeated endpoint pairs, two snapshots, nullable properties, zero skipped rows. |
| Per-entity live SQL source lookup invariants | PASS, headless | Three focused tests and the 31-test suite cover multi-source composite identities, read-only behavior, unsafe/ambiguous query rejection, and bounded display; native inspector interaction remains pending. See [SQL-DRILLBACK-01](evidence/sql-automation/SQL-DRILLBACK-01-linux.md). |
| Linux scheduler generation and invocation | PASS, stubbed | Five cadences and generated service invocation passed; `systemctl` was a temporary stub. Live timer firing is not tested. |
| macOS LaunchAgent generation and runner simulation | PASS, stubbed | Five cadences and generated runner passed; `launchctl` was a temporary stub and the real macOS host was untouched. |
| Candidate GitHub Actions run | PENDING | Actions API returned zero runs for product source commit `fae574d`. |
| Native Automation UI and IBM XSD/application compatibility | BLOCKED / not tested | No native pointer-driven review or IBM application/schema validation was performed. |

Exact commands, fixture and binary hashes, report digests, and scope limits are in [SQL-AUTO-04](evidence/sql-automation/SQL-AUTO-04-current-release-linux.md). All harness data is fictional; the joined-header fixture contains no message bodies. Scheduler stubs establish generated runner behavior, not live host dispatch.

## Previous SQL automation acceptance — 2026-11-26

**Task / owner:** Resume Arch Notebook SQL ingestion and automation acceptance; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source baseline:** `bc536880d29075234525c0fde4f6a860caf71101`; product source `a5208a0e3aa8c0fe3b3a1e3f02986e45e97b919a`.
**Installed artifact:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `765f8fa2e6384a2df4747890a9b954920a1668dc35557068f749f89afcea1fc7`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| SQL scale ingestion and longitudinal comparison | PASS | 7,121 composite relationship IDs, two snapshots, 1,500 entities, NULL handling, query-only preflight, and zero skipped rows; synthetic fixture. |
| SQL automation, failure diagnosis, recovery, and interchange | PASS | Preflight, baseline/current capture, history, comparison export, schema drift rejection, retained failures, CSV/TSV round trip, reviewed query repair, and batch resume all passed. Mismatched resume configuration was rejected. |
| Linux scheduler generation and service invocation | PASS, harness scoped | Five cadences and case-isolated units; generated service command captured the expected snapshot and CSV. `systemctl` was stubbed; live timer dispatch is unverified. |
| macOS LaunchAgent runner | PASS, simulation scoped | Five cadences and case isolation; runner simulation generated the expected capture and CSV. `launchctl` was stubbed on Linux; macOS dispatch is unverified. |
| Native Automation UI, IBM XSD/application compatibility | BLOCKED | Pointer-driven Automation UI review and IBM validation have not been performed. |

Historical commands, exit status, report hashes, and fixture caveats are recorded in [SQL-AUTO-02](evidence/sql-automation/SQL-AUTO-02-current-release-linux.md). This older acceptance record does not describe the current release binary above.

## Linux analyst packet acceptance — 2026-11-26

**Task / owner:** `SNG-DELIVERY-004`, local analyst finding-packet export; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Candidate source:** `bb5443ef27b4be03f35de58a9d07e000627e5ccb` (feature commit `84ddfd3`, followed by staging-cleanup hardening).
**Release artifact:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `d02aca2aa4f59a229f7f12c533ca8bcb9abd7782d2d94ec7d4ff43eca5fc99c4`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Format, Rust tests, offline check, debug/release builds | PASS | Formatting, `git diff --check`, 27 tests, locked offline check, debug build, and optimized release build all exited 0. |
| CLI acceptance on fictional data | PASS | Default redaction, explicit disclosures, 1-entity alternate-chart selection, source immutability, valid SVG, parallel/directed/self-loop semantics, no-overwrite, duplicate JSON rejection, and 0700/0600 output permissions passed. |
| Rendered SVG | PASS, implementation-owner inspection | `rsvg-convert` rendered a 1200×800 SVG with bounded labels, parallel lanes, direction cues, self-loop, and legend. This is a compact overview, not the interactive chart. |
| Independent code/security review | BLOCKED | No independent review handoff is present; implementation-owner review is recorded separately. |
| IBM application and macOS support | BLOCKED / not tested | IBM ANX XSD/application validation was not performed. Atomic packet-directory publication is Linux-only; macOS was not cross-compiled or run. |

Exact commands, report and artifact digests, and scope limits are in [PACKET-01](evidence/analyst-packet/PACKET-01-linux.md). This CLI handoff is separate from SQL automation acceptance above and does not establish IBM interoperability or native UI acceptance.

> **Historical record:** The original local build below is retained as written for provenance. Its dirty `main` worktree and “no tests” findings describe the 2026-11-25 candidate only; they do not describe the current feature branch. Current branch evidence is summarized in [Current continuation: UI-00 and local install](#current-continuation-ui-00-and-local-install).

[![Linux local build: passing](https://img.shields.io/badge/Linux%20local%20build-passing-brightgreen)](#feature-branch-increment-reusable-chart-templates)

Badge scope: recorded Linux compilation at `ada3a01`; runtime, package installation and release acceptance remain separate.

**Task:** `01a2016c-06c1-7b02-bb7d-a0ac3edcdd3c` — continue the Rust Arch Notebook parity build, including SQL automation and deployed application graphics.
**Repository:** `/home/sysadmin/github/arch_notebook`
**Branch / source base:** `main` / `7677248ebbc8732a38aa8c8187f2fe61c1799c7a`
**Recorded:** 2026-11-25
**Candidate state:** dirty worktree; no commit or immutable source-tree digest identifies the combined candidate. The checks below apply to the working tree at the time they ran.

This record accepts the local Linux Rust build and the listed packaging metadata checks. It is not a release-readiness decision or authorization to publish or deploy.

## Gate results

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Scope and source recorded | PASS | Task thread ID, repository, branch, base SHA, and local-build scope are recorded above. The worktree contains accumulated uncommitted changes. |
| Local format and integration build | PASS | Formatting, diff validation, offline Rust check, debug build, and release build all exited 0; exact commands are below. |
| Linux desktop metadata and icon files | PASS | `desktop-file-validate` accepted the desktop entry; SVG and 1024×1024 PNG files were recognized. |
| Packaging script syntax | PASS | `bash -n` accepted the Linux user installer and macOS bundler. This checks syntax only. |
| Behavioral and independent verification | BLOCKED | No automated tests or independent review were run. Source-identity navigation and UI behavior have build evidence only. |
| Native platform packages and runtime graphics | BLOCKED | The Arch package and macOS `.app`/`.icns` were not built here. This Linux run did not execute macOS `sips`/`iconutil`, install the app, or inspect its runtime window icon. |
| IBM interoperability and visual parity | BLOCKED | ANX was not validated against IBM's XSD or Analyst's Notebook. IBM's published guidance locates those artifacts in an installed Analyst's Notebook tree: [IBM ANX format support note](https://www.ibm.com/support/pages/node/546669). No IBM application review was performed. |
| Release authorization | BLOCKED | No release dossier, trusted approval signature, or human publication/deployment authorization is present. |

## Commands and results

The following local build chain completed with exit code 0:

```sh
cargo fmt --all -- --check
git diff --check
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline
```

The packaging and asset checks completed with exit code 0:

```sh
bash -n scripts/install-linux-user.sh scripts/package-macos.sh
desktop-file-validate packaging/arch/arch-notebook.desktop
file assets/icons/arch-notebook.svg assets/icons/arch-notebook.png
identify assets/icons/arch-notebook.png
```

No automated tests were run. Cross-platform packaging and live scheduler/UI behavior remain unverified.

## Artifact digests

These hashes identify the local binaries and icon assets produced or checked for this candidate; they do not bind the dirty source tree to a commit.

| Artifact | SHA-256 |
| --- | --- |
| `/tmp/arch-notebook-target/debug/arch-notebook` | `eca98ce1ea794650e8bcf89299900e1beebe53dc6fd4664b9d3ec60960a96aa8` |
| `/tmp/arch-notebook-target/release/arch-notebook` | `ad7e8ca413583cda5ffe3502b670c7e0dc33c3c23f7494a88a52fc47e7ce17f0` |
| `assets/icons/arch-notebook.svg` | `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8` |
| `assets/icons/arch-notebook.png` | `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88` |

## Candidate worktree paths

These modified and untracked paths were present during acceptance. The worktree contains accumulated changes; this list does not claim that every path was authored in this acceptance turn.

- `.github/workflows/build.yml`, `README.md`, `src/main.rs`
- `assets/icons/arch-notebook.svg`, `assets/icons/arch-notebook.png`
- `docs/analyst-notebook-parity.md`, `docs/automation.md`, `docs/build-and-deploy.md`, `docs/chart-legends.md`, `docs/evidence-handoff.md`, `docs/notebook-workflows.md`, `docs/project-reference.md`, `docs/sql-ingestion.md`
- `packaging/arch/PKGBUILD`, `packaging/arch/arch-notebook.desktop`, `packaging/arch/arch-notebook-case.xml`, `packaging/arch/arch-notebook.install`
- `packaging/systemd/arch-notebook-automation.service`
- `scripts/install-linux-automation.sh`, `scripts/install-linux-user.sh`, `scripts/install-macos-automation.sh`, `scripts/package-macos.sh`, `scripts/run-macos-automation.sh`

## Next acceptance work

1. Run the repository's automated tests and record their exact results.
2. Build and inspect the Arch package and Linux archive installation on Linux; build the macOS `.app` and inspect its Dock/Finder icon on macOS.
3. Exercise SQL scheduling, cancellation, resume, and failure review with a disposable case on supported platforms.
4. Validate ANX fixtures with IBM's XSD and application, then update the parity map with the results.
5. Repeat the checks against a clean, immutable source commit before any release decision.

## Current continuation: UI-00 and local install

### UI-01 geometry and coincident-route follow-up

| Check | Result | Evidence |
| --- | --- | --- |
| Loop geometry, label separation, arrowhead placement, and selection regression | PASS | [UI-01 self-loop evidence](evidence/vector-ui/UI-01-self-loop-linux.md); focused and full offline Rust suites pass. |
| Distinct routes and exact picking for links sharing endpoint positions | PASS, source test | [UI-01 coincident-route evidence](evidence/vector-ui/UI-01-coincident-routes-linux-fae574d.md); all nine fixture routes differ and each route sample selects its exact record. |
| Independent code review | PASS | [UI-01 review handoff](evidence/vector-ui/UI-01-coincident-routes-review-linux.md); source route/picking review and disclosure wording recheck passed. This does not include security review. |
| Earlier self-loop candidate Linux render | PASS, render-only | Screenshot inspected at 1440×900; input was unavailable. It does not visually verify the coincident-route follow-up candidate. |
| Current-candidate native interaction and UI-00 baseline | BLOCKED | Current release launch stayed alive for the bounded smoke interval, but display capture and pointer/keyboard input are unavailable; repeated interaction and performance sampling remain open. |
| IBM application/schema interoperability | NOT TESTED | Requires validation in the IBM application/schema environment. |

The source changes repair self-link and coincident-position routes while preserving record-level picking. They do not close UI-00 or establish IBM visual parity. Continue with an input-capable native session and the established UI-00 sequence before treating UI-01 as natively accepted.

**Task / owner:** Continue the Rust Arch Notebook parity build; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline/source:** `abffef37101bd298defd1e243c257a26e370323f` (matches `origin/codex/ui-00-chart-baseline` at this record).
**Scope:** Reconcile current branch acceptance status across SQL automation, native chart interaction, IBM interoperability, and the installed Linux launcher.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Branch/worktree provenance | PASS | `HEAD` and `origin/codex/ui-00-chart-baseline` resolve to `abffef37101bd298defd1e243c257a26e370323f`; worktree was clean before this documentation update. |
| SQL automation | PASS, scoped | Rolling previous-UTC-month selection is exercised through the Linux generated-service harness and the macOS runner harness. Evidence: [SQL-SCHED-03](evidence/sql-automation/SQL-SCHED-03-dynamic-window-linux.md) and [SQL-SCHED-04](evidence/sql-automation/SQL-SCHED-04-macos-window-linux-harness.md). Actual systemd timer dispatch and native macOS `launchd` dispatch remain unverified. |
| Local Linux installation and launcher/icon sync | PASS, scoped | Release binary, desktop/menu entries, case-file association, automation installer, and icons match and validate as recorded in [LOCAL-INSTALL-01](evidence/deployment/LOCAL-INSTALL-01-linux.md). GUI relaunch/interaction was not verified. |
| UI-00 native chart acceptance | BLOCKED | The latest runtime record describes a screenshot marked “Not Responding” and unavailable stable compositor access; responsive interaction, repeated workload sampling, and visual inspection remain open. Evidence: [UI-00 recheck](evidence/vector-ui/UI-00-linux-recheck-e44d485.md). Do not advance to UI-01 until this gate passes. |
| IBM interoperability | BLOCKED | Internal supported-subset ANX round-trip passes, but IBM XSD and Analyst's Notebook application validation have not run. Evidence: [ANX-ROUNDTRIP-01](evidence/anx/ANX-ROUNDTRIP-01-linux.md). |
| Remote workflow result for this source | BLOCKED | GitHub Actions API was unreachable during this continuation; no workflow conclusion is asserted for `abffef3`. Earlier CI evidence applies only to the source SHAs named in those records. |

No build or test command was run for this documentation-only reconciliation. Next action is to repeat the UI-00 sequence in a stable native desktop session; retain the IBM validation and live scheduler dispatch gates separately.

## SQL acceptance: synchronized branch automation and scale recheck

**Task / owner:** Revalidate SQL automation behavior on the current pushed feature-branch release; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source:** `750b6cb9f9214089678cc3341dae5ecd7c65005b`; release binary SHA-256 `f8c14cabd73164b27218068685d75c62ecd2e9d854287299b9aca534ec373032`.
**Scope:** Re-run the full synthetic SQL automation recovery workflow and 7,121-row joined-header scale harness against the release binary.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| SQL automation/recovery workflow | PASS | Query-only preflight, override isolation, schema drift diagnosis and reviewed repair, failed-run retention, batch resume, mismatch rejection, and CSV/TSV round-trips passed. The app left the SQLite source unchanged. |
| Joined-header SQL scale | PASS | 7,121 result rows, 1,500 endpoint entities, 7,121 distinct composite record IDs, nullable attributes, two snapshots, and deliberate confidence decline all passed; source unchanged during each capture. |
| Native GUI and live scheduled dispatch | BLOCKED | The graphical display rejected `wayland-info` and X11 access. Linux systemd and macOS LaunchAgent live dispatch were not exercised by these CLI harnesses. |
| IBM interoperability | BLOCKED | IBM XSD/application validation was not performed. |

Exact commands, exit codes, paths, and report digests are in [SQL-AUTO-03](evidence/sql-automation/SQL-AUTO-03-current-branch-linux.md). These automated synthetic results do not establish interactive, live-scheduler, production-schema, IBM-compatibility, or release acceptance.

## UI-00 acceptance: current fixture reproduction

**Task / owner:** Verify UI-00 fixture determinism and bounded workload generation; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source commit:** `4fda60ad5d98caa198364ab1c4c40da8af8d1101`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Correctness fixture | PASS | Two generated copies matched each other and `fixtures/chart/correctness.case.json`; JSON parsing passed. 8 entities, 8 relationship records, fixture SHA matches the manifest. |
| Workload families | PASS | Sparse/hub/disconnected/dense at 1,000 nodes each reproduced byte-for-byte with 3,000/3,000/2,000/10,000 links. Dense 10,000-node output reproduced twice at the 100,000-link cap. |
| Native baseline and measurements | BLOCKED | Display access is denied. No interaction or rendering metric is claimed. |
| CI for current candidate | PENDING | Workflow source runs `cargo test` and generator checks, but no implementation PR run is associated with this candidate. Local reproduction does not substitute for CI. |

See [UI-00 fixture reproduction evidence](evidence/vector-ui/UI-00-fixture-reproduction-linux.md) for exact commands, hashes, and the corrected local summary assertion.

## SQL scheduler acceptance: current-release installer and runner recheck

**Task / owner:** Verify isolated Linux systemd and macOS LaunchAgent installation/runner paths with the current release; implementation session.
**Product source:** `750b6cb9f9214089678cc3341dae5ecd7c65005b`; release binary SHA-256 `f8c14cabd73164b27218068685d75c62ecd2e9d854287299b9aca534ec373032`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Linux unit installer / generated-service invocation | PASS, harness-scoped | Five cadences, failed-preflight rejection, special paths, case-specific removal, scheduled capture and CSV export passed under temporary `systemctl` stubs. |
| macOS LaunchAgent installer / generated runner | PASS, Linux harness | Five cadences, failed install/replacement rollback, special paths, scheduled capture and CSV export passed under temporary `launchctl` stubs. |
| Live host dispatch | NOT VERIFIED | No live systemd timer or native macOS launchd dispatch occurred. |

See [SQL-SCHED-06](evidence/sql-automation/SQL-SCHED-06-current-branch-linux.md) for commands, exact report hashes, and scope. These installer/runner harness results are separate from live scheduler acceptance.

## Feature-branch increment: aspect-aware entity type layout

**Task / owner:** Improve analyst-facing vector organization; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline:** `0e54d8d70378a1a9f47f1268f679e381d105e997`.
**Source commit:** `a09b49c` (`feat: adapt entity clusters to chart canvas`).
**Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `README.md`.
**Scope:** Place entity-type clusters in a canvas-aspect-aware grid, center sparse cluster rows, preserve deterministic type/name ordering, and leave pinned entity positions unchanged.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Rust unit tests | PASS | Offline `cargo test` exited 0; 20 passed, 0 failed, 0 ignored, including three entity-type layout regressions. |
| Offline integration check and release build | PASS | `cargo check --locked --offline` and optimized `cargo build --release --locked --offline` exited 0. |
| Installed release binary, launcher, and art synchronization | PASS | Installed executable matched release binary SHA-256 `54d57b05cd9ebb66de4b60274fb4062e06a96705f4a68f80eabb7649e249bdb7`; desktop entries validate and match; installed SVG/PNG match repository assets. |
| Native chart inspection | BLOCKED | Wayland display access is denied in this session; layout appearance, responsiveness, and input remain unverified. UI-00 remains open. |
| IBM interoperability | NOT IN SCOPE / STILL BLOCKED | This feature does not establish IBM ANX schema or application compatibility. |

See [UI-ENTITY-TYPE-LAYOUT-02 evidence](evidence/vector-ui/UI-ENTITY-TYPE-LAYOUT-02-linux.md) for exact commands, hashes, and limitations. The change is pushed on the feature branch; proceed with native UI-00 interaction review when a compositor session is available.

## SQL acceptance: current-branch scheduler recheck

**Task / owner:** Verify rolling SQL window scheduling against the current application release; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source:** `a09b49c6faddbd0401c9ac59b191e8151a830e70`; current branch head at recheck: `cb0ff938ae1328b3f9a9859b19edae874f82138c`.
**Scope:** Execute the Linux generated service command and macOS packaged runner against isolated synthetic databases using the current release binary.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Linux generated-service SQL scheduling | PASS, harness-scoped | Five cadence installers passed; generated service saved one scheduled capture and one-row CSV; prior-UTC-month start/end behavior passed. `systemctl` was stubbed. |
| macOS automation runner and rollback lifecycle | PASS, harness-scoped | Five LaunchAgent schedules, preflight, rollback and removal passed under a temporary `launchctl` stub on Linux. Live `launchd` was not run. |
| Source DB immutability | PASS | macOS harness confirmed unchanged source SHA-256 `7c62cccec9459451a0c00da3cbb35265be7bf36f7ddb81c6bf18527c0d1a3071`. |
| Live host scheduler dispatch | BLOCKED | systemd user manager access and a native macOS host are unavailable in this run. |

Exact commands, report hashes, window boundaries, and work directories are recorded in [SQL-SCHED-05](evidence/sql-automation/SQL-SCHED-05-current-branch-window-linux.md). These harnesses use synthetic SQLite data and make no claims about live scheduler dispatch.

## Feature-branch increment: geodesic navigation and responsive ANX parsing

**Task / owner:** Improve analyst chart navigation and exchange-file ingestion; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline:** `dff7acfb5a5ffa9c79b692e0e32841821c4f64c8`.
**Source commit:** `b637d965ae5ed4aeea9ec4300860523dfb95b71b`.
**Changed paths:** `src/main.rs`, `README.md`, `docs/analyst-notebook-parity.md`, `docs/notebook-workflows.md`, and `docs/research/social-network-graphing-and-delivery-review.md`.
**Scope:** Add defined unweighted, undirected shortest-hop layers from an entity; move desktop ANX XML parsing to a worker while preserving synchronous CLI/automation behavior and guarding against chart-switch races.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting, diff, and Rust unit tests | PASS | Formatting and diff checks exited 0; offline tests: 22 passed, 0 failed, 0 ignored. |
| Offline integration check and release build | PASS | `cargo check --locked --offline` and optimized release build exited 0. |
| ANX internal round-trip and rejection behavior | PASS, internal scope | 8 entities, 8 relationships, and one label retained; malformed shape and DTD external entity rejected without case mutation. IBM XSD/application validation remains NOT RUN. |
| Current-source SQL scheduler harnesses | PASS, harness-scoped | Linux generated-service and macOS runner simulation selected the expected rolling-month row; SQLite source unchanged in macOS simulation. Host dispatch remains unverified. |
| User install and desktop graphics synchronization | PASS, scoped | Installed binary equals the release artifact; desktop launcher validation passed; installed and repository icon hashes match. |
| Remote workflow for source commit | BLOCKED / unverified | GitHub Actions API was unreachable; no remote workflow conclusion is asserted for `b637d96`. |
| Native UI rendering | PASS, render-only | The installed app rendered the fictional UI-00 chart, navigator, and geodesic panel in a native window. See [UI-00 render evidence](evidence/vector-ui/UI-00-native-render-20261126.md). |
| Native chart/input and ANX dialog interaction | BLOCKED | Pointer/keyboard event injection remains unavailable; responsive interaction and visual parity against IBM are not established. |

Exact commands, artifact hashes, report hashes, and remaining interoperability/platform gates are in [UI-INGEST-NAV-01](evidence/vector-ui/UI-INGEST-NAV-01-linux.md). The source commit is pushed on this non-main branch.

## Feature-branch increment: reusable chart templates

**Task / owner:** Continue Analyst's Notebook parity, especially adaptability; orchestrator-owned implementation.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/analyst-notebook-parity`.
**Baseline:** `204b114f9188d59ffeacf913a13acdd49cc593c2`.
**Source commit:** `ada3a012b813ed75fd0f119b2976e53c5c3d3810` (`feat: add reusable chart templates`).
**Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `docs/analyst-notebook-parity.md`.
**Scope:** Case-local chart templates store entity-type representations/colors and relationship vocabulary; apply styles to matching chart entities; provide defaults for manually added entities; persist the active template with chart tabs; allow template deletion. Import mapping and SQL query transformation are outside this increment.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Feature source recorded on non-main branch | PASS | Source commit above is on `codex/analyst-notebook-parity`; subsequently pushed to GitHub; originally local at time of verification. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0 against the feature source. |
| Offline integration check | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Debug build | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0. |
| Release build | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0. |
| Template runtime behavior / independent review | BLOCKED | No automated tests or independent review were run. Build evidence does not verify save, apply, delete, or chart-switch interactions in a running UI. |
| IBM visual and ANX parity | BLOCKED | No IBM application or IBM XSD/fixture validation was available. |
| Release authorization | BLOCKED | No publication or deployment was requested or performed. |

The four build commands above completed with exit code 0. The artifacts were produced from the committed Rust source; documentation-only acceptance logging is committed separately. Their SHA-256 digests are:

| Artifact | SHA-256 |
| --- | --- |
| `/tmp/arch-notebook-target/debug/arch-notebook` | `7956f64c8c8ca576eee056d91649042955543471f422a4b26f6534fe454e2442` |
| `/tmp/arch-notebook-target/release/arch-notebook` | `b48b93962e3d3a5f265c9a7aa1da46eab73b91454a614e0b0b41b70b26949c53` |

The product change is committed and reproducibly builds locally. Next acceptance work is interactive template workflow review, automated verification when requested, and IBM-backed interoperability review when the vendor artifacts are available.

## Feature-branch increment: Clinton email candidate capture

**Task / owner:** TRANSCRIBE-to-Arch Notebook integration evidence and SQL capture progress; orchestrator-owned.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/analyst-notebook-parity`.
**Baseline:** `1c7f912170417aa40e30b227a72234a60de13ccd`.
**Source status:** The SQL progress implementation and documentation are pending in this candidate; this acceptance entry records a local dataset run and does not imply a trusted or released TRANSCRIBE handoff.
**Scope:** Install the application on this Linux user's desktop, prepare a local review-required candidate case from the Clinton email PDF in the TRANSCRIBE share, and capture its header-derived relationship rows from SQLite.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Local SQL capture | PASS | The saved case contains 2,813 entities, 6,173 relationship records, two snapshots, and two automation runs. Both snapshots report 0 skipped rows; the latest batch completed with 6,173 captured relationships. The later capture followed cleanup of three out-of-range normalized dates. |
| Case and source safety | PASS | The case is explicitly titled `CLINTON EMAIL PUBLIC RELEASE · LOCAL REVIEW REQUIRED`; all extracted records remain unreviewed candidates. The SQLite source and case are local share artifacts and are not included in this repository. Raw message bodies were not stored in the analysis database. |
| Data completeness and semantic validation | BLOCKED | Extraction yielded 4,767 header candidates from 7,121 PDF pages, not a verified complete set of 7,000 emails. No human validation or released TRANSCRIBE handoff was performed. The counts above describe the captured candidate graph only. |
| Qwen aggregate structural review | PASS, bounded | Local Qwen 3.5 9B reviewed a private aggregate-only prompt. One unsupported explanation for human-review flags was rejected; semantic content review remains NOT ESTABLISHED. See [CLINTON-QWEN-STRUCTURAL-01](evidence/sql-automation/CLINTON-QWEN-STRUCTURAL-01-20261128.md). |
| Desktop installation | PASS | The Linux user installer placed the application launcher, icon assets, MIME definition, and desktop shortcut in the user's local application directories. `desktop-file-validate` passed both installed launchers; the installed executable reported version 0.1.0. Desktop filesystem trust metadata was unsupported, although the shortcut is executable. |
| Rust build and diff checks | PASS | `cargo fmt --all -- --check`, `git diff --check`, offline `cargo check`, debug build, and release build passed for the candidate source before installation. No automated tests were run. |

Local artifacts are held in the user's TRANSCRIBE share and excluded from Git. Email names, addresses, subjects, bodies, and row-level analysis are intentionally omitted from this repository record. The DB stored source/transcription provenance and records that Qwen was not run.

**Current Rust CLI follow-up (2026-11-25, source `aaf8ece`):** The retained case still opens in the release CLI; run and batch history list two `PASS` runs and two `COMPLETE` batches. Its disabled saved job passes a direct query-only validation without saving. Exporting the two saved snapshots produces three `DATE CHANGED` rows. Replaying the latest saved run against the local SQLite source passes with 6,173 records, zero skipped rows, and zero deltas; case and SQLite SHA-256 values match before and after the read-only checks. See [SQL-CANDIDATE-01 evidence](evidence/sql-automation/SQL-CANDIDATE-01-readonly-revalidation-linux-aaf8ece.md). This verifies saved-case compatibility and current query reproducibility only.

**CSV exchange follow-up (2026-11-25, source `f6c5994`):** The latest SQL capture exported to CSV, received a generated import design, and re-imported into an empty disposable case with 6,173 records created, 0 skipped, and all endpoint names, relationship labels, source references, and notes retained. The original case and SQLite source were unchanged. See [SQL-CANDIDATE-02 evidence](evidence/sql-automation/SQL-CANDIDATE-02-csv-roundtrip-linux-f6c5994.md). This is Arch Notebook internal exchange acceptance; IBM receiving-application validation, completeness, human review, release approval, and Qwen analysis remain unverified.

Next work: improve analyst-facing vector organization for larger relationship graphs, then validate that layout on synthetic fixtures and inspect it in the running desktop application. Dataset completeness, human review, and Qwen analysis remain separate gates.

## Feature-branch increment: analyst vector organization

**Task / owner:** Improve chart organization for analyst use; orchestrator-owned implementation.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/analyst-notebook-parity`.
**Baseline:** `7a5155e` (`feat: report SQL capture application progress`).
**Scope:** Make entity-type layout use compact, deterministic clusters and make alphabetical-grid column count track the most recently rendered canvas aspect ratio. Pinned entity positions remain unchanged.
**Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `docs/build-acceptance.md`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Unit tests | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0; 4 passed, 0 failed. |
| Offline integration check and debug build | PASS | Offline `cargo check` and debug `cargo build` completed with exit 0. |
| Interactive chart review | BLOCKED | The changed layout controls were not exercised in a running GUI; no visual acceptance is claimed. |

The initial grid implementation used a 1.6 chart aspect estimate. It now uses the most recently rendered chart canvas aspect, with a 1.6 default before the first chart frame and a 0.75–2.5 clamp for narrow and ultrawide windows. The layout action runs before the current frame's chart allocation, so a resize affects the next layout invocation.

**Follow-up verification:** Entity-type position calculation is now an isolated pure function with a regression test covering deterministic name order, type grouping, and exclusion of pinned entities from the generated positions. `cargo fmt --all -- --check`, `git diff --check`, offline tests (5 passed, 0 failed), offline `cargo check`, offline debug build, and offline release build all exited 0. Interactive chart review remains BLOCKED.

**Aspect-ratio follow-up:** Source commit `8dfa690` makes alphabetical-grid dimensions use the last observed chart-canvas aspect. A regression test checks that wider canvases allocate more columns and fewer rows, empty input returns an empty grid, and invalid aspect values use the stable default. `cargo fmt --all -- --check`, `git diff --check`, offline tests (6 passed, 0 failed), offline `cargo check`, offline debug build, and offline release build all exited 0. The installed release binary matches SHA-256 `8021eb8e1232840977a1c625ad4f79964c15f44d3293e1cb6b384e0d94d2ec24`; both launcher files pass `desktop-file-validate`. Visual review remains BLOCKED because the session cannot open an X display.

**Local desktop installation:** The vector organization release executable was installed to the user's local application bin. The latest binary digest is recorded above; the installed executable reports version `0.1.0`. Both the application-menu entry and desktop shortcut pass `desktop-file-validate`. Installed icon assets match the repository SVG/PNG digests above. This verifies local file installation and metadata, not an interactive visual review.

## Feature-branch increment: Analysis Method Card

**Task / owner:** SNG-DELIVERY-001, active-chart Analysis Method Card; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline:** `008537813d60dd76db0732b06f87b6266a02603a`.
**Product source commit:** `a5208a0e3aa8c0fe3b3a1e3f02986e45e97b919a`.
**Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `docs/analyst-notebook-parity.md`.
**Scope:** Add a transient, read-only chart summary and current graph-method notes; copy only aggregate/method text; clear cached results on graph structure and relationship date/direction edits.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Rust formatting, diff, and unit tests | PASS | Format and diff checks exited 0; 25 offline Rust tests passed, 0 failed, 0 ignored. |
| Offline check and debug/release builds | PASS | All three locked offline commands exited 0. Debug SHA-256 `dfab5049a21a08063a3bf244bcd16d164d51aed516d01d7f18e3034d1697c38d`; release SHA-256 `765f8fa2e6384a2df4747890a9b954920a1668dc35557068f749f89afcea1fc7`. |
| Local Linux install and launcher/icon sync | PASS, scoped | Installed binary matches release hash; menu and desktop entries compare byte-for-byte and both validate; MIME handler resolves to `arch-notebook.desktop`; release version is `0.1.0`; installed icon and automation-installer hashes match the repository. |
| Fictional method-card metrics and copy privacy | PASS, unit-tested | Fixture covers isolates, directed/undirected records, self and repeated/opposite-direction pairs, missing endpoints, and valid/blank/invalid dates. Copied text omits chart title, names, IDs, relationship labels, source IDs, notes, and evidence; case serialization remains unchanged. |
| Native method-card interaction and screenshot review | BLOCKED | The new installed window could be launched, but the compositor capture bridge failed and pointer/keyboard input is unavailable. The older UI-00 screenshot predates this feature. |
| Independent review provenance | NOT VERIFIED | The roadmap has prior code/security review statements, but no separate handoff/report artifact is checked in; this session did not perform independent review. |
| IBM visual or XSD/application validation | BLOCKED / not run | No IBM application or schema was used; this increment makes no IBM parity claim. |

See [UI-ANALYSIS-CARD-01 evidence](evidence/vector-ui/UI-ANALYSIS-CARD-01-linux.md) and the [phased implementation roadmap](roadmaps/social-network-graphing-roadmap.md). The native and independent-review gates remain open; this is not a release-readiness result.

## Linux SQL scheduler calendar parser — 2026-11-27

The Linux scheduler acceptance now asks `systemd-analyze calendar` to validate the five emitted cadence expressions, while retaining isolated unit-install and service-invocation checks. All five cadence expressions and the complete synthetic SQL scheduler harness pass. Exact-head cross-platform CI passed in [run 45988117371](https://github.com/pskeffington/arch_notebook/actions/runs/45988117371). Live timer delivery remains untested because the harness uses a stub and the current sandbox cannot access the user systemd bus. See [SQL-SCHED-08](evidence/sql-automation/SQL-SCHED-08-systemd-calendar-parser-linux.md) and its [acceptance JSON](evidence/sql-automation/artifacts/SQL-SCHED-08-linux.json).

## Primary Clinton corpus local QA — 2026-11-27

Use the real review-required Clinton candidate case as the primary local QA batch after each candidate build; use synthetic CI fixtures as supplemental coverage. The current Rust release passes the saved job's read-only query preflight and replays its latest saved run against the local SQLite source: 6,173 rows, zero skipped, zero added/removed/changed records. Default-redacted network analysis covers 2,813 entities, with 643 structural communities and 291 proxy candidates. Its report SVG maps the out-of-range saved coordinates into 704 rows (largest row: 4), with all labels in bounds. Entity drag geometry uses one map for nodes, routes, hit targets and navigator throughout a gesture. Saved bounds travel with the active chart, packet export and chart tabs; Fit all explicitly rebases the frame. Labels do not affect entity bounds. PDF, case, and database hashes are unchanged. No raw row-level content is stored in the report or Git. PDF extraction completeness, Qwen analysis, human content review, live scheduler delivery, native interaction on this case, and IBM validation remain open. See [CLINTON-CORPUS-QA-01](evidence/sql-automation/CLINTON-CORPUS-QA-01-linux.md) and its [aggregate-only report](evidence/sql-automation/artifacts/CLINTON-CORPUS-QA-01-linux.json).

## Topology contact-line hover repair — 2026-11-27

**Task / owner:** Restore analyst feedback when hovering sampled proximity-map contact lines; implementation session with topology review.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline / source commit:** `44a6900305d4e969e1bfb473a249feab5bc53fe6` / `30b8020` (`Restore topology line hover preview`), pushed to the non-main remote branch.
**Changed paths:** `src/main.rs`; acceptance documentation and current-install evidence are recorded in the following documentation commit.
**Scope:** Restore hover-route highlighting and a fixed-height preview of groups, record/pair totals, and direction counts. Preserve node hover/click priority and route selection for exact-record review.

| Gate | Result | Evidence |
| --- | --- | --- |
| Full offline Rust suite | PASS | 73 passed, 0 failed, 3 ignored on candidate source. |
| Retained Clinton topology QA | PASS | Both local topology checks passed: 2,813 entities, 6,173 records, a 48-group / 47-pair sample with 1,315 represented records. |
| Optimized release build | PASS | Locked offline release build exited 0; SHA-256 `330bc4fe2446bf8c7bff685ac4e56ddc5ffabc7b82cb59199b2cf537bd0d09b0`. |
| Local desktop launcher and icon sync | PASS | Installed binary matches release; menu and Desktop launchers compare equal and validate; SVG/PNG icons match repository assets. |
| Native visual review | BLOCKED | Capture utilities failed in this session, so current-candidate layout and pointer feel were not visually accepted. |
| Exact-head cross-platform CI | PENDING | Feature-branch push does not trigger the configured workflow; no PR or manual run was created. |

See [PROXIMITY-MAP-HOVER-02 evidence](evidence/vector-ui/PROXIMITY-MAP-HOVER-02-linux.md) and refreshed [LOCAL-INSTALL-08 evidence](evidence/deployment/LOCAL-INSTALL-08-linux.md). The change makes no corpus-semantic or IBM-compatibility claim.

## Current-release SQL automation revalidation — 2026-11-27

**Task / owner:** Revalidate SQL automation against the current installed product and retain the Clinton corpus as the primary batch; orchestrator-owned acceptance.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source / runtime:** `30b8020`; installed release SHA-256 `330bc4fe2446bf8c7bff685ac4e56ddc5ffabc7b82cb59199b2cf537bd0d09b0`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Clinton real-corpus SQL replay | PASS | Query-only preflight and saved-run replay passed with 6,173 rows, zero skipped, zero deltas; hashes unchanged and report packet redacted all identifying fields. |
| SQL automation recovery/interchange | PASS | Schema diagnosis and reviewed repair, failure retention, batch resume, mismatch rejection, CSV/TSV round trips, and read-only source checks passed. |
| 7,121-row joined-header scale | PASS | 7,121 composite record IDs, 4,121 repeated endpoint pairs, nullable attributes, zero skipped, source unchanged during captures. |
| Linux scheduler harness | PASS, stubbed | Five cadence expressions parsed; generated service invocation, case isolation, UTC window and per-case removal passed. Live timer delivery is untested. |
| IBM interoperability / human or Qwen corpus review | NOT TESTED | This local automation revalidation does not establish ANX compatibility, extraction completeness, or semantic content validity. |

Full commands, limitations, and SHA-256 digests for all four aggregate reports are in [SQL-AUTO-08 current-release evidence](evidence/sql-automation/SQL-AUTO-08-current-release-20261127.md). Raw email content is not in Git.

## Temporary artifact archive — 2026-11-27

**Task / owner:** Move inactive Arch Notebook build and acceptance outputs off the nearly full `/tmp`; implementation session.
**Destination:** `smb://archive.local/Research/EVIDENCE/arch_notebook/tmp-archive-2026-11-27/`.

| Gate | Result | Evidence |
| --- | --- | --- |
| SMB copy and checksum comparison | PASS | 3,195,663,365 bytes transferred; `rsync -rc --dry-run --itemize-changes` reported no file differences. |
| Local cleanup | PASS | Only the archived Arch Notebook directories/reports were removed; `/tmp/arch-notebook-target` was retained. `/tmp` availability increased from 417 MB to 3.0 GB. |
| Other project files | NOT TOUCHED | No Room 237, Airspy, or other project artifacts were moved. |

See [TMP-ARCHIVE-01 evidence](evidence/TMP-ARCHIVE-01-linux.md) and the archive share's `README.txt` manifest. The remaining `/tmp` usage includes the active canonical Rust build target and unrelated files.

## Nested chart-legend conditions and political email QA source research — 2026-11-28

## Local Qwen structural review and political email QA inputs — 2026-11-28

The retained Clinton SQLite case was reviewed with a local Qwen 3.5 9B model using aggregate counts only. The prompt excluded email text and identity fields. The model repeated supplied totals and identified date gaps and represented-page coverage as checks. Its explanation that review flags reflected “low confidence or unreviewed status” was unsupported by the provided metrics and is explicitly rejected. This does not establish semantic accuracy, PDF completeness, or human review. See [CLINTON-QWEN-STRUCTURAL-01](evidence/sql-automation/CLINTON-QWEN-STRUCTURAL-01-20261128.md).

Separately, 11 base official-source PDFs plus two recent NARA supplements are staged outside Git. Their checksums pass; `pdfinfo` reports 2,850 pages total. One supplement triggers a Poppler optional-content warning but passed a full Ghostscript parse and first-page render. The current TRANS/Eagle-Eye accepted pipeline cannot process this public-record bundle: no PDF-to-page worker is installed, and TRANS's accepted OCR adapter is a placeholder stub. Arch Notebook PDF ingestion and Qwen analysis of this bundle have not run. See the [processing follow-up](evidence/political-email-pdf-processing-followup-2026-11-28.md) and [original intake](evidence/political-email-qa-inputs-2026-11-28.md).


The chart legend supports bounded nested/mixed AND/OR conditions (four group levels, eight leaves), with an optional case field and legacy flat rules retained. Focused tests pass (3/3). The current source also passes the full offline suite (78 passed, 0 failed, 3 ignored), locked offline check, formatting, debug build, and optimized release build; see [SQL-AUTO-11](evidence/sql-automation/SQL-AUTO-11-current-source-clinton-20261128.md). Native editor interaction, IBM compatibility, and exact-head CI remain pending. The repository includes a ten-source public-record candidate manifest. Separately, 13 source PDFs are staged outside Git but remain unprocessed because the approved TRANS/Eagle-Eye PDF path is unavailable. See [nested-condition evidence](evidence/vector-ui/CHART-LEGEND-NESTED-CONDITIONS-01-20261128.md), [QA source manifest](political-email-qa-sources.md), and [PDF processing follow-up](evidence/political-email-pdf-processing-followup-2026-11-28.md).

Historical remote-base revalidation on `codex/ui-00-chart-nested-conditions`, derived from remote head `d4257d48`, passed the full test suite (78 passed, 0 failed, 3 ignored), locked offline check, debug build, formatting, and diff validation. The isolated optimized release attempt lost its Cargo process before returning an exit status at that checkpoint. This limitation is superseded by the later successful optimized build from source `5bb4593`; see [SQL-AUTO-11](evidence/sql-automation/SQL-AUTO-11-current-source-clinton-20261128.md). The earlier corrupted blob remains in the old branch history; the restored-source candidate preserves a complete Rust source tree.

## Current Linux user installation — 2026-11-28

The optimized Rust release from product source `5bb4593` is installed in the current user's application directories. The isolated installer acceptance passed; the installed executable, application-menu entry, Desktop shortcut, SVG/PNG graphics, SQL automation helper, and case MIME file were checked against their release/package sources. Native launch, rendering, pointer/keyboard interaction, live scheduling, public release, and IBM compatibility remain separate gates. See [LOCAL-INSTALL-12](evidence/deployment/LOCAL-INSTALL-12-linux.md).

## Quick open case actions — 2026-11-28

**Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-nested-conditions` / `0291865fd639de1656e94f86792902df978cb0e3`.

Quick open exposes Open case and New case. Both reuse the existing unsaved-change confirmation before replacing current work. Chart remains the first result, preserving Enter with an empty search.

| Gate | Result | Evidence |
| --- | --- | --- |
| Command search and unsaved-change confirmation | PASS | `quick_command_search_matches_labels_and_keywords_case_insensitively`, `quick_case_actions_keep_the_unsaved_change_confirmation`, `quick_open_case_commands_reach_unsaved_change_confirmation`, and `quick_open_enter_without_a_query_keeps_chart_as_the_default` passed. Headless UI tests click both case commands and send Enter with an empty search. |
| Full offline suite | PASS | 81 passed, 0 failed, 3 ignored. |
| Formatting, offline check, debug build | PASS | Locked offline check and formatting exited 0; the earlier debug build passed before its generated profile was cleared to free `/tmp` space. |
| Optimized release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0 in 2m58s; binary SHA-256 `8381bd57bdf84017377d92ef62916bde737e05ebe15dbca4b98317ec4b62ed27`. Earlier attempts were interrupted before a result. |
| Native visual / pointer review | NOT RUN | Native display access unavailable in this session. |

Details: [UI-QUICK-OPEN-CASE-ACTIONS-01](evidence/vector-ui/UI-QUICK-OPEN-CASE-ACTIONS-01-20261128.md).

## Current-user XML/XSLT release installation — 2026-11-28

The release with the desktop XML/XSLT import workflow is installed in the current user's launcher. The executable, menu/Desktop shortcuts, icon graphics, case MIME definition, and SQL automation helper are synchronized with the tested repository assets.

| Gate | Result | Evidence |
| --- | --- | --- |
| Isolated installer acceptance | PASS | Binary, menu/Desktop entries, graphics, helper, MIME definition, and version checks passed. |
| Current-user install and artifact matching | PASS | Installed binary hash matches release; menu and Desktop entries validate and are identical; SVG/PNG and helper/MIME hashes match repository sources. |
| Native XML/XSLT menu interaction and IBM validation | NOT TESTED | This installation acceptance does not exercise native input or vendor compatibility. |

Exact hashes, commands, and limits: [LOCAL-INSTALL-14](evidence/deployment/LOCAL-INSTALL-14-linux.md).

## Primary Clinton corpus QA on the XML/XSLT desktop release — 2026-11-28

The latest installed Rust release was rechecked against the real Clinton corpus case as the primary SQL regression workload.

| Gate | Result | Evidence |
| --- | --- | --- |
| Query-only saved-job validation and saved-run replay | PASS | 1 saved job passed; 6,173 rows; 0 skipped; 0 added/removed/changed deltas. |
| Default-redacted graph handoff | PASS | 2,813 entities rendered, 6,173 relationships, 643 communities, 291 proxy candidates, zero labels outside the SVG view, and default pseudonymization retained. |
| Source immutability | PASS | PDF, case, and SQLite hashes were unchanged. |
| PDF completeness, Qwen, human content review | NOT ESTABLISHED / NOT RUN | SQL replay and graph aggregation are not content-level transcription or analysis acceptance. |

Full command, artifact hashes, and scope: [CLINTON-CORPUS-QA-05](evidence/sql-automation/CLINTON-CORPUS-QA-05-installed-release-20261128.md).

## Current installed release SQL automation / scale / scheduler recheck — 2026-11-28

The current per-user Rust release passed the synthetic SQL automation/recovery, 7,121-row joined-header scale, and Linux schedule/generated-runner acceptance harnesses. All scheduler operations ran against an isolated `systemctl` stub; live dispatch is not established.

| Gate | Result | Evidence |
| --- | --- | --- |
| SQL capture, diagnosis, failure retention, repair, and batch recovery | PASS | [SQL-AUTO-11](evidence/sql-automation/SQL-AUTO-11-current-installed-release-20261128.md); [aggregate JSON](evidence/sql-automation/artifacts/SQL-AUTO-11-current-installed-release-20261128.json), SHA-256 `a348b239e8e67481e819724a2da908e4a6944fac0efb2ac8cc51f655c5a02713`. |
| 7,121-row joined-header workload | PASS | 7,121 records captured, 1,500 endpoint entities, 0 skipped. [Aggregate JSON](evidence/sql-automation/artifacts/SQL-SCALE-02-current-installed-release-20261128.json), SHA-256 `1c6dfde1f7a252238cc1abed3be279eb2897cb59322c838acc762bee9a0d04ea`. |
| Five-cadence Linux scheduler harness | PASS, stub-scoped | Dynamic window, generated invocation, status variants, and case isolation passed. [Aggregate JSON](evidence/sql-automation/artifacts/SQL-SCHED-11-current-installed-release-20261128.json), SHA-256 `481da061d8e5cb94e8e0ec796b4c97e28d544cc9c6e3be6b95c93a3079d7645c`. |
| Live systemd dispatch and reboot recovery | NOT TESTED | No live timer delivery was observed. |

## Current-user Quick open release installation — 2026-11-28

The optimized release containing searchable Open case and New case actions is installed for the current Linux user. Release build, isolated installer acceptance, installed executable hash, matching desktop launchers, icon assets, MIME definition, and SQL helper all pass. Native display interaction and IBM validation remain open. Exact hashes and commands: [LOCAL-INSTALL-13](evidence/deployment/LOCAL-INSTALL-13-linux.md).

## Political-record PDF text-layer inventory — 2026-11-28

A read-only Poppler diagnostic succeeded on all 13 staged PDFs. It emitted 5,579,267 bytes and 2,560 nonempty form-feed-delimited text segments; the combined 2,842 segments do not align to the 2,850 pages reported by `pdfinfo` (the Mark Foley report yields 23 segments for 31 pages). No extracted text was persisted. This is not TRANS processing, page-complete extraction, email completeness, or Qwen analysis. Accepted Eagle-Eye `main` has an optional local visual backend with a 10-page default cap, but it emits bounded notice-specific observations and requires optional Python modules absent from this host; the pointer-only page worker still requires a separately approved page manifest. TRANS `main` retains its `ocr_stub`, so the political-record handoff remains blocked. See [text-layer inventory](evidence/political-email-text-layer-inventory-2026-11-28.md) and [pipeline follow-up](evidence/political-email-pdf-processing-followup-2026-11-28.md).

## Current installed-release Clinton corpus QA — 2026-11-28

The current per-user Rust release (SHA-256 `8381bd57…4b62ed27`) passes the primary Clinton case's read-only SQL preflight and saved-run replay: 6,173 records, zero skipped, zero deltas. Its default-redacted packet covers all 2,813 entities, keeps every label inside the SVG viewport, and preserves private output modes. PDF extraction completeness, Qwen analysis, and human review are separate and remain unestablished. Full source hashes, output digest, command, and counts: [CLINTON-CORPUS-QA-04](evidence/sql-automation/CLINTON-CORPUS-QA-04-installed-release-current.md).


## Recent political email and messaging-record case acquisition — 2026-11-28

A separate local QA package now contains ten official NARA case-letter PDFs (21 pages) about email, messaging, and federal-record retention, with one 2020 comparator. It includes active allegations and one resolved negative/control case; case status is stated as listed by NARA, not as a criminal conclusion. PDF inventory, text-layer diagnostic, and checksums pass. The USDA index/PDF ID mismatch is flagged. No PDFs or extracted full text entered Git.

| Gate | Result | Evidence |
| --- | --- | --- |
| Acquisition, `pdfinfo`, text-layer diagnostic, SHA-256 | PASS | [political email/messaging acquisition QA](evidence/political-email-messaging-case-acquisition-20261128.md) |
| TRANS transcription, Eagle-Eye evidence, Qwen analysis | BLOCKED / NOT RUN | Accepted general-purpose PDF processing executables are unavailable; local Ollama endpoint is inaccessible. |
| Arch Notebook raw-PDF ingestion | NOT RUN | Current product does not ingest raw PDFs. |

Source PDFs remain local under `/home/sysadmin/TRANSCRIBE-QA/political-email-cases-2026-11-28/recent-record-cases/`; they are not committed or copied to the archive share. The Clinton SQLite QA remains a separate workflow.


## Political records ingestion and Arch Notebook presentation QA — 2026-11-28

The prior acquisition status is superseded: TRANS processing, constrained Qwen inference, and a candidate-only Arch Notebook handoff have now been exercised over ten public NARA case letters. All ten TRANS candidates attached to source entities with human review required; the notebook exported ten isolates and zero relationships. Qwen structure and source quote checks admitted five candidate outputs for presentation, held out four quote-mismatch outputs and one truncated response, and did not import model-suggested links. A fresh Qwen rerun is blocked by local Ollama endpoint unavailability. This does not validate the Clinton corpus path, IBM interoperability, semantic accuracy, or human approval. Full evidence and limits: [political records ingestion and notebook QA](evidence/political-records-qa-20261128.md).

## Most connected contact-map action layout — 2026-11-28

The native 1440×900 chart review showed the ranked `Map contacts · <name>` control clipping against the analysis-pane edge and obscuring long entity labels. The action is now compact, labels reserve space for the action/count, and full entity labels remain available on hover. Full Rust suite (81 passed, 0 failed, 3 ignored), locked offline check/debug/release builds, formatting, isolated launcher/icon/MIME acceptance, current-user installation, and installed-release SQL automation harness pass. Before/after screenshots show the default-width native render; the 240 px minimum-width check is headless and passes. Pointer interaction and the narrow-width native screenshot remain open. See [ANALYSIS-MAP-CONTACTS-01](evidence/vector-ui/ANALYSIS-MAP-CONTACTS-01-20261128.md). This does not establish IBM visual or schema compatibility.

## Linux SQL timer dispatch status — 2026-11-28

The Linux per-case **Check schedule** output now reports systemd's last timer dispatch and next dispatch time when available. The output explicitly distinguishes a timer dispatch from SQL job success; the Automation guide directs analysts to batch history for capture outcomes.

| Gate | Result | Evidence |
| --- | --- | --- |
| Shell/Python syntax and diff check | PASS | `bash -n scripts/install-linux-automation.sh`, `python3 -m py_compile scripts/test-linux-sql-scheduler-acceptance.py`, and `git diff --check` exited 0. |
| Linux schedule and SQL harness | PASS, stub-scoped | Five cadences, calendar parsing, query preflight, generated service invocation, SQL snapshot, CSV export, failure handling, case isolation, and last/next timer-status output passed. The synthetic `systemctl` stub does not establish live timer delivery. |
| Current-user helper synchronization | PASS | Installed automation helper hash matches the repository script; prior helper retained for rollback. See [SQL-SCHED-10](evidence/sql-automation/SQL-SCHED-10-timer-status-linux.md). |
| Live user-manager dispatch | BLOCKED | User-systemd bus access is denied in this sandbox; no timer was installed or fired. |
| IBM compatibility | NOT TESTED | No IBM app or schema was involved. |

Exact command, runner binary hash, and JSON digest: [SQL-SCHED-10](evidence/sql-automation/SQL-SCHED-10-timer-status-linux.md).

## Trusted XML/XSLT adaptation into ANX — 2026-11-28

The command-line importer can now run a trusted XSLT stylesheet over XML and import the transformed document through Arch Notebook's existing strict ANX parser. This supports organization-specific XML mappings while keeping transformation and case persistence explicit in scripts.

| Gate | Result | Evidence |
| --- | --- | --- |
| Formatting, diff, offline Rust suite | PASS | Formatting and diff checks exited 0; 81 tests passed, 0 failed, 3 ignored. |
| Locked offline check and debug/release builds | PASS | All exited 0. Debug binary SHA-256 `c463117b92e774dfd2d7e019db4fd026bee04d473b91aadafa2c479a75a62748`; release binary SHA-256 `f5ff73d6676c565499a39b879dbb245a0e5813acbe217ec1430550ee60a39196`. |
| XSLT import harness on debug and release | PASS | Both imported 8 entities and 8 relationships; UTF-8/UTF-16 source DTD and stylesheet DTD were rejected without changing the case. |
| IBM ANX schema/application | NOT TESTED | No vendor schema or IBM application was used. |
| Operational SQL schedule integration | NOT TESTED | No scheduler was configured to invoke the new CLI. |

Implementation detail, exact commands, file limits, fixture digest, and remaining validation: [XML-XSLT-IMPORT-01](evidence/anx/XML-XSLT-IMPORT-01-linux-20261128.md).

## Desktop XML/XSLT import workflow — 2026-11-28

The **Import…** menu now accepts an XML input and trusted stylesheet path. Transform and ANX parsing run in the background; a result is discarded if the analyst switches charts before completion.

| Gate | Result | Evidence |
| --- | --- | --- |
| Offline Rust suite | PASS | 82 passed, 0 failed, 3 ignored; includes background worker DTD error routing. |
| Locked offline check and debug/release builds | PASS | All exited 0. Final debug SHA-256 `5013c713fdac357dd2bdb691a1b1b62b4ee9edd9ed1a07a426efb8651b6b4cba`; final release SHA-256 `4f37a78a91230b8c1aeafc16d0d3aedc18638dd83bb56eaf2172d504cf24c08d`. |
| XSLT import acceptance on debug/release | PASS | Each imported 8 entities and 8 relationships and rejected DTD cases without changing the destination case. |
| Native UI interaction and IBM schema/application | NOT TESTED | Headless worker and CLI evidence do not prove pointer/keyboard interaction or IBM compatibility. |

Evidence and next gates: [XML-XSLT-IMPORT-02](evidence/anx/XML-XSLT-IMPORT-02-desktop-20261128.md).

## SQL selected-entity seeded query review — 2026-11-28 (search and bulk selection follow-up)

SQL Review binds an active-chart SQL identity to a one-off parameter, pages through read-only query results 200 source rows at a time, carries selections across pages, displays bounded transient unmapped query fields, filters each page, and bulk-selects only visible matching rows. It adds explicitly selected relationships without recording a snapshot or automation run. Paging requires deterministic SQL ordering and preserves saved automation configuration.

| Gate | Result | Evidence |
| --- | --- | --- |
| Formatting, diff, and full offline Rust suite | PASS | `cargo fmt --all`, `git diff --check`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exit 0; 89 passed, 0 failed, 3 ignored. |
| Locked offline check | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Locked offline release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0; binary SHA-256 `dbc6b600c0ef79a3e0876846b2a43fa314bf9c686826c6a02fc3800632c0003f`. |
| Per-user launcher/icon install acceptance | PASS | Installer and isolated acceptance exited 0; desktop menu and shortcut match, SVG/PNG and MIME assets match, and installed executable matches release. |
| Native interaction and IBM compatibility | NOT TESTED | Headless state regression does not prove native pointer interaction or i2 Analyze integration. |

Details: [SQL-SEED-01](evidence/sql-automation/SQL-SEED-01-one-off-parameter-20261128.md).

## Multi-entity SQL seed review — 2026-11-28

SQL Review can bind a shared SQL identity source from multiple selected entities as a JSON parameter for SQLite `json_each`. It omits sources where any selected entity has no identity or more than one distinct identity, and switches the current query path to the selected source database when binding. Single-entity binding now follows the same path rule. The binding stays in the one-off review draft; the analyst still validates results and selects relationships to add.

| Gate | Result | Evidence |
| --- | --- | --- |
| Shared identity source, JSON parameter, SQLite expansion, and SQL Review event | PASS | [SQL-SEED-02](evidence/sql-automation/SQL-SEED-02-multi-entity-json-seed-20261128.md). |
| Formatting, diff, offline check, full Rust suite | PASS | Formatting and diff checks exited 0; offline check exited 0; 94 passed, 0 failed, 3 ignored. |
| Locked offline release build | PASS | Exit 0; release SHA-256 `8d1c1087cdbe126785ba6ca57721a3f2f704ab08e537d9dcd0a0b11d4b739c04`. |
| Current-user launcher and graphics synchronization | PASS | Installed binary matches release; menu and desktop shortcut, SVG/PNG icons, MIME definition, and SQL helper match package assets. Isolated installer acceptance passed. |
| IBM connector/schema and native pointer interaction | NOT TESTED | This local SQLite seed helper is not an i2 Analyze external-search connector; headless UI tests do not establish native interaction. |

## Commented SQL query-plan inspection — 2026-11-28

Read-only query-plan inspection normalizes a terminal semicolon and comments with the SQL-aware statement parser used by seeded paging. The SQLite regression binds a typed parameter, checks returned planner steps, and verifies source bytes remain unchanged.

| Gate | Result | Evidence |
| --- | --- | --- |
| Formatting, diff, locked check, full Rust suite | PASS | Formatting and diff checks exit 0; offline check exits 0; 90 passed, 0 failed, 3 ignored. |
| Parameterized commented-query plan and source read-only check | PASS | [SQL-PLAN-01](evidence/sql-automation/SQL-PLAN-01-commented-readonly-query-20261128.md). |
| Locked offline release build | PASS | Exit 0; release SHA-256 `52c2a22db4b6c762bb88103137bab7b8419bce8189d7765464c3727d4efd1339`. |
| Launcher and graphics synchronization | PASS | Current-user installation and isolated acceptance passed; release binary, desktop entry, SVG/PNG icons, MIME entry, and SQL helper match package assets. |
| Native window and interaction | PARTIAL | X11 window mapped at 1440×900, but the display became unavailable to screenshot/input tools; visual and pointer/keyboard interaction remain unverified. |
