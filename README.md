# Arch Notebook

[![Build](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml)

A local desktop notebook for mapping relationships, comparing read-only SQLite snapshots, and automating recurring captures. Case files and run history stay on your machine.

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
| Interactive host acceptance and external IBM import validation | Pending evidence |
| Published release and macOS notarization | Pending owner approval / signing evidence |

Recent work added index and SQL-definition inspection, schema drift reporting in SQL Review and automation, KML placemark import, and chart-import exports of selected historical captures. See [SQL ingestion and interchange](docs/sql-ingestion.md) for details. SQLite schema_version is a drift signal, not a complete schema fingerprint. CSV/TSV is the supported IBM i2 handoff; native ANX is not supported.

Next acceptance pass: use the fictional fixtures to exercise baseline/current capture, schema and property drift, chart export/reimport, and scheduled capture on native hosts; record commands, host versions, output and defects. Then validate the CSV/TSV import in IBM i2 if an installation is available. Obtain the vendor ANX schema and fixtures before implementing ANX. CI success establishes build/package success, not interactive or production acceptance.

## Start here

1. [Install Arch Notebook](docs/build-and-deploy.md).
2. In **SQL Review**, select a SQLite database and a read-only query with `source` and `target` columns.
3. Capture a baseline, then recapture with the same series name. Review removals, confidence declines, and property changes; export comparisons as CSV or TSV.

## Automation

Create a job, preflight it, then run it in the app, from the CLI, or on a systemd or LaunchAgent schedule. See [Automated SQL ingestion](docs/automation.md).

## Notebook

- Chart entities and connections; inspect timelines, paths, records, and sources.
- Import and export CSV/TSV, GraphML, and KML.
- Save cases, SQL snapshots, jobs, and run history locally.

## Platforms

Arch Linux x86_64, generic Linux x86_64/ARM64, and macOS on Apple Silicon or Intel.

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
