# Arch Notebook

[![Build](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/pskeffington/arch_notebook/actions/workflows/build.yml)

Arch Notebook is a local Rust desktop workbench for investigating relationship data and detecting connection regressions across SQL snapshots. It supports typed entity/link charts, evidence references, chronological analysis, shortest paths, duplicate review, and repeatable SQLite ingestion.

## SQL regression workflow

1. Open **Data → SQL regressions** and choose a SQLite database file.
2. Enter a stable **Snapshot series** label and a read-only query that returns `source` and `target` columns. Optional columns include `connection_id`, `source_entity_id`/`target_entity_id` (stable source-system entity keys), `source_type`, `target_type`, `relationship`, `date`, `confidence`, `directed`, `source_reference`, and `notes`. Column aliases are matched without case or surrounding whitespace; if a query returns multiple aliases for the same field (such as both `source` and `from`), ingestion reports the ambiguity instead of silently choosing one. Rows with a blank or NULL source or target are skipped and their count is retained with the snapshot and shown in validation, run history, and snapshot history. SQL `notes` are retained as source notes; analyst notes are stored separately and survive later refreshes. Missing confidence defaults to 2; supplied confidence must be an integer from 1 to 3. Direction accepts true/false, 1/0, yes/no, directed/undirected, or →/↔. In either the SQL regressions editor or automation job editor, **Build query from schema** inspects the selected database read-only and drafts a `SELECT` when it recognizes source and target columns. It inspects table columns through SQLite metadata and view columns through a prepared zero-row query, so analyst views can be used as query-builder sources without reading their records. If endpoint columns declare foreign keys to an entity table with a recognizable display name, the draft joins all columns in each declared key; composite keys are encoded without ambiguity as stable endpoint IDs. Otherwise it uses the endpoint columns directly. Generated left joins keep unmatched endpoints as NULL so ingestion can report them as skipped rows. Review and edit the generated query before ingestion.
   Bind changing query values with the SQL parameters field: enter one `name=value` per line and use `:name`, `@name`, or `$name` in the query. SQLite binds the values instead of interpolating them into SQL text. Unprefixed values bind as text; use `text:`, `int:`, `real:`, `bool:`, or `null:` to select a SQLite value type. Parameter values are saved with automation jobs, captures, and run history so an outcome can be reproduced.
3. Ingest a baseline query result. Update the query or database to the next period and ingest again using the same series label.
4. Review removed connections, confidence declines, changed event dates, additions, and confidence increases. Each changed connection appears once, with separate change labels for confidence, date, connection details, evidence references, and source notes. Filter the review by regression, direction, evidence, source-note, or other changes, and search across IDs, endpoint names/types, relationship labels, evidence, and notes. Comparison rows show typed endpoints, direction, evidence references, and notes before and after the change, including for removed connections. The **Show** action on a latest-snapshot row jumps to that current relationship on the chart; historical rows stay in comparison review because they may no longer exist on the current chart. Use **Export comparison CSV/TSV** to share the complete selected baseline/current delta report with other analysis tools; the export includes before/after facts, both snapshot IDs and capture times, and the skipped-row count for each capture, regardless of the on-screen filter. The **Review saved series** picker shows the number of chart links and endpoint entities from each SQL source. **Highlight source on chart** fits the chart to and emphasizes those links and entities while dimming unrelated chart records; the chart toolbar identifies the SQL series and database, and **View source records** returns to the Connections table with that source filter applied; choose **Review SQL snapshots** there to return to that source’s comparison history. The fields under **Ingest new snapshot** configure a separate new capture. The latest SQL snapshot also updates the chart for that database/series while preserving manually created and other-source links.

The saved-series review table can filter regressions, direction changes, evidence changes, source-note changes, or other updates, and search both sides of a change by connection ID/key, endpoint, type, date, evidence, or notes. Filtering only changes the on-screen rows; the comparison export always includes every change. The export retains a readable `change` label and adds a boolean column for each change type, so spreadsheet filters and downstream tools can select exact regression categories. Direction changes have their own `direction_changed` category while remaining included in `connection_details_changed` for existing consumers. The `connection_key` column exposes the same stable ID or natural key used to match each record across snapshots, including when the source query does not provide `connection_id`. Structured automation deltas use this same normalized key in each connection’s `key` field.

In **Data → SQL regressions**, comparisons default to the immediately preceding capture in the selected series. Use **Compare with** to select any earlier capture in that same series for longer-range analysis; the selected baseline changes the on-screen delta table and CSV/TSV export together.

Export a saved SQL comparison without opening the desktop app by passing the baseline and current snapshot UUIDs. The snapshots must be ordered in the same saved database/series. A `.tsv` or `.tab` output path selects tab-separated output; other paths use CSV. Export is atomic and refuses to overwrite a saved SQLite source database.

```sh
cargo run --offline -- --export-sql-comparison harbor-light.case.json <baseline-snapshot-id> <current-snapshot-id> comparison.csv --json
```

The JSON response reports the output format, snapshot IDs, and number of changed records. Get snapshot UUIDs from `--list-runs --json` or the case’s saved snapshot records.

CSV/TSV and comparison reports are written to a temporary sibling file and atomically replace the destination only after the full export succeeds. **Import GraphML** stages nodes and relationships through the validated import path; it combines multiple top-level graphs into one case, flattens nested subgraphs while retaining each child entity’s `arch_notebook_parent_id` attribute, and imports recognized IDs, aliases, entity attributes, per-graph direction defaults, evidence, notes, and chart styling. **Export GraphML** works with graph tools such as Gephi and Cytoscape and includes endpoint IDs, direction, labels, evidence, notes, confidence, source provenance, entity attributes, and chart styling. Graph-level `source_catalog_json` maps each ingest-source key to its database and series, latest snapshot, exported link and distinct entity counts, and capture IDs with timestamps, connection counts, and skipped-row counts. Valid source-catalog entries survive GraphML import into the case file; re-export unions distinct capture IDs and prefers the newest capture metadata when a source key appears more than once. GraphML is an interchange format; CSV/TSV carries the full Arch Notebook case fields for round-trips. For scripts, import into an existing case or export without opening the desktop app:

```sh
arch-notebook --import-graphml case.json network.graphml --json
arch-notebook --export-graphml case.json chart.graphml --json
```

GraphML import uses the same validation and entity reconciliation as desktop import, locks the case against concurrent automation writes, and atomically saves only after the full import succeeds. The JSON response reports total entities and relationships in the updated case. GraphML export’s schema-versioned JSON response reports the output path and exported entity and relationship counts.

Connections with a unique, stable `connection_id` are matched across snapshots by that ID. A query result with duplicate IDs is rejected so ambiguous rows cannot distort regression comparisons. Without an ID, the app uses a natural key from source, target, relationship, types, and direction; duplicate natural keys in one result are rejected, so include a unique ID when the source contains parallel connections with otherwise identical fields. During CSV or SQL ingestion, entities are reconciled by name, a unique compatible alias, supplied type, and stable SQL source IDs. This keeps a merged entity connected to its imported records on refresh; changed source names for a stable ID are retained as searchable aliases without overwriting the chart label; ambiguous aliases remain separate for analyst review. Same-name entities of different types remain distinct while generic entities can gain a type from later imports.

SQLite databases open in read-only mode, and SQLite-classified writable statements are rejected before execution. The query cannot write to or create a database. Each capture reads its timestamp and query results in one SQLite read transaction, so a concurrent commit cannot make the timestamp and rows describe different database states. Reads wait up to five seconds for a SQLite write lock before returning an error, which helps scheduled captures run through short source updates. A capture query is interrupted after 15 minutes, and automation records the timeout as a failed run so a runaway query cannot hold the case lock indefinitely. The fictional seed files are archived under `archive/research.databases/`; `sample_connections.sql` creates baseline/current records demonstrating a removed connection and confidence decline. To build a disposable copy without overwriting the tracked archive, run:

CSV/TSV exports also refuse to replace a database used by a saved snapshot, automation job, or current SQL editor path.

```sh
sqlite3 /tmp/sample_connections.sqlite < archive/research.databases/sample_connections.sql
cargo run --offline
```

Select `/tmp/sample_connections.sqlite` as the source for the disposable copy, or open the archived `archive/research.databases/sample_connections.sqlite` directly. For the sample comparison, ingest once with the parameter `snapshot_id=baseline`, then change it to `snapshot_id=current` and ingest again with the same snapshot series.

## Automate repeatable ingestion

In **Data → Automation**, **Unreviewed regressions** filters to captures with removals or confidence declines and no analyst comment, including runs that passed configured limits. **Unreviewed failures** remains available for failures without comments.

For an ad-hoc desktop run, expand **One-run SQL parameter overrides** and enter `name=value` lines. Overrides apply to the next run that selects exactly one job, are retained in that run's configuration for replay and audit, then clear from the editor. Clear them before a multi-job batch; they do not modify saved jobs or scheduled captures.

The validated query preview shows up to eight rows with endpoint names and source IDs, relationship, direction, date, confidence, evidence, and SQL supplied source notes. A prominent result-quality label distinguishes usable results, partial skips, empty results, and cases where every row was skipped, even when a zero-row query passes configured limits. The preview reports removals and confidence declines against the previous capture in the same series, labels a first capture as the baseline, and lists up to eight changes of any type in a **Change preview** with typed endpoints, stable connection IDs/keys, before/current confidence and dates, and evidence/source-note values on hover. The regression count remains separate for evaluating capture limits. Use **Regressions only** in the preview to focus on removals and confidence declines. The preview is horizontally scrollable for wide schemas. In **SQL regressions**, set optional automation capture limits, validate the current query and enter a job name before using **Save and review in Automation** to carry the database, series, query, parameters, and capture limits into a repeatable job without re-entering them. Expand **Automation capture limits** to set optional minimum/maximum connections, regression, or skipped-row thresholds before validating. This handoff is enabled only for a usable result with no skipped endpoints and passing capture limits; the new job starts disabled until you review and enable it in Automation; disabled rows are labeled and expose Validate/Edit actions.

In both the automation editor and one-off SQL snapshot panel, **Build query from schema** inspects the configured SQLite file read-only and generates an editable `SELECT` from the first table or view with recognizable source and target columns. Review and adjust the selected columns before validating or saving the query.

When endpoint IDs have declared foreign keys to entity tables, generated queries join all columns in those keys and select a recognizable name column (and entity type when available). Composite endpoint IDs encode each SQLite storage type and hexadecimal value bytes, so embedded NULs and separator characters remain unambiguous and text, blob, and numeric IDs stay distinct. Without a usable foreign key, the template falls back to direct endpoint columns or the endpoint IDs themselves. View schemas are inspected without reading records, allowing the builder to recognize source and target aliases exposed by analyst-authored views.

If a schema has no declared foreign keys, the builder infers an ID join only when one unambiguous entity-like table has a single primary key and a recognizable name column. Ambiguous schemas keep the endpoint IDs for manual query editing. Generated results retain endpoint IDs alongside display names so same-named source entities stay distinct and snapshots can match those identities over time.

SQL can return stable endpoint IDs as `source_entity_id`/`target_entity_id`, `source_external_id`/`target_external_id`, or common endpoint names such as `target_id`, `from_id`/`to_id`, `entity_a_id`/`entity_b_id`, `source_person_id`/`target_person_id`, `from_person_id`/`to_person_id`, `source_node_id`/`target_node_id`, and `from_node_id`/`to_node_id`. The `source_id` column name remains available for evidence references. These IDs preserve entity identity across captures when an entity’s display name changes.

When multiple tables or views match, the editor offers a source picker so you can choose which generated query to use. The CLI’s `query_template` is the first matching option; the full schema remains available in `objects`.

Applying a generated template clears the parameter editor because the draft contains no named placeholders; add any required `WHERE` clause and parameters before validation.

Open **Data → Automation**, enter a unique job name, SQLite database path, snapshot series, and query, then use **Validate query** to check the read-only query, returned connection count, and configured limits against the latest saved snapshot without saving case changes. A successful validation also previews up to eight returned connections so you can confirm the endpoint mapping before ingestion. The status distinguishes an invalid query from a valid query whose configured limits fail; `--validate-enabled-jobs` applies the same capture-limit assessment as a job run, while `--check-enabled-job-queries` checks query validity without enforcing those limits. Each database/series identity can belong to only one automation job; aliases to the same existing database file resolve to the same identity. This keeps one query stream from replacing another job’s chart links or mixing unrelated results in a comparison chain. Relative database paths resolve from the case file’s directory. Snapshots retain both the canonical database path and the configured path; when a case and its relative-path database move together, loading the case rebases its saved series, imported-link provenance, and SQL entity identity namespaces to the new location. When loading a case, paths in earlier snapshots are normalized too when their source databases are still available. Set **Minimum connections** to require a nonzero result size; the default of 0 disables this check. A result below the configured minimum is recorded as a failed run, while its successfully captured snapshot remains in history for review. Optionally enable **Limit connections** and set a maximum accepted result size; an oversized result is retained as a snapshot for review but the run fails. Optionally enable **Limit regressions** and set a maximum count; a regression is a removed connection or confidence decline versus the preceding snapshot in the same database/series. The baseline snapshot has no regressions to compare, and runs still save their snapshot and history when a limit is exceeded. Optionally set **Limit skipped rows** and a maximum count to fail runs when the SQL result has too many rows with blank or NULL endpoints. The skipped-row count and threshold are retained with the run, while the successfully captured snapshot stays available for review. Add the job when it is ready. The Automation list shows each job’s latest run status, capture quality, connection/skipped-row counts, and duration. Failed jobs also show a concise error summary inline; hover for the full result message. Use **View latest run** to filter and expand that exact run in history. Search the list by job name, database path, snapshot series, or SQL text without changing saved jobs or run history. Expand **SQL query and capture configuration** to inspect a saved job without loading it into the editor. Use **Validate** beside a saved job to preflight its query and limits without changing the case; **Edit** loads its configuration into the editor. Enable the jobs you want included in a batch run. Filter the job list, then choose **Run visible enabled jobs and save** to run only the matching enabled jobs; the unfiltered **Run enabled jobs and save** action still runs every enabled job. Running an individual job or batch keeps the current editor fields intact. Each run records its status and timestamps in the case and appends a new immutable snapshot when the query succeeds. Save case and job edits before starting automation. Desktop runs execute on a background worker and show progress while pausing case editing; the worker holds the case lock and atomically saves each job checkpoint, including failed run records, so a slow SQL query does not freeze the window or lose completed work. One-off SQL validation and ingestion also run in the background; validation leaves the case unchanged, while ingestion updates the open case and still requires an explicit save. The command-line runner executes enabled jobs without opening the desktop UI and saves the updated case automatically:

Set the maximum to 0 to reject any skipped row. Older jobs default to no skipped-row limit. Expand a run in **Run history** to review its capture quality, connection and skipped-row counts, snapshot ID, and delta counts against the prior snapshot in that database/series. Use **Review changes** to open that exact capture pair in SQL comparison review, with its filters reset to show every change.

Use SQLite named parameters when a scheduled query needs a changing period, tenant, or threshold instead of editing the SQL each run. Enter one `name=value` per line in **SQL parameters** and reference it in the query with `:name`, `@name`, or `$name`. Unprefixed values bind as text; type a value with `text:`, `int:`, `real:`, `bool:`, or `null:` when SQLite should receive text, a 64-bit integer, a finite real, 0/1, or NULL. For example, use `snapshot_id=current`, `min_confidence=int:2`, and `include_unknown=bool:false`; `null:` binds SQL NULL and `text:null` binds the literal text `null`. Parameters are saved with the automation job, each immutable snapshot, and each run for reproducibility. For a one-off CLI run or preflight, repeat `--param name=value` to override or add parameter values without changing the saved job. For example, use `WHERE snapshot_id = :snapshot_id` with `snapshot_id=current`.

Desktop batch runs save an atomic checkpoint after each job, then save the final batch summary. If a later job or checkpoint step fails, the open app reloads the latest saved checkpoint and keeps the regression view focused on its newest snapshot. The command-line runner also saves after each job, preserving completed run history if a later query stalls or the process is interrupted.

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

Automation run history keeps a separate analyst comment for review context without changing the captured SQL result or machine-generated run message. In **Data → Automation**, expand a run, edit **Analyst comment**, then choose **Save comment** and save the case. For scheduler or review scripts, add or replace a comment using the run UUID from `--list-runs`:

```sh
cargo run --offline -- --annotate-run harbor-light.case.json <run-id> "Reviewed against the source release notes; expected removals."
cargo run --offline -- --list-runs harbor-light.case.json --json
```

Run-list JSON includes `analyst_note`; pass an empty comment to clear it. Comment updates lock and atomically save the case.

Share a saved SQL automation definition without copying the case, captured snapshots, or run history. Export by job name or UUID, then import the JSON package into another case:

```sh
cargo run --offline -- --export-job harbor-light.case.json "Harbor Light baseline" harbor-light-baseline.job.json --json
cargo run --offline -- --import-job another-case.json harbor-light-baseline.job.json --json
cargo run --offline -- --import-job another-case.json harbor-light-baseline.job.json --name "Local baseline" --database ./data/connections.sqlite --series "Local review" --json
cargo run --offline -- --validate-job another-case.json "Local baseline" --query-only --param snapshot_id=current --json
```

The versioned package carries the job name, configured database path, snapshot series, SQL query, typed parameters, and capture limits. Import assigns a new job UUID, starts the job disabled, and rejects a duplicate job name or a database/series already owned by another job. CLI imports can override the name, database path, or series when adapting a shared query to another environment; relative database paths resolve from the destination case file. Use `--validate-job` to preflight a named or UUID-selected job even while it is disabled. The default checks both query validity and its capture limits; add `--query-only` to report query validity without failing on unmet capture limits. Repeat `--param name=value` to override or add bound SQL parameters for this validation only; the saved job and case remain unchanged. Validation is read-only and does not save the case. In the desktop app, use the separate **Import JSON** and **Export JSON** paths in **Data → Automation**; expand **Adapt imported job** to override the name, database path, or series, and use **Export** beside a job to write its package. After import, the job list focuses on the new disabled entry and the status displays its UUID for review and script handoff. Use **Validate** before enabling it for a scheduled batch; package import does not connect to or modify its SQLite source. Export writes atomically and refuses to replace the case or a SQLite source used by its jobs or snapshots.

Read saved automation outcomes without opening the desktop app:

```sh
cargo run --offline -- --list-runs harbor-light.case.json --json
```

The result uses schema version 1 and lists newest runs first with status, job identity, timestamps, elapsed query and validation duration in milliseconds, database/series, the captured query and bound parameters, thresholds, enabled state, outcome message, linked snapshot ID, capture time, connection count, and skipped-row count when a snapshot was produced. It also returns the structured limit checks, `query_valid`, `capture_limits_pass`, and typed snapshot delta with stable connection keys and prior/current facts, matching run and preflight outcomes. Older runs without recorded configuration return `limits` and `capture_limits_pass` as null rather than implying their default fields were saved settings; runs from older case files report a null duration. Listing run history does not modify the case.

Filter by job name or UUID and page through newest-first history with `--job <name-or-id>`, `--offset <count>` (default `0`), and `--limit <count>` (positive integer). JSON responses include the active filter, total matches, returned count, and `next_offset` when another page exists. The selection is applied before snapshot deltas are computed.

Inspect a SQLite source before writing an ingestion query. This command opens the database read-only and lists its tables, views, column names, declared types, and key/required-column markers:

```sh
cargo run --offline -- --inspect-sqlite source.sqlite --json
```

The JSON response uses schema version 1 and includes `ok`, `object_count`, table/view definitions, declared foreign-key mappings, and `query_templates` for every table/view with recognizable source and target columns. The existing singular `query_template` field remains as the first candidate for compatibility. It reports structure only; it does not read table rows or modify the source. The text output reports when the database contains no user tables or views, lists declared foreign keys, and prints the first suggested query when one can be generated.

To check enabled SQL queries and their SQLite sources without requiring current data to meet capture thresholds, use query-only preflight. It executes queries read-only and leaves the case unchanged; failing connection or regression limits are reported but do not fail the query check. A zero-connection result is still query-valid, but is classified as `empty_result` or `all_rows_skipped` and reported as having no usable connections, even when no minimum is configured. Desktop live checkpoints and run-history rows also show the result quality. Validation and query preflight acquire the per-case automation lock, so they wait for an active runner’s checkpoint and read a stable case snapshot. JSON output sets top-level `mode` to `query` and reports `query_valid`, `result_quality`, and `has_usable_connections` separately from each outcome’s normal run `status` and `capture_limits_pass`:

```sh
cargo run --offline -- --check-enabled-job-queries harbor-light.case.json
```

Add `--require-usable` to fail preflight when any selected query returns no usable connections. The Linux and macOS schedule installers use this stricter check, so an empty or all-skipped enabled job cannot be scheduled accidentally. This option does not enforce regression or capture thresholds; those remain visible runtime outcomes.

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

Run history is bounded to the latest 200 records. Search the history by job, run ID, PASS/FAIL result, result message, analyst comment, database path, series, SQL text, or time. Use **Failures only** to show unsuccessful runs or **Unreviewed failures** to show failures without an analyst comment; these filters do not change saved history. Plain-text `--list-runs` output summarizes result quality, connection and skipped-row counts, regression counts, duration, and run ID; use `--json` for the full machine-readable capture and delta details. Each run retains the job configuration and SQL query used, including failures, so later job edits or deletion do not erase the run context; expand a row in the desktop history to inspect it. Select **Replay saved configuration** to run that exact saved SQL, parameters, and thresholds against the current SQLite source; replay appends a new run and snapshot without changing the job definition. Use **Load configuration into editor** to revise a saved run before execution; if its original job was deleted, the app opens a uniquely named recovered job draft; saving adds it disabled, filters the job list to that recovered job, and leaves it for you to review and enable. The new run records the ID of the run it replayed, in both desktop history and `--list-runs --json` output. Older runs that predate this configuration capture are labeled as such rather than showing default values as if they were recorded settings. Successful runs include a comparison with the previous snapshot in the same database and series; the history records removed connections, confidence declines, date changes, additions, confidence increases, and other updates. Each snapshot has a stable ID, and automation run records link directly to it; older cases migrate those links from their saved snapshot indices when loaded. In the desktop app, use **Review snapshot #N** beside a run or select any numbered capture in **Snapshot history** to compare it with the preceding capture in that series; the capture number disambiguates runs recorded within the same clock interval. Batch summaries report passed and failed job counts. The runner continues through the selected jobs if one fails, saves each outcome, and exits nonzero when any job fails. Filter the desktop job list by name, database, series, or SQL, then use **Run visible enabled jobs and save** to run only the enabled jobs shown without changing their enabled settings.

In the desktop app, open **Data → Automation** and select **Check schedule** to see whether the open case's timer or LaunchAgent is active and which cadence it uses. Choose hourly, daily, or weekly, then select **Install/update schedule** to apply it. Each case has its own schedule, so multiple cases can run independently; updating one case leaves other case schedules in place. The app uses the saved case, checks enabled SQL queries in read-only mode, and installs or updates that case's Linux systemd timer or macOS LaunchAgent. Save case and job edits before installing. Select **Remove schedule** to disable and remove the open case's schedule. To manage schedules manually, use the platform commands below.

On Arch Linux with systemd, configure enabled jobs in the app, then install the hourly timer for the default case:

```sh
arch-notebook-install-automation
```

Pass a case-file path to schedule it; the path is canonicalized to assign a stable per-case unit or LaunchAgent ID. Set `ARCH_NOTEBOOK_AUTOMATION_SCHEDULE=daily` or `weekly` before running either platform installer to change the cadence; hourly is the default. Rerunning an installer updates only that case's schedule. Both installers require at least one enabled job and use query-only preflight against each read-only SQLite source before creating or replacing that schedule. Current connection and regression limits remain runtime outcomes, so a real regression does not prevent its schedule from being installed. If query preflight fails, fix the job or make its database available; existing schedules remain active. Scheduler invocations print concise per-job status lines to keep journald and macOS log files from accumulating full connection deltas; add `--json` to manual CLI runs when structured output is needed. The Linux installer writes user-owned, case-specific units under `${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user` and enables or restarts the timer after preflight. It catches up a missed calendar run after the machine was offline and adds up to five minutes of randomized delay. User timers run only while the user systemd manager is active; to keep them running after logout and start them at boot, enable lingering for the account with `loginctl enable-linger "$USER"`. Each install prints its timer and service names. Check or remove one case with `arch-notebook-install-automation --status /path/to/case.json` or `arch-notebook-install-automation --remove /path/to/case.json`; omitting the case path addresses only the legacy single schedule. On macOS, first open the app and enable jobs, then install the bundled LaunchAgent for the case:

```sh
app_bundle="/Applications/Arch Notebook.app"
bash "$app_bundle/Contents/Resources/install-macos-automation.sh" \
  "$HOME/Library/Application Support/Arch Notebook/harbor-light.case.json" \
  "$app_bundle"
```

After query preflight succeeds, the macOS installer creates and lints a case-specific LaunchAgent plist before unloading that case's previous agent. If the replacement cannot be loaded, it restores the prior plist and attempts to reload it. The agent runs once when loaded and then at the selected interval. The bundled runner keeps each case's logs private to the current account, rotates each at 5 MiB, and retains five older files under `~/Library/Logs/ArchNotebook/`; use the case schedule ID printed by the installer to locate the directory. Check or remove one case with `bash "$app_bundle/Contents/Resources/install-macos-automation.sh" --status "/path/to/case.json"` or `--remove "/path/to/case.json"`; omitting the case path addresses only the legacy single schedule. Keep the SQLite source available at the configured path when automation runs.

## Other notebook workflows

- Build and edit entity/link charts, choose both endpoints when creating a link, inspect attributes and source references, arrange with connectivity, breadth-first hierarchy, or alphabetical grid layouts while pinning selected entity positions, and review timelines with ISO dates positioned by elapsed time. Use **Fit all** to frame every entity after manual rearrangement; path analysis can fit the full shortest path separately. The chart navigator shows the full network and current viewport; click a region in its overview to jump there. Parallel relationship records render on separate chart lanes and can be selected individually.
- Find weakly connected NETWORK GROUPS from the Analysis sidebar, treating directed links as undirected for discovery. Show a group to fit it on the chart and highlight its entities and internal links.
- Review the grouped Connections table by entity pair and relationship type, then expand a pair to inspect its link records. The selected-entity inspector groups linked entities and their records; focus a counterpart or an individual dated/directed record on the chart. The selected-link inspector identifies its SQL series and database and opens that series directly in snapshot comparison review. Entity connection counts remain distinct counterpart entities; the chart’s MOST CONNECTED ranking can compare distinct entities, all link records, directed incoming links, or directed outgoing links and filter each measure by relationship type. Highlight top 10 to fit those entities on the chart and emphasize their incident links. Parallel link records count separately in the link-based measures, and undirected links are excluded from incoming/outgoing rankings. Switch to the Link records table for a sortable row view by record ID, source, relationship, target, date, confidence, or source reference. Filter across endpoints, evidence, notes and ingest provenance; scope results to all connections or those involving the selected entity, then return a selected link to the chart.
- Search and sort entity records, focus chart entities from the most-connected list, and fit the full shortest path into the chart view.
- Review likely duplicate entities of compatible types before merging. Merge rewires links and preserves aliases, attributes, notes, and the original record in entity history.
- CSV/TSV import inspects the full file on a background worker, reports rows scanned and the total data-row count, and displays the first four records in the mapping preview without blocking the chart UI. Cancel a long inspection from the toolbar; scanning stops between records.
- Import/export relationship CSV or TSV (`.tsv` and `.tab` paths use tab delimiters) with optional `connection_id`, `ingest_source`, and entity properties retained across round-trips. Exports include `source_entity_id` and `target_entity_id` UUID columns so reimport preserves chart identity even when names are duplicated, plus `source_external_ids_json` and `target_external_ids_json` for source-system identities used by SQL ingestion. **Import CSV/TSV** opens a design dialog with a row preview that shows transformed values, whether each sample passes the active row filters, flags incomplete filters or transforms, and disables Import until endpoint mappings, filters, and transforms are valid, plus field mapping for source/target, entity IDs, types, relationship, date, confidence, direction, identifiers, notes, provenance, endpoint details, aliases, colors, chart positions, position locks, and custom entity attributes. Duplicate headers, ignoring case and surrounding whitespace, are rejected so a mapping cannot silently choose the wrong column. Save all-match row filters (equals, contains, is empty, or is not empty), optional whitespace trimming, and per-column value transforms with each design; transforms can replace substrings, map exact values, or normalize case, run in saved list order before row filters, and appear in the import preview with the original value available on hover. Exact-value mappings standardize categories without altering partial matches in identifiers. Skipped rows are reported. Save named import designs in the case file; a unique design saved for the same set of column names is applied automatically, while multiple matches remain available for manual selection. Header order, case, and surrounding whitespace do not affect the match; standard headers are detected automatically. Rows marked `record_type=entity` carry isolated entities; connection rows leave `record_type` blank or use `connection`. The `notes` column carries analyst notes; `source_notes` carries notes supplied by SQL or another source. If a matching row is reimported without a `source_notes` column, its saved source notes are preserved; a present but blank `source_notes` cell clears them. Endpoint detail, aliases, chart color, and normalized chart position use `source_detail_json`/`target_detail_json`, `source_aliases_json`/`target_aliases_json`, `source_color`/`target_color` (`#RRGGBB`), and `source_x`/`source_y`/`target_x`/`target_y` (0–1), and `source_position_locked`/`target_position_locked` (true/false); JSON `""` clears detail and `[]` clears aliases. Empty attribute, color, and position cells leave existing values unchanged. Reimporting an ID-bearing row updates the matching record within the same ingest source; provenance-bearing rows without IDs update only a unique natural-key match. When that source belongs to a saved SQL snapshot, imported rows must match the snapshot-owned records exactly, while analyst-note changes remain allowed; changed or new SQL-owned facts must be ingested from SQLite so the chart cannot diverge from its immutable snapshot. Rows without IDs or provenance append because they cannot be matched safely. Missing confidence defaults to 2; supplied confidence must be 1–3. Direction accepts true/false, 1/0, yes/no, directed/undirected, or →/↔; invalid supplied values reject the staged import. Use `source_attribute:<key>` and `target_attribute:<key>` columns for entity attributes; non-empty imported values update matching attributes. Repeated rows must agree on non-empty values for the same entity property or the staged import is rejected. Save/open local JSON case files; SQL snapshot history is retained with the case. The default case is restored at launch when it exists. Opening another case or creating a new case prompts before discarding unsaved edits. **Save case** writes to the active case file; use **Save as** to choose a different path, or edit the staged **Case file** path and use **Open case** to load it. Keyboard shortcuts: Ctrl/Command+1, +2, and +3 switch to Chart, Timeline, and Data; Ctrl/Command+4 opens Automation directly and Ctrl/Command+5 opens SQL review directly; Ctrl/Command+S saves, Ctrl/Command+Shift+S saves as, Delete removes the selected chart item when a text field is not active, and Escape clears selection or dismisses the unsaved-changes prompt.

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

The package includes the desktop menu entry and declares its shared-library dependencies. `makepkg` downloads the locked Cargo dependencies and builds the release binary for the host architecture. After opening the app and configuring enabled automation jobs, install the hourly user timer with:

```sh
arch-notebook-install-automation
```

Pass a case file as the first argument to schedule a different case. The installer writes user-owned systemd units under `${XDG_CONFIG_HOME:-~/.config}/systemd/user`, enables the timer, and validates the case before installation.

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
"$archive_dir/install-linux-user.sh" "$archive_dir"
```

The installer puts the executable and its automation installer in `~/.local/bin`, and the desktop launcher under `${XDG_DATA_HOME:-~/.local/share}/applications`. Add `~/.local/bin` to `PATH` to launch it from a terminal. The companion installer lets the app’s **Install/update schedule** button configure systemd after the extracted archive has been removed. To remove this user installation, delete `~/.local/bin/arch-notebook`, `~/.local/bin/arch-notebook-install-automation`, and `${XDG_DATA_HOME:-$HOME/.local/share}/applications/arch-notebook.desktop`; case files and SQL databases are kept separately.

To schedule hourly SQL ingestion, first launch the app and enable the desired jobs in the case. Then run the bundled installer with the case path and binary path if `~/.local/bin` is not on `PATH`:

```sh
"$archive_dir/install-linux-automation.sh" \
  "${XDG_DATA_HOME:-$HOME/.local/share}/arch-notebook/harbor-light.case.json" \
  "$HOME/.local/bin/arch-notebook"
```

The installer creates and enables a user-level systemd timer. Inspect it with `systemctl --user status arch-notebook-automation.timer`; job output is available through `journalctl --user -u arch-notebook-automation.service`.

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


## Repository sweep and change control

Documentation and maintenance passes follow the generalized [repository sweep decision-routing policy](https://github.com/pskeffington/agent_skills/blob/main/docs/REPO_SWEEP_DECISION_ROUTING.md). Build, packaging, hardware, visual, and release claims remain separately evidenced; a passing source build does not by itself establish interactive or production acceptance.
