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

Named chart tabs retain their own viewports. Analysts can apply saved CSV/TSV import designs, open a saved case from the Linux launcher, and import approved downstream document handoffs into a separate evidence chart.

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
| ANX XML chart import/export | Implemented; IBM XSD and application validation pending |
| Deterministic chart fixtures and automated chart regressions | Implemented on `codex/ui-00-chart-baseline` |
| Interactive host acceptance and external IBM import validation | Pending evidence |
| Published release and macOS notarization | Pending owner approval / signing evidence |

Recent work added index and SQL-definition inspection, schema drift reporting in SQL Review and automation, KML placemark import, chart-import exports of selected historical captures, and an ANX internal round-trip acceptance gate that preserves supported record IDs, repeated links, positions, labels, and whitespace. See [SQL ingestion and interchange](docs/sql-ingestion.md) and the [Analyst's Notebook compatibility map](docs/analyst-notebook-parity.md) for details. SQLite schema_version is a drift signal, not a complete schema fingerprint. Validate CSV/TSV and ANX files in IBM's application before claiming external interoperability.

**Linux host progress (2026-11-25):** The operator reports that automatic builds appear to be working on the Linux box. This is a preliminary host observation, separate from CI success; the host architecture, build logs, installed version and runtime/scheduler acceptance remain to be recorded.

Next acceptance pass: finish the UI-00 native interaction and visual sequence. The release binary now launches on Wayland and emits chart-preparation and navigator CPU samples for the compact fictional fixture; picking, repeated interaction traces, and visual inspection remain pending. Then use fictional fixtures to exercise baseline/current SQL capture, schema and property drift, chart export/reimport, scheduled capture, and chart navigation. Validate CSV/TSV and supported ANX XML in IBM i2 when its schema/application are available. See the [Wayland runtime evidence](docs/evidence/vector-ui/UI-00-linux-wayland-a84a938.md). CI success establishes build/package success, not interactive or production acceptance.

**Vector UI-00 branch evidence:** `codex/ui-00-chart-baseline` adds deterministic fictional workloads, chart geometry and selected-record identity regressions, opt-in local timings from the application chart path, SQL automation acceptance, isolated Linux systemd and macOS LaunchAgent installer lifecycle checks, and supported ANX round-trip checks. On candidate `cc7ed12`, 13 Rust tests pass and one parallel-lane uniqueness assertion is intentionally ignored because it reproduces a known reversed-record lane collision; see the [fixture regression report](docs/evidence/vector-ui/UI-00-fixture-regressions-cc7ed12.md). CI run 45723413554 passed SQL/ANX, both isolated scheduler acceptance suites, Linux x86_64/ARM64, Arch packaging, and macOS Intel/Apple Silicon on `986f3e8`; run 45709682338 passed the earlier Linux scheduler candidate on `1e5a86f`. A local release build and bounded Wayland run pass, but the native interaction and visual baseline remains pending; see the [UI-00 follow-up](docs/evidence/vector-ui/UI-00-followup-linux.md), [Wayland runtime evidence](docs/evidence/vector-ui/UI-00-linux-wayland-a84a938.md), [R1 source follow-up](docs/evidence/vector-ui/R1-linux-30de20a.md), [literature matrix](docs/research/vector-ui-literature-matrix.md), [Linux scheduler evidence](docs/evidence/sql-automation/SQL-SCHED-01-linux.md), [isolated macOS scheduler evidence](docs/evidence/sql-automation/SQL-SCHED-02-macos-isolated.md), and [ANX acceptance record](docs/evidence/anx/ANX-ROUNDTRIP-01-linux.md).

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
