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

Network analysis includes weakly connected groups, articulation entities (network bridges), connectivity rankings and shortest paths constrained by direction, timeline range or chronological order. Saved paths retain investigation criteria and recompute against the current case rather than preserving stale routes.

## Development status — 2026-11-25

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

Recent work added index and SQL-definition inspection, schema drift reporting in SQL Review and automation, KML placemark import, chart-import exports of selected historical captures, and ANX internal round-trip acceptance that preserves supported record IDs, repeated links, positions, labels, and whitespace. The importer rejects DTD-bearing files before modifying a case. SQL automation acceptance now includes joined header-shaped data with 7,121 records, composite identities, nullable attributes, and a two-snapshot comparison; see the [SQL scale acceptance evidence](docs/evidence/sql-automation/SQL-SCALE-01-joined-header-linux-620e733.md). See also [ANX acceptance evidence](docs/evidence/anx/ANX-ROUNDTRIP-02-xxe-linux-1014ba5.md), [SQL ingestion and interchange](docs/sql-ingestion.md), and the [Analyst's Notebook compatibility map](docs/analyst-notebook-parity.md) for details. SQLite schema_version is a drift signal, not a complete schema fingerprint. Validate CSV/TSV and ANX files in IBM's application before claiming external interoperability.

**Linux host progress (2026-11-25):** The operator reports that automatic builds appear to be working on the Linux box. This is a preliminary host observation, separate from CI success; the host architecture, build logs, installed version and runtime/scheduler acceptance remain to be recorded.

Next acceptance pass: finish the UI-00 native interaction and visual sequence. The release binary launches on Wayland and emits chart-preparation and navigator CPU samples for the compact fictional fixture; picking, repeated interaction traces, and visual inspection remain pending. Scripted CLI acceptance passes fictional SQLite baseline/current capture, override isolation, run history, deltas, schema-stability rejection, failed-run retention, and comparison export. Schema adaptation now persists the missing-column diagnosis as structured run data, retains the failed run, and supports an explicit reviewed query repair followed by a successful capture; CSV and TSV chart exports also round-trip through saved import designs while preserving record and source-scoped endpoint identity and custom properties. Native Automation UI review, live scheduled execution, and chart navigation remain pending; IBM i2 schema/application validation is still required for external interoperability claims. See the [Wayland runtime evidence](docs/evidence/vector-ui/UI-00-linux-wayland-a84a938.md), [SQL schema and failed-run acceptance](docs/evidence/sql-automation/SQL-CAPTURE-03-schema-and-failure-linux-d5c07ec.md), [chart CSV/TSV round-trip evidence](docs/evidence/sql-automation/SQL-CAPTURE-04-chart-csv-tsv-linux-b365fdc.md), [schema adaptation and reviewed repair evidence](docs/evidence/sql-automation/SQL-CAPTURE-05-schema-adaptation-linux-86e2be5.md), and [structured diagnosis acceptance](docs/evidence/sql-automation/SQL-CAPTURE-06-structured-adaptation-linux-2ed5163.md). CI success establishes build/package success, not interactive or production acceptance.

The SQL CLI acceptance now also exercises interrupted batch recovery: it saves the first successful job checkpoint, repairs a missing view in a disposable SQLite fixture, resumes the same batch, confirms the completed job is skipped, and rejects a changed-parameter resume without changing the case. See [SQL batch recovery evidence](docs/evidence/sql-automation/SQL-CAPTURE-07-batch-resume-linux.md). This validates CLI recovery only; native scheduler restart, sleep/reboot recovery, and interactive cancellation remain pending.

The Linux scheduler harness now invokes a generated service `ExecStart` against its own fresh synthetic SQLite case and verifies the scheduled-trigger run, saved snapshot, and chart CSV. The host systemd user bus denies access, so actual timer dispatch remains unverified; see the [scheduler invocation follow-up](docs/evidence/sql-automation/SQL-SCHED-01-linux.md).

The scheduler harness also checks a rolling previous-UTC-month SQL query through that generated service command, including the inclusive start and exclusive end boundary. Scheduled parameter values remain fixed; SQLite date functions provide rolling time windows. See [SQL-SCHED-03 evidence](docs/evidence/sql-automation/SQL-SCHED-03-dynamic-window-linux.md).

The isolated macOS LaunchAgent harness now runs the bundled automation runner against the same rolling-window case and checks the scheduled trigger, boundary rows, CSV output, and unchanged SQLite source. The harness uses a temporary `launchctl` stub on Linux; live macOS dispatch is still pending. See [SQL-SCHED-04 evidence](docs/evidence/sql-automation/SQL-SCHED-04-macos-window-linux-harness.md).

Desktop SQL automation now has worker-level cancellation regression coverage: after cancelling a long second query, the failed run is retained while the earlier successful snapshot and batch checkpoint remain. The test confirms the source database stays unchanged, but does not exercise the native cancel button; see [cancellation evidence](docs/evidence/sql-automation/SQL-CANCEL-01-worker-linux.md).

**Vector UI-00 branch evidence:** `codex/ui-00-chart-baseline` adds deterministic fictional workloads, chart geometry and selected-record identity regressions, opt-in local timings from the application chart path, SQL automation acceptance, isolated Linux systemd and macOS LaunchAgent installer lifecycle checks, and supported ANX round-trip checks. The follow-up on `df23448` corrects the parallel-link regression to compare rendered routes after direction normalization; all 16 Rust tests now pass with no ignored tests. See the [reversed-link route evidence](docs/evidence/vector-ui/UI-00-reversed-link-routes-linux-df23448.md) and the [Quick open interaction record](docs/evidence/vector-ui/UI-QUICK-OPEN-01-linux.md). Earlier CI run 45723413554 passed SQL/ANX, both isolated scheduler acceptance suites, Linux x86_64/ARM64, Arch packaging, and macOS Intel/Apple Silicon on `986f3e8`; run 45709682338 passed the earlier Linux scheduler candidate on `1e5a86f`. The local release build passes, but native chart interaction and visual review remain pending; see the [UI-00 follow-up](docs/evidence/vector-ui/UI-00-followup-linux.md), [Wayland runtime evidence](docs/evidence/vector-ui/UI-00-linux-wayland-a84a938.md), [R1 source follow-up](docs/evidence/vector-ui/R1-linux-30de20a.md), [literature matrix](docs/research/vector-ui-literature-matrix.md), [Linux scheduler evidence](docs/evidence/sql-automation/SQL-SCHED-01-linux.md), [isolated macOS scheduler evidence](docs/evidence/sql-automation/SQL-SCHED-02-macos-isolated.md), and [ANX acceptance record](docs/evidence/anx/ANX-ROUNDTRIP-01-linux.md).

The current source also has a release build and limited Wayland runs of the fictional chart fixture. An active-window screenshot shows the rendered chart, but its title bar said “Not Responding”; inconsistent compositor access prevented a controlled repeat or interaction trace. The UI-00 native baseline remains open pending responsiveness, input, and visual review; see the [latest Linux recheck](docs/evidence/vector-ui/UI-00-linux-recheck-e44d485.md).

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
