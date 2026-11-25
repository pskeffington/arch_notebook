# Arch Notebook

[![Build](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml)

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
| Native IBM i2 ANX import/export | Not implemented |
| TRANS → Arch Notebook PDF corpus workflow | Successful test run recorded; technical artifacts pending |
| GUI and vector-rendering refinement | In progress |
| Interactive host acceptance and external IBM import validation | Pending evidence |
| Published release and macOS notarization | Not published / signing evidence pending |

Recent work added index and SQL-definition inspection, schema drift reporting in SQL Review and automation, KML placemark import, and chart-import exports of selected historical captures. See [SQL ingestion and interchange](docs/sql-ingestion.md) for details. SQLite schema_version is a drift signal, not a complete schema fingerprint. CSV/TSV is the supported IBM i2 handoff; native ANX is not supported.

Native Linux host build and scheduler acceptance remain pending documented execution results. See [build and deployment](docs/build-and-deploy.md).

Native-host acceptance uses fictional fixtures for snapshot comparison, schema drift, interchange and scheduled execution. External IBM i2 import fidelity and native ANX support remain outside the verified scope.

## Real-world email analysis

The November 25, 2026 development test record documents a successful end-to-end TRANS → Arch Notebook workflow using a PDF dump of the Clinton email corpus. The workflow progressed from document processing in TRANS to correspondence-derived actor relationships and native graph display in Arch Notebook. GUI layout and vector rendering remain refinement areas. The [acceptance record](docs/email-analysis-acceptance.md) distinguishes the recorded test outcome from pending exact-revision execution artifacts.

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
| IBM i2 Analyst's Notebook | CSV/TSV handoff via a receiving import specification; native ANX is not implemented. |

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
