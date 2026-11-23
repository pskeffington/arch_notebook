# Arch Notebook

[![Build](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml)

A local desktop workbench for exploring relationship data and finding changes between read-only SQL snapshots. Case files, charts, queries, and run history stay on your machine.

## Quick start

1. Install Arch Notebook using the platform instructions in [Build and deploy](docs/build-and-deploy.md).
2. Open **SQL Review**, choose a SQLite database, and enter a read-only query returning `source` and `target` columns.
3. Give the capture a stable series name and ingest a baseline. Ingest later results under the same name.
4. Review removals, confidence declines, changed details, and additions; export the comparison as CSV or TSV when needed.

See [SQL snapshot ingestion](docs/sql-ingestion.md) for supported columns, query parameters, comparison options, and data safety details.

## Automate ingestion

Create jobs under **Automation** to validate and capture recurring SQLite queries. Preflight enabled jobs and one-run parameter overrides before a batch, inspect progress and per-job diagnostics, then review outcomes in run history. Jobs can run from the desktop, CLI, or scheduled systemd/LaunchAgent tasks.

See [Automated SQL ingestion](docs/automation.md) for job setup, CLI commands, scheduling, replay, and package sharing.

## Notebook features

- Explore typed entities and relationships with custom properties in chart, timeline, and data views.
- Trace paths, inspect evidence and history, and review possible duplicates.
- Import and export CSV/TSV, GraphML, and KML with source attribution and geographic coordinates.
- Save cases as local JSON files and retain SQL snapshot and automation history.

See [Notebook workflows](docs/notebook-workflows.md) for these features.

## Supported platforms

- Arch Linux x86_64 package
- Generic Linux x86_64 and ARM64 archives
- macOS on Apple Silicon and Intel

## Documentation

- [SQL snapshot ingestion](docs/sql-ingestion.md)
- [Automated SQL ingestion](docs/automation.md)
- [Notebook workflows](docs/notebook-workflows.md)
- [Build and deploy](docs/build-and-deploy.md)
- [Project references and maintenance](docs/project-reference.md)
- [Release process](RELEASING.md)
- [Archived research databases and fixtures](archive/research.databases/README.md)

## Development

Rust 1.89 or newer is required. From the repository root:

```sh
cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline
```

See [Build and deploy](docs/build-and-deploy.md) for packaging details. Bundled case examples are fictional; the app does not connect to a service or transmit case data.
