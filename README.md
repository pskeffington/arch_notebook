Warning: truncated output (original token count: 11018)
Total output lines: 234

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

For chart readability, relationship labels use **Auto** by default: up to 36 visible routes show all labels; denser views show labels and attribute callouts for selected, path, or focused relationships. Label visibility does not change route rendering or picking; fully offscreen routes remain culled, and exact records remain in Connections. Choose **View… → Show all relationship labels** to override Auto for the current session.

## Development status — 2026-11-28

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

**Previous candidate install (2026-11-27):** source `280f5f0` is installed in the current user launcher; the executable matches release SHA-256 `4bc1d5b7…`, the menu and Desktop shortcuts match, and the installed SVG/PNG and SQL automation helper match repository assets. The case MIME handler resolves to Arch Notebook. A 1440×900 native window render of the fictional correctness chart is captured; pointer/keyboard input remains unavailable, so this is not interactive acceptance. See [LOCAL-INSTALL-05](docs/evidence/deployment/LOCAL-INSTALL-05-linux.md).

**Analysis filter layout refinement (2026-11-27):** source `6dd04e4` stacks the Most connected ranking controls and scales relationship-label shortening with pane width, keeping the full Unicode value available on hover. The 60-test offline suite and locked check/debug/release builds pass; release SHA-256 `76622945…` is installed in the current user launcher with synchronized graphics. X11 window creation succeeds, but current screenshot and input review remain blocked. See [UI-ANALYSIS-FILTERS-01](docs/evidence/vector-ui/UI-ANALYSIS-FILTERS-01-linux.md).

**Dense chart render review (2026-11-27):** the prior release renders a fictional 30-entity/90-record chart with Auto mode suppressing dense relationship labels while retaining routes. Its native screenshot showed cards clipped at the left viewport edge. A current-source native recapture after startup Fit all shows all 30 cards inside the viewport with Auto-mode labels suppressed; pointer/keyboard behavior, Arrange/Restore, and the real Clinton chart's native view remain untested. See [G4 dense-label implementation](docs/evidence/vector-ui/G4-DENSE-LABEL-AUTO-01-linux.md), [prior native render evidence](docs/evidence/vector-ui/G4-DENSE-LABEL-02-native-render-20261127.md), [current native render evidence](docs/evidence/vector-ui/G4-DENSE-LABEL-03-current-topology-render-20261127.md), and [Clinton corpus QA](docs/evidence/sql-automation/CLINTON-CORPUS-QA-01-linux.md).

**Clinton topology rendering cleanup (2026-11-27):** the real-corpus SVG now maps its out-of-range saved chart coordinates across the viewport instead of collapsing most entities onto the bottom edge. The current-source debug revalidation renders all 2,813 entities over 704 rows (largest row: 4), clips no entity labels, and passes query-only preflight and saved-run replay with zero deltas. Entity/route/hit/navigator transforms share the same per-chart bounds during drag; bounds serialize with chart tabs and are rebased by **Fit all**. Page-level text extraction now covers all 7,121 PDF pages, but email completeness is not established; Qwen/content review is blocked by local model access. Full native interaction on the real chart and IBM compatibility remain open. See [CLINTON-CORPUS-QA-02](docs/evidence/sql-automation/CLINTON-CORPUS-QA-02-current-debug-20261127.md), [CLINTON-CORPUS-EXTRACT-01](docs/evidence/sql-automation/CLINTON-CORPUS-EXTRACT-01-text-layer-coverage-20261127.md), and [CLINTON-CORPUS-QA-01](docs/evidence/sql-automation/CLINTON-CORPUS-QA-01-linux.md).

**Topology contact-line hover repair (2026-11-27):** source `30b8020` restores route highlighting and an anchored preview of group, record, pair, and direction counts; the hovered line remains unselected until clicked, and group interaction keeps priority. The 73 active offline tests and both Clinton topology checks pass. The optimized release (`330bc4fe…`) is installed with the Desktop/menu launcher and SVG/PNG assets synchronized. Native screenshot/pointer acceptance is blocked by this session's display capture failure; exact-head CI is pending. See [hover repair evidence](docs/evidence/vector-ui/PROXIMITY-MAP-HOVER-02-linux.md) and [current install evidence](docs/evidence/deployment/LOCAL-INSTALL-08-linux.md).

**Community interaction…5018 tokens truncated…view plus the optimized offline build pass. Native G3 arrange/restore and G4 draw/presentation performance gates remain open. See the [phased roadmap](docs/roadmaps/vector-community-exploration-roadmap.md), [headless UI evidence](docs/evidence/vector-ui/SNG-DELIVERY-002-G1-G2-headless-ui-20261126.md), and [native UI evidence](docs/evidence/vector-ui/SNG-DELIVERY-002-G1-G2-native-ui-20261126.md).

**Fit all viewport bounds (2026-11-27):** the headless regression now verifies that fitted entity cards at chart edges remain inside wide and narrower viewports. The focused test, 61-test suite, offline check/debug/release builds, and independent test review pass. Native Fit all, current-candidate label-mode review, and Arrange/Restore remain open; the live screen capture shows a panned subset and cannot identify the running binary. See [G4 Fit all bounds evidence](docs/evidence/vector-ui/G4-FIT-BOUNDS-01-linux.md).

**Current topology render (2026-11-27):** the current-source debug candidate renders a synthetic 30-entity chart with all cards inside the viewport; dense Auto mode suppresses relationship text labels while retaining route strokes. The hop-layer group contact map shows weighted ties and direction arrows, explains node/line encodings, spaces group metrics, uses singular-aware entity/record/pair labels, previews exact direction counts on contact-line hover, and lets analysts select a contact line to highlight and inspect its exact records. Selected contact lines are repainted above crossing routes, their direction arrows stay visible above the highlight, and nodes remain above the route layer. Independent topology review and headless paint-order regressions pass. The locked local release build and exact-head cross-platform CI both pass; the release binary is installed for the current user. Native map review remains open. Clinton-corpus visual review, IBM parity, and renderer performance remain open; see [G4-DENSE-LABEL-03](docs/evidence/vector-ui/G4-DENSE-LABEL-03-current-topology-render-20261127.md), [PROXIMITY-MAP-PICK-01](docs/evidence/vector-ui/PROXIMITY-MAP-PICK-01-linux.md), [selected route layering](docs/evidence/vector-ui/PROXIMITY-MAP-LAYER-01-linux.md), [PROXIMITY-MAP-LABELS-01](docs/evidence/vector-ui/PROXIMITY-MAP-LABELS-01-linux.md), and the [current branch acceptance](docs/build-acceptance.md#selected-topology-route-layering--2026-11-27).

**Current-user Linux deployment (2026-11-27):** the release binary built from source `57a714e` is installed at `~/.local/bin/arch-notebook` with SHA-256 `b3456f71…`. The application-menu entry and Desktop shortcut match; the scalable SVG, 1024×1024 PNG, MIME definition, and SQL automation installer match repository assets. Exact-head CI passed all six jobs. Native visual interaction remains unverified because this session cannot open the X display; see [LOCAL-INSTALL-09](docs/evidence/deployment/LOCAL-INSTALL-09-linux.md).

**Bounded group topology (2026-11-27):** the overview now samples at most 48 proxy-prioritized groups across hop layers, with a compact per-layer summary and complete virtualized group/contact indexes for drilldown. The production headless UI and retained Clinton structural regression pass; see [PROXIMITY-MAP-CLINTON-01](docs/evidence/vector-ui/PROXIMITY-MAP-CLINTON-01-linux.md). Native Clinton visual review and semantic/Qwen review remain open.

The production topology view also has an opt-in headless regression on the retained Clinton case at 1440×900. It scrolls to the real-corpus map and checks the selected-group details and full group index; native display review remains unavailable in this session. See [PROXIMITY-MAP-CLINTON-UI-01](docs/evidence/vector-ui/PROXIMITY-MAP-CLINTON-UI-01-linux.md).

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
