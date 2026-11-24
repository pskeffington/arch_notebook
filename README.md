# Arch Notebook

[![Build](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml)

A local desktop notebook for mapping relationships, comparing read-only SQLite snapshots, and automating recurring captures. Case files and run history stay on your machine.

## Start here

1. [Install Arch Notebook](docs/build-and-deploy.md).
2. In **SQL Review**, select a SQLite database and a read-only query with `source` and `target` columns.
3. Capture a baseline, then recapture with the same series name. Review removals, confidence declines, and property changes; export comparisons as CSV or TSV.

## Automation

Create a job, preflight it, then run it in the app, from the CLI, or on a systemd or LaunchAgent schedule. See [Automated SQL ingestion](docs/automation.md).

## Notebook

- Work across multiple named chart tabs; create a blank chart or copy the current selection to a new chart. Each chart retains its zoom and pan across tab switches and case reloads.
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
