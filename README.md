# Arch Notebook

[![Build](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml)

Arch Notebook is a local Rust desktop workbench for investigating relationship data and detecting connection regressions across SQL snapshots. It supports typed entity/link charts, evidence references, chronological analysis, shortest paths, duplicate review, and repeatable SQLite ingestion.

## SQL regression workflow

1. Open **Data → SQL regressions** and choose a SQLite database file.
2. Enter a stable **Snapshot series** label and a read-only query that returns `source` and `target` columns. Optional columns include `connection_id`, `source_type`, `target_type`, `relationship`, `date`, `confidence`, `directed`, `source_reference`, and `notes`.
3. Ingest a baseline query result. Update the query or database to the next period and ingest again using the same series label.
4. Review removed connections, confidence declines, changed event dates, additions, and confidence increases. The latest SQL snapshot also updates the chart for that database/series while preserving manually created and other-source links.

Connections with a stable `connection_id` are matched across snapshots by that ID. Without it, the app uses a natural key from source, target, relationship, types, and direction; include an ID when the source contains parallel connections with otherwise identical fields.

SQLite databases open in read-only mode. The query cannot write to or create a database. The included `sample_connections.sql` creates fictional baseline/current records that demonstrate a removed connection and a confidence decline:

```sh
sqlite3 sample_connections.sqlite < sample_connections.sql
cargo run --offline
```

For the sample comparison, ingest once with `snapshot_id = 'baseline'`, then change the query to `snapshot_id = 'current'` and ingest again with the same snapshot series.

## Automate repeatable ingestion

Open **Data → Automation**, enter a unique job name, SQLite database path, snapshot series, and query, then add the job and save the case. Enable the jobs you want included in a batch run. Each run records its status and timestamps in the case and appends a new immutable snapshot when the query succeeds. In the desktop app, save the case to persist job edits and run history. The command-line runner executes enabled jobs without opening the desktop UI and saves the updated case automatically:

```sh
cargo run --offline -- --run-enabled-jobs harbor-light.case.json
```

Run one named job, including a disabled job, with:

```sh
cargo run --offline -- --run-job harbor-light.case.json "Harbor Light baseline"
```

The runner saves the updated snapshots and run history back to the case file after processing. It continues through the selected jobs if one fails, saves each outcome, and exits nonzero when any job fails. Use an operating-system scheduler such as cron or systemd timers to run the enabled-job command on a recurring schedule. Keep the SQLite source available at the configured path when automation runs.

## Other notebook workflows

- Build and edit entity/link charts, inspect attributes and source references, arrange by connectivity, and review chronological timelines.
- Search and sort entity/relationship records, inspect most-connected entities, and analyze directed shortest paths.
- Review likely duplicate entities before merging. Merge rewires links and preserves aliases, attributes, notes, and the original record in entity history.
- Import/export relationship CSV and save/open local JSON case files. SQL snapshot history is retained with the case.

All bundled case examples are fictional. Arch Notebook does not connect to a service or transmit case data.

## Build and deploy

The supported desktop targets are Arch Linux and macOS on Apple Silicon or Intel. Rust 1.89 or newer is required. On Linux, the application stores its database, CSV example, and default case under `$XDG_DATA_HOME/arch-notebook`, or `~/.local/share/arch-notebook` when `XDG_DATA_HOME` is unset. On macOS, it uses `~/Library/Application Support/Arch Notebook`. Existing sample files are left in place on later launches.

### Arch Linux

Build and install the current `main` branch as a native package with `makepkg`:

```sh
git clone https://github.com/pskeffington/arch_notebook.git
cd arch_notebook/packaging/arch
makepkg -si
```

The package includes the desktop menu entry and declares its shared-library dependencies. `makepkg` downloads the locked Cargo dependencies and builds the release binary for the host architecture.

### macOS

Install Xcode Command Line Tools and Rust, then build a self-contained `.app` bundle and zip archive for the current Mac architecture:

```sh
./scripts/package-macos.sh
open "dist/Arch Notebook.app"
```

The script supports native Apple Silicon (`arm64`) and Intel (`x86_64`) hosts. For direct distribution outside the Mac App Store, sign with a Developer ID certificate and notarize with Apple. Configure a `notarytool` keychain profile, then run:

```sh
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
export APPLE_NOTARY_PROFILE="arch-notebook-notary"
./scripts/package-macos.sh
```

The profile is created with `xcrun notarytool store-credentials`; the signing identity and credentials are not stored in this repository. See [Apple's notarization guide](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution) and [Developer ID signing overview](https://developer.apple.com/developer-id/).

### Continuous builds

GitHub Actions checks formatting and builds release artifacts for Linux x86_64, macOS Apple Silicon, macOS Intel, and an Arch Linux package. Pull requests and pushes build without publishing a release; the generated workflow artifacts are available for 14 days. Publishing a GitHub Release triggers the workflow to attach the Linux archive, Arch package, macOS zips, and a `SHA256SUMS` file. macOS zip names state whether they are `unsigned`, `developer-id-signed`, or `notarized`; CI artifacts are unsigned because signing credentials are not stored in the repository.

To attach a locally signed and notarized macOS archive to a published release, build it with the credentials described above and upload it with:

```sh
gh release upload v0.1.0 "dist/Arch-Notebook-macos-$(uname -m)-notarized.zip" --clobber
```

The CI runner labels follow [GitHub's current hosted runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners). Arch package metadata follows the [Arch package guidelines](https://wiki.archlinux.org/title/Arch_package_guidelines).

See [RELEASING.md](RELEASING.md) for the version, tag, and signed macOS release process.

## Research basis

IBM i2 Analyst's Notebook documentation describes charts built from typed entities and links, attributes on chart items, confidence and direction on links, highly connected item analysis, and timeline charts. Those concepts inform the chart and investigation workflow:

- [IBM i2 Analyst's Notebook User Help (9.2.4)](https://www.ibm.com/docs/en/SSJSV9_9.2.4/pdf/analysts_notebook_pdf.pdf)
- [IBM Analyst's Notebook Essentials](https://www.ibm.com/training/course/ibm-i2-analysts-notebook-essentials-9I02G)
- [SQLite read-only open flags](https://www.sqlite.org/c3ref/open.html)
- [rusqlite Connection API](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html)
