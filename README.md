# Arch Notebook

[![Build](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml)

Arch Notebook is a local Rust desktop workbench for investigating relationship data and detecting connection regressions across SQL snapshots. It supports typed entity/link charts, evidence references, chronological analysis, shortest paths, duplicate review, and repeatable SQLite ingestion.

## SQL regression workflow

1. Open **Data → SQL regressions** and choose a SQLite database file.
2. Enter a stable **Snapshot series** label and a read-only query that returns `source` and `target` columns. Optional columns include `connection_id`, `source_type`, `target_type`, `relationship`, `date`, `confidence`, `directed`, `source_reference`, and `notes`. Column aliases are matched without case or surrounding whitespace; if a query returns multiple aliases for the same field (such as both `source` and `from`), ingestion reports the ambiguity instead of silently choosing one. Rows with a blank or NULL source or target are skipped and their count is retained with the snapshot and shown in validation, run history, and snapshot history. SQL `notes` are retained as source notes; analyst notes are stored separately and survive later refreshes. Missing confidence defaults to 2; supplied confidence must be an integer from 1 to 3. Direction accepts true/false, 1/0, yes/no, directed/undirected, or →/↔. In either the SQL regressions editor or automation job editor, **Build query from schema** inspects the selected database read-only and drafts a `SELECT` when it recognizes source and target columns; review and edit the generated query before ingestion.
   Bind changing query values with the SQL parameters field: enter one `name=value` per line and use `:name`, `@name`, or `$name` in the query. SQLite binds the values instead of interpolating them into SQL text. Parameter values are saved with automation jobs, captures, and run history so an outcome can be reproduced.
3. Ingest a baseline query result. Update the query or database to the next period and ingest again using the same series label.
4. Review removed connections, confidence declines, changed event dates, additions, and confidence increases. Each changed connection appears once, with separate change labels for confidence, date, connection details, evidence references, and source notes. Filter the review by regression, direction, evidence, source-note, or other changes, and search across IDs, endpoint names/types, relationship labels, evidence, and notes. Comparison rows show typed endpoints, direction, evidence references, and notes before and after the change, including for removed connections. Use **Export comparison CSV/TSV** to share the complete selected baseline/current delta report with other analysis tools; the export includes before/after facts, both snapshot IDs and capture times, and the skipped-row count for each capture, regardless of the on-screen filter. The **Review saved series** picker shows the number of chart links and endpoint entities from each SQL source. **Highlight source on chart** fits the chart to and emphasizes those links and entities while dimming unrelated chart records; **View source records** opens the Connections table with a clearable source filter. The fields under **Ingest new snapshot** configure a separate new capture. The latest SQL snapshot also updates the chart for that database/series while preserving manually created and other-source links.

The saved-series review table can filter regressions, direction changes, evidence changes, source-note changes, or other updates, and search both sides of a change by connection ID/key, endpoint, type, date, evidence, or notes. Filtering only changes the on-screen rows; the comparison export always includes every change. The export retains a readable `change` label and adds a boolean column for each change type, so spreadsheet filters and downstream tools can select exact regression categories. Direction changes have their own `direction_changed` category while remaining included in `connection_details_changed` for existing consumers. The `connection_key` column exposes the same stable ID or natural key used to match each record across snapshots, including when the source query does not provide `connection_id`. Structured automation deltas use this same normalized key in each connection’s `key` field.

Export a saved SQL comparison without opening the desktop app by passing the baseline and current snapshot UUIDs. The snapshots must be ordered in the same saved database/series. A `.tsv` or `.tab` output path selects tab-separated output; other paths use CSV. Export is atomic and refuses to overwrite a saved SQLite source database.

```sh
cargo run --offline -- --export-sql-comparison harbor-light.case.json <baseline-snapshot-id> <current-snapshot-id> comparison.csv --json
```

The JSON response reports the output format, snapshot IDs, and number of changed records. Get snapshot UUIDs from `--list-runs --json` or the case’s saved snapshot records.

CSV/TSV and comparison reports are written to a temporary sibling file and atomically replace the destination only after the full export succeeds. **Import GraphML** stages nodes and relationships through the validated import path; it supports a single graph and imports recognized IDs, aliases, entity attributes, direction, evidence, notes, and chart styling. **Export GraphML** works with graph tools such as Gephi and Cytoscape and includes endpoint IDs, direction, labels, evidence, notes, confidence, source provenance, entity attributes, and chart styling. GraphML is an interchange format; CSV/TSV carries the full Arch Notebook case fields for round-trips. For scripts, run `arch-notebook --export-graphml case.json chart.graphml --json` without opening the desktop app; the schema-versioned JSON result reports the output path and exported entity and relationship counts.

Connections with a unique, stable `connection_id` are matched across snapshots by that ID. A query result with duplicate IDs is rejected so ambiguous rows cannot distort regression comparisons. Without an ID, the app uses a natural key from source, target, relationship, types, and direction; duplicate natural keys in one result are rejected, so include a unique ID when the source contains parallel connections with otherwise identical fields. During CSV or SQL ingestion, entities are reconciled by name, a unique compatible alias, and supplied type. This keeps a merged entity connected to its imported records on refresh; ambiguous aliases remain separate for analyst review. Same-name entities of different types remain distinct while generic entities can gain a type from later imports.

SQLite databases open in read-only mode, and SQLite-classified writable statements are rejected before execution. The query cannot write to or create a database. Each capture reads its timestamp and query results in one SQLite read transaction, so a concurrent commit cannot make the timestamp and rows describe different database states. Reads wait up to five seconds for a SQLite write lock before returning an error, which helps scheduled captures run through short source updates. The fictional seed files are archived under `archive/research.databases/`; `sample_connections.sql` creates baseline/current records demonstrating a removed connection and confidence decline:

CSV/TSV exports also refuse to replace a database used by a saved snapshot, automation job, or current SQL editor path.

```sh
sqlite3 archive/research.databases/sample_connections.sqlite < archive/research.databases/sample_connections.sql
cargo run --offline
```

For the sample comparison, ingest once with the parameter `snapshot_id=baseline`, then change it to `snapshot_id=current` and ingest again with the same snapshot series.

## Automate repeatable ingestion

In both the automation editor and one-off SQL snapshot panel, **Build query from schema** inspects the configured SQLite file read-only and generates an editable `SELECT` from the first table or view with recognizable source and target columns. Review and adjust the selected columns before validating or saving the query.

Open **Data → Automation**, enter a unique job name, SQLite database path, snapshot series, and query, then use **Validate query** to check the read-only query, returned connection count, and configured limits against the latest saved snapshot without saving case changes. A successful validation also previews up to eight returned connections so you can confirm the endpoint mapping before ingestion. The status distinguishes an invalid query from a valid query whose configured limits fail; CLI preflight uses the same limit assessment as an actual job run. Each database/series identity can belong to only one automation job; aliases to the same existing database file resolve to the same identity. This keeps one query stream from replacing another job’s chart links or mixing unrelated results in a comparison chain. Relative database paths resolve from the case file’s directory. Snapshots retain both the canonical database path and the configured path; when a case and its relative-path database move together, loading the case rebases its saved series and imported-link provenance to the new location. When loading a case, paths in earlier snapshots are normalized too when their source databases are still available. Set **Minimum connections** to require a nonzero result size; the default of 0 disables this check. A result below the configured minimum is recorded as a failed run, while its successfully captured snapshot remains in history for review. Optionally enable **Limit regressions** and set a maximum count; a regression is a removed connection or confidence decline versus the preceding snapshot in the same database/series. The baseline snapshot has no regressions to compare, and runs still save their snapshot and history when a limit is exceeded. Optionally set **Limit skipped rows** and a maximum count to fail runs when the SQL result has too many rows with blank or NULL endpoints. The skipped-row count and threshold are retained with the run, while the successfully captured snapshot stays available for review. Add the job when it is ready. The Automation list shows each job’s latest run status and time; hover for its result message. Search the list by job name, database path, snapshot series, or SQL text without changing saved jobs or run history. Expand **SQL query and capture configuration** to inspect a saved job without loading it into the editor. Use **Validate** beside a saved job to preflight its query and limits without changing the case; **Edit** loads its configuration into the editor. Enable the jobs you want included in a batch run. Running an individual job or batch keeps the current editor fields intact. Each run records its status and timestamps in the case and appends a new immutable snapshot when the query succeeds. Desktop **Run and save** actions atomically save the updated case, including job edits, snapshots, and run history; use **Save case** to persist edits without running a job. The command-line runner executes enabled jobs without opening the desktop UI and saves the updated case automatically:

Set the maximum to 0 to reject any skipped row. Older jobs default to no skipped-row limit.

Use SQLite named parameters when a scheduled query needs a changing period, tenant, or threshold instead of editing the SQL each run. Enter one `name=value` per line in **SQL parameters** and reference it in the query with `:name`, `@name`, or `$name`. Values bind as text; SQLite column affinity handles common date, ID, and numeric comparisons. Parameters are saved with the automation job, each immutable snapshot, and each run for reproducibility. For a one-off CLI run or preflight, repeat `--param name=value` to override or add parameter values without changing the saved job. For example, use `WHERE snapshot_id = :snapshot_id` with `snapshot_id=current`.

Desktop batch runs save an atomic checkpoint after each job, then save the final batch summary. The command-line runner also saves after each job, preserving completed run history if a later query stalls or the process is interrupted.

```sh
cargo run --offline -- --run-enabled-jobs harbor-light.case.json
```

Supply repeated `--job <name-or-id>` selectors to validate or run a subset of enabled jobs. Name matching ignores case; UUIDs remain stable across renames, and selected jobs execute in case-file order. A selected disabled or unknown job is an error. Use `--run-job` to run one named or UUID-selected job even when it is disabled:

```sh
cargo run --offline -- --validate-enabled-jobs harbor-light.case.json --job "Harbor Light baseline" --json
cargo run --offline -- --run-enabled-jobs harbor-light.case.json --job "Harbor Light baseline" --json
```

Run one named job, including a disabled job, with:

```sh
cargo run --offline -- --run-job harbor-light.case.json "Harbor Light baseline"
# A UUID selector continues to work if the job is renamed.
cargo run --offline -- --run-job harbor-light.case.json 6f20f078-0dd4-4df3-b81a-e646aaaf4412 --json
cargo run --offline -- --run-job harbor-light.case.json "Harbor Light baseline" --param snapshot_id=current --json
```

Pass `--param name=value` to apply a shared override to every selected job. Every selected query must use each shared parameter name. Use `--job-param <job-name-or-id> name=value` for values that differ by job; repeat the option for additional jobs or values. Job-specific overrides are applied after shared overrides and replace only that job’s matching value. A job-specific selector must refer to a job in the selected batch, and each targeted query must use its supplied parameter names.

```sh
cargo run --offline -- --validate-enabled-jobs harbor-light.case.json --job "Harbor Light baseline" --param snapshot_id=current --json
cargo run --offline -- --run-enabled-jobs harbor-light.case.json --job "Harbor Light baseline" --param snapshot_id=current --json
cargo run --offline -- --run-enabled-jobs harbor-light.case.json --job "Harbor Light baseline" --job "Harbor Light follow-up" --job-param "Harbor Light baseline" snapshot_id=baseline --job-param "Harbor Light follow-up" snapshot_id=current --json
```

List configured jobs as JSON without changing the case file:

```sh
cargo run --offline -- --list-jobs harbor-light.case.json
```

Add `--json` to get a schema-versioned object containing each job’s UUID, enabled state, database, series, SQL query and parameters, limits, and compact latest-run status/timestamps/message/snapshot ID, plus total, enabled, and returned counts. Use `--enabled-only` to inspect just the batch set; it can be combined with `--json`. Without `--json`, output remains a JSON array for compatibility.

```sh
cargo run --offline -- --list-jobs harbor-light.case.json --enabled-only --json
```

Read saved automation outcomes without opening the desktop app:

```sh
cargo run --offline -- --list-runs harbor-light.case.json --json
```

The result uses schema version 1 and lists newest runs first with status, job identity, timestamps, elapsed query and validation duration in milliseconds, database/series, the captured query and bound parameters, thresholds, enabled state, outcome message, linked snapshot ID, capture time, connection count, and skipped-row count when a snapshot was produced. It also returns the structured limit checks and typed snapshot delta with stable connection keys and prior/current facts, matching run and preflight outcomes. Older runs without recorded configuration return `limits: null` rather than implying their default fields were saved settings; runs from older case files report a null duration. Listing run history does not modify the case.

Filter by job name or UUID and page through newest-first history with `--job <name-or-id>`, `--offset <count>` (default `0`), and `--limit <count>` (positive integer). JSON responses include the active filter, total matches, returned count, and `next_offset` when another page exists. The selection is applied before snapshot deltas are computed.

Inspect a SQLite source before writing an ingestion query. This command opens the database read-only and lists its tables, views, column names, declared types, and key/required-column markers:

```sh
cargo run --offline -- --inspect-sqlite source.sqlite --json
```

The JSON response uses schema version 1 and includes `ok`, `object_count`, and the table/view definitions for query-authoring tools. It reports structure only; it does not read table rows or modify the source. The text output reports when the database contains no user tables or views.

Preflight every enabled SQL job before scheduling a batch. Validation runs each query read-only, checks minimum and optional regression limits, and evaluates jobs in run order with a temporary snapshot chain. It does not save snapshots or change the case file:

```sh
cargo run --offline -- --validate-enabled-jobs harbor-light.case.json
```

Append `--json` for a machine-readable validation summary (`ok`, `saved`, pass/fail counts, and per-job outcomes). Batch execution also accepts `--json`; it prints the same summary shape after atomically saving snapshots and run history. Each executed batch outcome includes its persisted `run_id` (null for validation) and `duration_ms`, so callers can correlate failures without a snapshot, inspect query time, and reconcile outcomes with `--list-runs`. The desktop run history shows the same duration. The case is atomically updated after each completed job, so earlier outcomes survive if a later query stalls or the runner is interrupted. In JSON mode, standard output contains only one JSON result; errors that occur before a summary is available produce an `{ "schema_version": 1, "ok": false, "error": "..." }` object, while diagnostics also go to standard error. Successful results and JSON errors carry schema version 1. Failed jobs still return a nonzero process exit code. For example:

```sh
cargo run --offline -- --run-enabled-jobs harbor-light.case.json --json
```

Run and validation JSON use schema version 1 and retain the existing per-job fields. Each outcome also includes observed-versus-required limit checks, snapshot identity and capture time, a typed delta summary, and changed connection facts with stable identity keys and prior values. Direction changes are explicitly typed as `direction_changed` while still counting as `connection_updated` for compatibility. Downstream tools can evaluate regressions and inspect evidence changes without parsing the human-readable message. Automation CLI runs and desktop case saves coordinate through a per-case advisory lock. Concurrent CLI runs wait for the active runner; desktop saves compare the on-disk case with the version they loaded and reject stale writes, preserving newer automation checkpoints. The lock file remains beside the case file and is reused on later runs.

Replay a recorded automation run by its run UUID, including its saved query, parameters, and thresholds:

```sh
cargo run --offline -- --rerun-run harbor-light.case.json <run-id> --json
```

To check the same saved configuration against the current SQLite source without saving a snapshot or run, use `--validate-run` first:

```sh
cargo run --offline -- --validate-run harbor-light.case.json <run-id> --json
```

Validation leaves the case unchanged. Rerun saves a new run and snapshot, leaves the job definition unchanged, and reports `rerun_of` in JSON. Both commands work even if the original job was edited or removed, but reject older run records that did not preserve their configuration. Get the UUID from `--list-runs --json`.

To resume the same batch after an interruption, add `--skip-recent-success` to `--run-enabled-jobs`. A job is skipped only when its newest recorded run for the exact current database path, series, SQL query, parameters, and limits succeeded. A changed configuration, missing historical configuration, or latest failure runs again. Skips are reported separately in JSON (`skipped_jobs`) with their effective SQL configuration, prior run ID, and duration; they do not add snapshots, run history, or rewrite the case when every selected job is skipped. Omit this flag for regular scheduled captures so each invocation reads fresh source data.

Run history is bounded to the latest 200 records. Each run retains the job configuration and SQL query used, including failures, so later job edits or deletion do not erase the run context; expand a row in the desktop history to inspect it. Select **Replay saved configuration** to run that exact saved SQL, parameters, and thresholds against the current SQLite source; replay appends a new run and snapshot without changing the job definition. The new run records the ID of the run it replayed, in both desktop history and `--list-runs --json` output. Older runs that predate this configuration capture are labeled as such rather than showing default values as if they were recorded settings. Successful runs include a comparison with the previous snapshot in the same database and series; the history records removed connections, confidence declines, date changes, additions, confidence increases, and other updates. Each snapshot has a stable ID, and automation run records link directly to it; older cases migrate those links from their saved snapshot indices when loaded. In the desktop app, use **Review snapshot #N** beside a run or select any numbered capture in **Snapshot history** to compare it with the preceding capture in that series; the capture number disambiguates runs recorded within the same clock interval. Batch summaries report passed and failed job counts. The runner continues through the selected jobs if one fails, saves each outcome, and exits nonzero when any job fails.

On Arch Linux with systemd, copy the example user units installed with the package and edit the case path in the service if needed. From a source checkout, use the corresponding files under `packaging/systemd/` instead:

```sh
mkdir -p ~/.config/systemd/user
unit_dir=/usr/share/arch-notebook/systemd
[ -d "$unit_dir" ] || unit_dir=packaging/systemd
cp "$unit_dir/arch-notebook-automation.service" "$unit_dir/arch-notebook-automation.timer" ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now arch-notebook-automation.timer
```

The timer runs hourly, catches up a missed run after the machine was offline, and adds up to five minutes of randomized delay. User timers run only while the user systemd manager is active; to keep it running after logout and start it at boot, enable lingering for the account with `loginctl enable-linger "$USER"`. The service allows a run up to one hour to finish; adjust `TimeoutStartSec` for longer queries. Run the service immediately with `systemctl --user start arch-notebook-automation.service`; inspect results and errors with `journalctl --user -u arch-notebook-automation.service`. The example case path assumes the default Linux data directory; change it to the case file you want scheduled. On macOS, install the LaunchAgent for a readable case file and app bundle:

```sh
scripts/install-macos-automation.sh "$HOME/Library/Application Support/Arch Notebook/harbor-light.case.json" "/Applications/Arch Notebook.app"
```

The installer writes and validates a per-user LaunchAgent plist, then loads it for the current login session. It runs once when loaded and hourly afterward; inspect `~/Library/Logs/arch-notebook-automation.log` and `~/Library/Logs/arch-notebook-automation-error.log`. To remove the schedule, run `launchctl bootout "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.archnotebook.automation.plist"` and remove that plist. Keep the SQLite source available at the configured path when automation runs.

## Other notebook workflows

- Build and edit entity/link charts, choose both endpoints when creating a link, inspect attributes and source references, arrange with connectivity, breadth-first hierarchy, or alphabetical grid layouts while pinning selected entity positions, and review timelines with ISO dates positioned by elapsed time. Use **Fit all** to frame every entity after manual rearrangement; path analysis can fit the full shortest path separately. Parallel relationship records render on separate chart lanes and can be selected individually.
- Review the grouped Connections table by entity pair and relationship type, then expand a pair to inspect its link records. Entity connection counts and MOST CONNECTED ranking count distinct counterpart entities; parallel link records remain visible in the pair and record tables. Switch to the Link records table for a sortable row view by record ID, source, relationship, target, date, confidence, or source reference. Filter across endpoints, evidence, notes and ingest provenance; scope results to all connections or those involving the selected entity, then return a selected link to the chart.
- Search and sort entity records, focus chart entities from the most-connected list, and fit the full shortest path into the chart view.
- Review likely duplicate entities of compatible types before merging. Merge rewires links and preserves aliases, attributes, notes, and the original record in entity history.
- Import/export relationship CSV or TSV (`.tsv` and `.tab` paths use tab delimiters) with optional `connection_id`, `ingest_source`, and entity properties retained across round-trips. Exports include `source_entity_id` and `target_entity_id` UUID columns so reimport preserves entity identity even when names are duplicated. **Import CSV/TSV** opens a design dialog with a row preview and field mapping for source/target, entity IDs, types, relationship, date, confidence, direction, identifiers, notes, provenance, endpoint details, aliases, colors, chart positions, position locks, and custom entity attributes. Duplicate headers, ignoring case and surrounding whitespace, are rejected so a mapping cannot silently choose the wrong column. Save all-match row filters (equals, contains, is empty, or is not empty) and optional whitespace trimming with each design; skipped rows are reported. Save named import designs in the case file; a unique design saved for the same set of column names is applied automatically, while multiple matches remain available for manual selection. Header order, case, and surrounding whitespace do not affect the match; standard headers are detected automatically. Rows marked `record_type=entity` carry isolated entities; connection rows leave `record_type` blank or use `connection`. The `notes` column carries analyst notes; `source_notes` carries notes supplied by SQL or another source. If a matching row is reimported without a `source_notes` column, its saved source notes are preserved; a present but blank `source_notes` cell clears them. Endpoint detail, aliases, chart color, and normalized chart position use `source_detail_json`/`target_detail_json`, `source_aliases_json`/`target_aliases_json`, `source_color`/`target_color` (`#RRGGBB`), and `source_x`/`source_y`/`target_x`/`target_y` (0–1), and `source_position_locked`/`target_position_locked` (true/false); JSON `""` clears detail and `[]` clears aliases. Empty attribute, color, and position cells leave existing values unchanged. Reimporting an ID-bearing row updates the matching record within the same ingest source; provenance-bearing rows without IDs update only a unique natural-key match. When that source belongs to a saved SQL snapshot, imported rows must match the snapshot-owned records exactly, while analyst-note changes remain allowed; changed or new SQL-owned facts must be ingested from SQLite so the chart cannot diverge from its immutable snapshot. Rows without IDs or provenance append because they cannot be matched safely. Missing confidence defaults to 2; supplied confidence must be 1–3. Direction accepts true/false, 1/0, yes/no, directed/undirected, or →/↔; invalid supplied values reject the staged import. Use `source_attribute:<key>` and `target_attribute:<key>` columns for entity attributes; non-empty imported values update matching attributes. Repeated rows must agree on non-empty values for the same entity property or the staged import is rejected. Save/open local JSON case files; SQL snapshot history is retained with the case. The default case is restored at launch when it exists. Opening another case or creating a new case prompts before discarding unsaved edits. **Save case** writes to the active case file; use **Save as** to choose a different path, or edit the staged **Case file** path and use **Open case** to load it. Keyboard shortcuts: Ctrl/Command+1, +2, and +3 switch to Chart, Timeline, and Data; Ctrl/Command+S saves, Ctrl/Command+Shift+S saves as, Delete removes the selected chart item when a text field is not active, and Escape clears selection or dismisses the unsaved-changes prompt.

All bundled case examples are fictional. Arch Notebook does not connect to a service or transmit case data.

## Build and deploy

The supported desktop targets are Arch Linux on x86_64, generic Linux archives on x86_64 and ARM64, and macOS on Apple Silicon or Intel. Rust 1.89 or newer is required to build. macOS bundles SQLite so it does not depend on the older system SQLite library; Linux builds link to the system SQLite library, which is installed as an application dependency. On Linux, the application stores its database, CSV example, and default case under `$XDG_DATA_HOME/arch-notebook`, or `~/.local/share/arch-notebook` when `XDG_DATA_HOME` is unset. On macOS, it uses `~/Library/Application Support/Arch Notebook`. Existing sample files are left in place on later launches.

### Arch Linux

Build and install the current `main` branch as a native package with `makepkg`:

```sh
git clone https://github.com/pskeffington/arch_notebook.git
cd arch_notebook/packaging/arch
makepkg -si
```

The package includes the desktop menu entry and declares its shared-library dependencies. `makepkg` downloads the locked Cargo dependencies and builds the release binary for the host architecture.

### Generic Linux desktop archive

For a Linux distribution without an Arch package, download the matching `arch-notebook-linux-x86_64.tar.gz` or `arch-notebook-linux-aarch64.tar.gz` workflow/release artifact, then install it for the current user without `sudo`:

```sh
case "$(uname -m)" in
  x86_64) archive_arch=x86_64 ;;
  aarch64|arm64) archive_arch=aarch64 ;;
  *) echo "Unsupported Linux architecture: $(uname -m)" >&2; exit 1 ;;
esac
archive_dir=arch-notebook-linux-$archive_arch
tar -xzf "$archive_dir.tar.gz"
scripts/install-linux-user.sh "$archive_dir"
```

The installer puts the executable in `~/.local/bin` and the desktop launcher under `${XDG_DATA_HOME:-~/.local/share}/applications`. Add `~/.local/bin` to `PATH` to launch it from a terminal. To remove this user installation, delete `~/.local/bin/arch-notebook` and `${XDG_DATA_HOME:-$HOME/.local/share}/applications/arch-notebook.desktop`; case files and SQL databases are kept separately.

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

The script builds the native release binary by default. Set `ARCH_NOTEBOOK_SKIP_BUILD=1` only when the matching `aarch64-apple-darwin` or `x86_64-apple-darwin` release binary has already been built; CI uses this mode to package each matrix build without compiling it a second time.

The profile is created with `xcrun notarytool store-credentials`; the signing identity and credentials are not stored in this repository. See [Apple's notarization guide](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution) and [Developer ID signing overview](https://developer.apple.com/developer-id/).

### Continuous builds

GitHub Actions checks formatting and builds Linux x86_64 and ARM64 archives, macOS Apple Silicon and Intel apps, and an Arch Linux x86_64 package. Pull requests and pushes build without publishing a release; a newer push to `main` supersedes an older `main` build so CI prioritizes the latest commit. Generated workflow artifacts are available for 14 days. Publishing a GitHub Release triggers the workflow to attach the Linux archives, Arch package, macOS zips, and a `SHA256SUMS` file. macOS zip names state whether they are `unsigned`, `developer-id-signed`, or `notarized`; CI artifacts are unsigned because signing credentials are not stored in the repository.

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
