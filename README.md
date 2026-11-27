# Arch Notebook

[![Build](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml)

[![Linux local build: passing](https://img.shields.io/badge/Linux%20local%20build-passing-brightgreen)](docs/build-acceptance.md)

Arch Notebook is a native Rust desktop workbench for relationship analysis, source reconciliation, and longitudinal SQL investigations. It connects analytical charts to their underlying records and evidence, allowing an analyst to trace what changed between captures without modifying the source database.

Designed for local, repeatable investigations, it stores case files, snapshots, automation configurations, and run history on the analyst's machine. No hosted service or account is required.

## Analytical capabilities

| Workspace | Purpose |
| --- | --- |
| Chart | Build typed entity/link networks, inspect parallel relationships, search attributes and evidence, and save selection sets. |
| Timeline | Review dated events and restrict investigations to valid ISO-date intervals. |
| Connections | Inspect grouped relationships and individual records, including direction, confidence, notes and provenance. |
| SQL Review | Discover SQLite schemas, preview read-only queries, capture immutable results, and compare historical snapshots. |
| Automation | Preflight and execute repeatable captures, review exceptions and enforce configurable capture limits. |
| Entities | Review source identities, aliases, attributes and potential duplicates before merging. |
| Sources | Trace imported records to their origin, highlight source-specific networks and navigate to SQL captures or jobs. |

Named chart tabs retain their own viewports. The searchable Quick open palette (Ctrl/Command+K) jumps to analysis workspaces and common chart actions. Analysts can apply saved CSV/TSV import designs, open a saved case from the Linux launcher, and import approved downstream document handoffs into a separate evidence chart.

Network analysis includes weakly connected groups, articulation entities (network bridges), connectivity rankings, shortest paths constrained by direction, timeline range or chronological order, and geodesic hop layers around a selected entity. The detected-community view aggregates cross-community ties and ranks bridge entities by distinct communities contacted, then distinct neighboring entities; repeated relationship records are counted separately. The origin-centered contact map groups connected entities within each hop layer, draws ties between groups, summarizes record counts, marks groups containing ranked cross-group proxies, and drills back to exact relationship records. On large cases the map is sampled and bounded while the full groups and records remain available below it. These are read-only analysis views; the main chart retains its original entities and links. Each graph view states how link direction and weights affect its result; ordinary entity/link data does not imply an algebraic group operation or Cayley graph. Saved paths retain investigation criteria and recompute against the current case rather than preserving stale routes.

## Development status — 2026-11-26

Version 0.1.0 is under active development. The [2026-11-23 implementation baseline](https://github.com/pskeffington/arch_notebook/commit/2dadb09570e2a871489b2146f9ad1e1410b70929) passed [all five platform CI build/package jobs](https://github.com/pskeffington/arch_notebook/actions/runs/45269810987): Arch Linux x86_64, generic Linux x86_64 and ARM64, and macOS Intel and Apple Silicon. No GitHub Release is published.

| Capability | State |
| --- | --- |
| Local entity/link chart, timelines, paths and case persistence | Implemented |
| Read-only SQLite discovery, immutable captures and longitudinal comparisons | Implemented |
| Endpoint and relationship property changes, stable identities and schema drift signal | Implemented |
| Scheduled captures, preflight and bounded run history | Implemented |
| CSV/TSV, GraphML and KML interchange | Implemented |
| Historical SQL capture export to chart-import CSV/TSV, via desktop and CLI | Implemented |
| ANX XML chart import/export | Implemented; internal round-trip and DTD rejection pass; IBM XSD and application validation pending |
| Deterministic chart fixtures and automated chart regressions | Implemented on `codex/ui-00-chart-baseline` |
| Interactive host acceptance and external IBM import validation | Pending evidence |
| Published release and macOS notarization | Pending owner approval / signing evidence |

Recent work added index and SQL-definition inspection, schema drift reporting in SQL Review and automation, KML placemark import, chart-import exports of selected historical captures, and ANX internal round-trip acceptance that preserves supported record IDs, repeated links, positions, labels, and whitespace. Desktop ANX XML parsing now runs on a background worker and rejects results if the active chart changed before parsing completed; CLI imports remain synchronous for automation. The importer rejects DTD-bearing files before modifying a case. SQL automation acceptance now includes joined header-shaped data with 7,121 records, composite identities, nullable attributes, and a two-snapshot comparison; see the [SQL scale acceptance evidence](docs/evidence/sql-automation/SQL-SCALE-01-joined-header-linux-620e733.md). See also [ANX acceptance evidence](docs/evidence/anx/ANX-ROUNDTRIP-02-xxe-linux-1014ba5.md), [SQL ingestion and interchange](docs/sql-ingestion.md), and the [Analyst's Notebook compatibility map](docs/analyst-notebook-parity.md) for details. SQLite schema_version is a drift signal, not a complete schema fingerprint. Validate CSV/TSV and ANX files in IBM's application before claiming external interoperability.

**Linux host progress (2026-11-25):** The operator reports that automatic builds appear to be working on the Linux box. This is a preliminary host observation, separate from CI success; the host architecture, build logs, installed version and runtime/scheduler acceptance remain to be recorded.

**Local Linux user install (2026-11-26):** Rebuilt the feature-branch release and installed it with the documented user installer. The app menu entry, desktop shortcut, `.case.json` MIME handler, executable, and SVG/PNG icons now match the current build and repository assets. A later native-window capture rendered the fictional UI-00 chart; see [local install evidence](docs/evidence/deployment/LOCAL-INSTALL-01-linux.md) and the [render-only UI-00 recheck](docs/evidence/vector-ui/UI-00-native-render-20261126.md).

**Synchronized branch install (2026-11-26):** rebuilt source `e868146` and refreshed the local executable, launcher, case-file association, and desktop icons. The installed binary matches the optimized build hash; the desktop database now resolves `.case.json` files to Arch Notebook. Current-release synthetic SQL automation and 7,121-record scale checks pass; native UI and live scheduler dispatch remain unverified. See [current install evidence](docs/evidence/deployment/LOCAL-INSTALL-02-linux.md) and [SQL recheck](docs/evidence/sql-automation/SQL-AUTO-06-current-release-linux.md).

**Current feature-branch install (2026-11-26):** rebuilt and installed source `4c132cf` to the current user's launcher. The installed binary hash matches the release artifact; the desktop shortcut, menu entry, MIME association, automation helper, SVG, and PNG are synchronized. Native interaction remains unverified because the Wayland display is inaccessible. See [LOCAL-INSTALL-03](docs/evidence/deployment/LOCAL-INSTALL-03-linux.md).

**Current candidate install (2026-11-27):** source `280f5f0` is installed in the current user launcher; the executable matches release SHA-256 `4bc1d5b7…`, the menu and Desktop shortcuts match, and the installed SVG/PNG and SQL automation helper match repository assets. The case MIME handler resolves to Arch Notebook. A 1440×900 native window render of the fictional correctness chart is captured; pointer/keyboard input remains unavailable, so this is not interactive acceptance. See [LOCAL-INSTALL-05](docs/evidence/deployment/LOCAL-INSTALL-05-linux.md).

**Analysis filter layout refinement (2026-11-27):** source `1dd4cdc` stacks the Most connected ranking controls and safely shortens long relationship labels while keeping the full value available on hover. The 60-test offline suite and locked check/debug/release builds pass; the release is installed in the current user launcher with synchronized graphics. Exact-candidate native window launch is blocked by X11 access. See [UI-ANALYSIS-FILTERS-01](docs/evidence/vector-ui/UI-ANALYSIS-FILTERS-01-linux.md).

**Proxy-focus install (2026-11-26):** the launcher runs product source `e27ad5e`, including the proxy-row focus action; verification source `c406d81` adds a headless pointer-drag regression confirming that offscreen relationship labels are omitted after panning. The regression received an independent review recorded on the current branch. The installed executable matches release hash `4a373a…`, and desktop/menu entries, MIME handler, automation helper, and icon assets are synchronized. Headless UI coverage verifies proxy focus pans to the selected bridge; native review remains unavailable. See [LOCAL-INSTALL-04](docs/evidence/deployment/LOCAL-INSTALL-04-linux.md), [proxy evidence](docs/evidence/vector-ui/COMMUNITY-PROXY-01-linux.md), and [viewport culling evidence](docs/evidence/vector-ui/G4-VIEWPORT-CULLING-01-linux.md).

**SQL automation recheck (2026-11-27):** the installed release passed fresh synthetic capture, failure retention, schema diagnosis and reviewed repair, resumable batches, CSV/TSV round trips, and a 7,121-record joined-header workload with zero skipped rows. Generated Linux and macOS scheduler runners also passed in isolated stub harnesses; live timer dispatch remains unverified. The source database remained unchanged during captures. Native Automation UI, production schemas, and IBM interoperability remain separate open gates; see [SQL-AUTO-07](docs/evidence/sql-automation/SQL-AUTO-07-current-installed-release-linux.md) and [SQL-SCHED-07](docs/evidence/sql-automation/SQL-SCHED-07-current-installed-release-linux.md).

Next acceptance pass: finish UI-00 native interaction acceptance. The installed app renders the fictional chart, navigator, and geodesic panel; pointer/keyboard selection, repeated interaction traces, and input-driven responsiveness checks remain pending. A searchable **Fit chart** command and Ctrl/Command+0 now reframe all entities after zooming or panning; current-candidate native keyboard review remains pending ([UI-NAV-FIT-01](docs/evidence/vector-ui/UI-NAV-FIT-01-linux.md)). Scripted CLI acceptance passes fictional SQLite baseline/current capture, override isolation, run history, deltas, schema-stability rejection, failed-run retention, and comparison export. Schema adaptation now persists the missing-column diagnosis as structured run data, retains the failed run, and supports an explicit reviewed query repair followed by a successful capture; CSV and TSV chart exports also round-trip through saved import designs while preserving record and source-scoped endpoint identity and custom properties. Native Automation UI review, live scheduled execution, and chart navigation remain pending; IBM i2 schema/application validation is still required for external interoperability claims. See the [render-only UI-00 evidence](docs/evidence/vector-ui/UI-00-native-render-20261126.md), [SQL schema and failed-run acceptance](docs/evidence/sql-automation/SQL-CAPTURE-03-schema-and-failure-linux-d5c07ec.md), [chart CSV/TSV round-trip evidence](docs/evidence/sql-automation/SQL-CAPTURE-04-chart-csv-tsv-linux-b365fdc.md), [schema adaptation and reviewed repair evidence](docs/evidence/sql-automation/SQL-CAPTURE-05-schema-adaptation-linux-86e2be5.md), and [structured diagnosis acceptance](docs/evidence/sql-automation/SQL-CAPTURE-06-structured-adaptation-linux-2ed5163.md). CI success establishes build/package success, not interactive or production acceptance.

The SQL CLI acceptance now also exercises interrupted batch recovery: it saves the first successful job checkpoint, repairs a missing view in a disposable SQLite fixture, resumes the same batch, confirms the completed job is skipped, and rejects a changed-parameter resume without changing the case. See [SQL batch recovery evidence](docs/evidence/sql-automation/SQL-CAPTURE-07-batch-resume-linux.md). This validates CLI recovery only; native scheduler restart, sleep/reboot recovery, and interactive cancellation remain pending.

The Chart analysis sidebar now includes an on-demand, read-only **Analysis Method Card** with active-chart record counts and definitions for the network analyses. Its copied summary omits names, IDs, source identifiers, and evidence. The 25-test offline suite and Linux release build pass; the new card is installed in the local launcher. Pointer-driven review of the card remains blocked, and this is not evidence of IBM visual parity. See [Analysis Method Card acceptance](docs/evidence/vector-ui/UI-ANALYSIS-CARD-01-linux.md) and the [phased social graphing roadmap](docs/roadmaps/social-network-graphing-roadmap.md).

The chart's known parallel-link overlap is repaired: lane assignment accounts for canonical endpoint direction, and the fixture verifies that a pointer on each reversed/parallel route resolves to its exact relationship record. Native interaction and visual review remain pending; see [UI-01 lane and selection acceptance](docs/evidence/vector-ui/UI-01-parallel-lanes-linux.md).

**UI-01 route geometry (2026-11-26):** self-links now render as visible cubic loops; distinct relationship records also receive separate routes when their endpoints occupy the same chart position. Labels, directed arrowheads, and picking use the route geometry. The 28-test offline suite and Linux release build pass. Native interaction and IBM visual parity remain unverified; see the [self-loop evidence](docs/evidence/vector-ui/UI-01-self-loop-linux.md) and [coincident-route evidence](docs/evidence/vector-ui/UI-01-coincident-routes-linux-fae574d.md).

The Linux scheduler harness now invokes a generated service `ExecStart` against its own fresh synthetic SQLite case and verifies the scheduled-trigger run, saved snapshot, and chart CSV. The host systemd user bus denies access, so actual timer dispatch remains unverified; see the [scheduler invocation follow-up](docs/evidence/sql-automation/SQL-SCHED-01-linux.md).

The scheduler harness also checks a rolling previous-UTC-month SQL query through that generated service command, including the inclusive start and exclusive end boundary. Scheduled parameter values remain fixed; SQLite date functions provide rolling time windows. See [SQL-SCHED-03 evidence](docs/evidence/sql-automation/SQL-SCHED-03-dynamic-window-linux.md).

The isolated macOS LaunchAgent harness now runs the bundled automation runner against the same rolling-window case and checks the scheduled trigger, boundary rows, CSV output, and unchanged SQLite source. The harness uses a temporary `launchctl` stub on Linux; live macOS dispatch is still pending. See [SQL-SCHED-04 evidence](docs/evidence/sql-automation/SQL-SCHED-04-macos-window-linux-harness.md).

The current installed Rust release was also rerun through the SQL scale, automation/recovery, Linux generated-service, and macOS runner harnesses. All four passed with synthetic SQLite fixtures. The scheduler checks use stubs/simulations and do not establish live timer dispatch; see [SQL-AUTO-02 current-release evidence](docs/evidence/sql-automation/SQL-AUTO-02-current-release-linux.md).

Both rolling-window scheduler harnesses were rerun against the current installed release binary: Linux generated-service invocation and the bundled macOS runner simulation each included only the expected prior-month boundary row. The macOS simulation also confirmed the SQLite source remained unchanged. These checks still do not exercise live systemd or macOS `launchd` dispatch; see [SQL-SCHED-05 evidence](docs/evidence/sql-automation/SQL-SCHED-05-current-branch-window-linux.md).

The Linux systemd and macOS LaunchAgent installer/runner lifecycle harnesses were rerun against the current release. Five cadences, preflight rejection, rollback/removal behavior, special-character paths, scheduled capture and CSV export passed; host dispatch remains unverified because scheduler commands were stubbed. See [SQL-SCHED-06 evidence](docs/evidence/sql-automation/SQL-SCHED-06-current-branch-linux.md).

**Current-release SQL recheck (2026-11-26):** the release built from product source `fae574d` passed SQL automation/recovery, 7,121-row joined-header scale, Linux service lifecycle/invocation, and macOS runner simulation harnesses. Scheduler commands were stubbed; live timer dispatch, native Automation UI, and IBM validation remain open. See [SQL-AUTO-04 evidence](docs/evidence/sql-automation/SQL-AUTO-04-current-release-linux.md).

**Synchronized-branch SQL recheck (2026-11-26):** the current release passed the SQL capture/recovery, 7,121-row scale, Linux generated-service, and macOS runner harnesses. Scheduler commands were stubbed; live dispatch remains unverified. The desktop compositor is now reachable by diagnostic tools, but the notebook still exits during Winit initialization, so native Automation interaction remains blocked. See [SQL-AUTO-05 evidence](docs/evidence/sql-automation/SQL-AUTO-05-current-branch-linux.md).

**Current-head TRANSCRIBE candidate recheck (2026-11-26):** the release built from `561d52f` revalidated the saved SQL query and latest run against the retained local SQLite source: 6,173 records, no skipped rows, and no saved-capture deltas. The case and SQLite hashes were unchanged. The job remains disabled; this does not verify source extraction completeness, email-level correctness, Qwen analysis, native Automation UI, or live scheduling. See [SQL-CANDIDATE-03](docs/evidence/sql-automation/SQL-CANDIDATE-03-current-head-readonly.md).

**G3 force-layout follow-up (2026-11-26):** Native review of the 30-entity fictional chart exposed a single-community overlap defect. The source fix removes centroid attraction and adds node/community repulsion; its new spread regression and all 46 Rust tests pass, as do the release build and refreshed 25-sample CPU benchmarks. Corrected arrange/restore interaction remains blocked because the desktop input/capture path stopped accepting connections; the previous failing layout capture is preserved as regression evidence. See [G3 evidence](docs/evidence/vector-ui/G3-community-force-benchmark-linux.md). The archive share is currently read-only, so temporary outputs have not been moved there.

**G3 baseline comparison (2026-11-26):** the production Connectivity arrangement and community force arrangement now have paired debug and release CPU comparisons. The release run covers 12 sparse, hub, disconnected, and dense charts from 1,000 to 10,000 entities, with 25 alternating-order samples per layout. Native arrange/restore interaction remains unverified; timings do not measure drawing or presentation. See [release comparison evidence](docs/evidence/vector-ui/G3-connectivity-vs-force-release-20261126.md).

The selected-entity **Query live** drillback now has regression coverage for source-scoped composite IDs across separate databases, read-only execution, unchanged saved snapshots, NULL/BLOB display, unsafe query rejection, and bounded results. These are headless lookup tests; native inspector interaction remains pending. See [SQL-DRILLBACK-01 evidence](docs/evidence/sql-automation/SQL-DRILLBACK-01-linux.md).

Desktop SQL automation now has a headless UI event regression for the actual **Cancel automation** button: it cancels a long second query, retains the earlier successful snapshot and batch checkpoint, and confirms the source database remains unchanged. This verifies the egui event path and worker persistence, not native pointer/visual acceptance. See [SQL-CANCEL-02 evidence](docs/evidence/sql-automation/SQL-CANCEL-02-headless-ui-linux.md) and the [worker-level record](docs/evidence/sql-automation/SQL-CANCEL-01-worker-linux.md).

**Vector UI-00 branch evidence:** `codex/ui-00-chart-baseline` adds deterministic fictional workloads, chart geometry and selected-record identity regressions, opt-in local timings from the application chart path, SQL automation acceptance, isolated Linux systemd and macOS LaunchAgent installer lifecycle checks, and supported ANX round-trip checks. The follow-up on `df23448` corrects the parallel-link regression to compare rendered routes after direction normalization; all 16 Rust tests now pass with no ignored tests. See the [reversed-link route evidence](docs/evidence/vector-ui/UI-00-reversed-link-routes-linux-df23448.md) and the [Quick open interaction record](docs/evidence/vector-ui/UI-QUICK-OPEN-01-linux.md). Earlier CI run 45723413554 passed SQL/ANX, both isolated scheduler acceptance suites, Linux x86_64/ARM64, Arch packaging, and macOS Intel/Apple Silicon on `986f3e8`; run 45709682338 passed the earlier Linux scheduler candidate on `1e5a86f`. The local release build passes, but native chart interaction and visual review remain pending; see the [UI-00 follow-up](docs/evidence/vector-ui/UI-00-followup-linux.md), [Wayland runtime evidence](docs/evidence/vector-ui/UI-00-linux-wayland-a84a938.md), [R1 source follow-up](docs/evidence/vector-ui/R1-linux-30de20a.md), [literature matrix](docs/research/vector-ui-literature-matrix.md), [Linux scheduler evidence](docs/evidence/sql-automation/SQL-SCHED-01-linux.md), [isolated macOS scheduler evidence](docs/evidence/sql-automation/SQL-SCHED-02-macos-isolated.md), and [ANX acceptance record](docs/evidence/anx/ANX-ROUNDTRIP-01-linux.md).

**UI-00 fixture recheck (2026-11-26):** the checked-in correctness chart and all four generated workload families are byte-deterministic locally, including the 10,000-node/100,000-link dense cap. Native baseline measurements remain blocked by unavailable display access, and CI has not run for the current feature candidate. See [fixture reproduction evidence](docs/evidence/vector-ui/UI-00-fixture-reproduction-linux.md).

**Social graph exploration gate update (2026-11-26):** the detected-community and cross-community workflow passes the 54-test offline suite and a scoped native pointer/wheel review on a disposable fictional chart. Native review detected six communities, selected a singleton, browsed a community pair's records, inspected an exact relationship, and confirmed both endpoints were focused on the chart. It also exercised hop-layer contact-map rendering. A narrow-pane clipping issue found in that review is addressed with compact actions and fixed-height metadata/endpoint rows; full text remains available on hover, and independent review plus the optimized offline build pass. Native G3 arrange/restore and G4 draw/presentation performance gates remain open. See the [phased roadmap](docs/roadmaps/vector-community-exploration-roadmap.md), [headless UI evidence](docs/evidence/vector-ui/SNG-DELIVERY-002-G1-G2-headless-ui-20261126.md), and [native UI evidence](docs/evidence/vector-ui/SNG-DELIVERY-002-G1-G2-native-ui-20261126.md).

**Chart-wide proxy and viewport-culling follow-up (2026-11-26):** detected communities now expose bridge entities ranked by distinct community contacts and distinct direct neighbors, with repeated records counted separately; headless UI coverage confirms the proxy list is reachable and displays the bridge. Chart link routes also skip paint submission when their conservative expanded bounds do not intersect the viewport. The release benchmark confirms offscreen culling on a panned synthetic chart and zero false drops in the neutral-panned fixtures. This records geometry cull counts, not paint-time savings; native visual review and renderer timing remain open. See [proxy evidence](docs/evidence/vector-ui/COMMUNITY-PROXY-01-linux.md) and [viewport-culling evidence](docs/evidence/vector-ui/G4-VIEWPORT-CULLING-01-linux.md).

An earlier capture showed “Not Responding” and compositor access was inconsistent. A later render-only capture confirmed the installed app displays the chart, navigator, and analysis sidebar. The UI-00 native baseline remains open for input, repeated interaction, and visual parity review; see the [latest render recheck](docs/evidence/vector-ui/UI-00-native-render-20261126.md) and the [earlier responsiveness record](docs/evidence/vector-ui/UI-00-linux-recheck-e44d485.md).

Entity-type chart layout now distributes type clusters across an aspect-aware grid and centers sparse clusters within their cells. The Rust regression and offline release build pass, and the installed Linux launcher matches the release binary; pointer and keyboard interaction remain unverified because this session cannot inject input events.

The chart layout menu also offers an experimental community-aware force arrangement with a configurable iteration count. Community detection and coordinate calculation run in a background worker; results are applied only if the active chart, source positions, pins, and relationship endpoints remain unchanged. Synthetic Linux CPU timing covers 1,000–10,000 entities, and a session-scoped restore action returns to pre-force positions; native visual review remains outstanding; see [G3 workload evidence](docs/evidence/vector-ui/G3-community-force-benchmark-linux.md).

The Analysis sidebar also measures unweighted geodesic hop layers from a selected entity and displays its direction/weight assumptions. ANX XML parsing in the desktop app runs in the background, while CLI import remains synchronous. The combined candidate passes Rust tests, release build, ANX round-trip/security checks, and current Linux/macOS-runner SQL scheduler harnesses; native GUI interaction and IBM schema/application validation remain open. See [UI-INGEST-NAV-01 evidence](docs/evidence/vector-ui/UI-INGEST-NAV-01-linux.md).

## Start here

1. [Install Arch Notebook](docs/build-and-deploy.md) or build from source.
2. Open a local case. Import relationship records or create entities and links directly in Chart.
3. In **SQL Review**, select a SQLite database and use **Build query from schema** or provide a read-only query returning `source` and `target`.
4. Validate the mapping, inspect skipped endpoints and choose a stable snapshot series. Capture the baseline.
5. Recapture the same series under comparable conditions and inspect removals, confidence declines, endpoint or relationship property changes, and schema drift.
6. Export the comparison or selected historical capture. For recurring collection, configure and preflight an Automation job.

### Reproducible example

Use the fictional relational fixture rather than operational data:

```sh
sqlite3 /tmp/relational-transfers.sqlite < archive/research.databases/relational_transfers.sql
arch-notebook --inspect-sqlite /tmp/relational-transfers.sqlite --json
```

Run from the repository root using an installed executable; alternatively invoke it through `cargo run --offline --`. The [SQL ingestion guide](docs/sql-ingestion.md) provides the joined transfer query and January/February parameter values for a two-period exercise. Use stable connection and endpoint identifiers to distinguish parallel events and reconcile renamed entities.

## Automated monitoring

Create a job from a validated query, review its configuration, enable it, then run it in the app, from the CLI, or through a systemd user timer or macOS LaunchAgent. Configure independent limits for connection counts, confidence, regressions, additions, property changes and skipped rows. Review run history for failures, partial data and unreviewed changes; a passing capture does not imply that no changes occurred.

See [Automated SQL ingestion](docs/automation.md) for configuration, preflight, scheduling and recovery.

## Data exchange

| Format | Supported use |
| --- | --- |
| CSV / TSV | Import and export relationship records, chart-import rows, snapshots, complete series and comparison reports. |
| GraphML | Exchange graph structures, attributes, styling and supported source-catalog metadata. |
| KML | Exchange geographic point and line relationships with supported coordinates and metadata. |
| JSON | Local case persistence, structured CLI results and investigation path packages. |
| IBM i2 Analyst's Notebook | CSV/TSV handoff via a receiving import specification and supported ANX XML interchange; external validation remains pending. |

GraphML and KML are interchange formats, not full substitutes for local case and snapshot history. External IBM import fidelity still requires testing in a receiving installation.

On Linux, the CLI can package a saved chart as `finding.md` plus a simplified SVG network overview for analyst handoff. Names, chart title, relationship labels, record IDs, and evidence reference strings are independently opt-in; source notes, evidence contents, and SQL connection details are omitted. The package is an Arch Notebook report handoff and does not claim IBM ANX compatibility. See [Notebook workflows](docs/notebook-workflows.md#export-an-analyst-finding-packet).

## Data integrity and boundaries

- SQLite sources open read-only; writable statements are rejected. Each capture records query results and schema version in one read transaction.
- Stable source IDs and connection IDs support longitudinal reconciliation. Ambiguous duplicate connection keys fail rather than silently merging records.
- SQL-managed properties remain distinct from analyst notes and manual chart data. Entity merges preserve original records and provenance.
- Automation and export operations use atomic writes where documented; case locking protects against concurrent automation writes.
- Schema version differences indicate potential structural change, not a complete schema fingerprint or proof of a change in meaning.

## Platforms

| Target | Distribution path |
| --- | --- |
| Arch Linux x86_64 | Native package and desktop launcher |
| Generic Linux x86_64 / ARM64 | Architecture-specific archive and user installer |
| macOS Apple Silicon / Intel | Native app bundle and zip; CI builds are unsigned |

See [Build and deployment](docs/build-and-deploy.md) for installation, scheduling, signing and checksum verification. Published release assets are not yet available.

## Guides

- [SQL snapshot ingestion](docs/sql-ingestion.md)
- [Automated SQL ingestion](docs/automation.md)
- [Notebook workflows](docs/notebook-workflows.md)
- [Social network graphing and delivery review](docs/research/social-network-graphing-and-delivery-review.md)
- [Analyst's Notebook compatibility map](docs/analyst-notebook-parity.md)
- [Local build acceptance record](docs/build-acceptance.md)
- [Build and deployment](docs/build-and-deploy.md)
- [Project maintenance](docs/project-reference.md)
- [Release process](RELEASING.md)
- [Archived research fixtures](archive/research.databases/README.md)

## Build from source

Rust 1.89 or newer is required. Run from the repository root:

```sh
cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline
```

The bundled examples are fictional. Arch Notebook does not connect to a service or transmit case data.
