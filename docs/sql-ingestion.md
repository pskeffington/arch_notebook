# SQL snapshot ingestion

1. Open **Data → SQL regressions** and choose a SQLite database file.
2. Enter a stable **Snapshot series** label and a read-only query that returns `source` and `target` columns. Optional columns include `connection_id`, `source_entity_id`/`target_entity_id` (stable source-system entity keys), `source_type`, `target_type`, `relationship`, `date`, `confidence`, `directed`, `source_reference`, and `notes`. Column aliases are matched without case or surrounding whitespace; if a query returns multiple aliases for the same field (such as both `source` and `from`), ingestion reports the ambiguity instead of silently choosing one. Rows with a blank or NULL source or target are skipped and their count is retained with the snapshot and shown in validation, run history, and snapshot history. SQL `notes` are retained as source notes; analyst notes are stored separately and survive later refreshes. Missing confidence defaults to 2; supplied confidence must be an integer from 1 to 3. Direction accepts true/false, 1/0, yes/no, directed/undirected, or →/↔. In either the SQL regressions editor or automation job editor, **Build query from schema** inspects the selected database read-only and drafts a `SELECT` when it recognizes source and target columns. It inspects table columns through SQLite metadata and view columns through a prepared zero-row query, so analyst views can be used as query-builder sources without reading their records. If endpoint columns declare foreign keys to an entity table with a recognizable display name, the draft joins all columns in each declared key; composite keys are encoded without ambiguity as stable endpoint IDs. Otherwise it uses the endpoint columns directly. Generated left joins keep unmatched endpoints as NULL so ingestion can report them as skipped rows. Review and edit the generated query before ingestion.
   Bind changing query values with the SQL parameters field: enter one `name=value` per line and use `:name`, `@name`, or `$name` in the query. SQLite binds the values instead of interpolating them into SQL text. Unprefixed values bind as text; use `text:`, `int:`, `real:`, `bool:`, or `null:` to select a SQLite value type. Parameter values are saved with automation jobs, captures, and run history so an outcome can be reproduced.
3. Ingest a baseline query result. Update the query or database to the next period and ingest again using the same series label.
4. Review removed connections, confidence declines, changed event dates, additions, and confidence increases. Each changed connection appears once, with separate change labels for confidence, date, connection details, evidence references, and source notes. Filter the review by regression, direction, evidence, source-note, or other changes, and search across IDs, endpoint names/types, relationship labels, evidence, and notes. Comparison rows show typed endpoints, direction, evidence references, and notes before and after the change, including for removed connections. The **Show** action on a latest-snapshot row jumps to that current relationship on the chart; historical rows stay in comparison review because they may no longer exist on the current chart. Use **Export comparison CSV/TSV** to share the complete selected baseline/current delta report with other analysis tools; the export includes before/after facts, both snapshot IDs and capture times, and the skipped-row count for each capture, regardless of the on-screen filter. The **Review saved series** picker shows the number of chart links and endpoint entities from each SQL source. **Highlight source on chart** fits the chart to and emphasizes those links and entities while dimming unrelated chart records; the chart toolbar identifies the SQL series and database, and **View source records** returns to the Connections table with that source filter applied; choose **Review SQL snapshots** there to return to that source’s comparison history. The fields under **Ingest new snapshot** configure a separate new capture. The latest SQL snapshot also updates the chart for that database/series while preserving manually created and other-source links.

## Review snapshots

The saved-series review table can filter regressions, direction changes, evidence changes, source-note changes, or other updates, and search both sides of a change by connection ID/key, endpoint, type, date, evidence, or notes. Filtering only changes the on-screen rows; the comparison export always includes every change. The export retains a readable `change` label and adds a boolean column for each change type, so spreadsheet filters and downstream tools can select exact regression categories. Direction changes have their own `direction_changed` category while remaining included in `connection_details_changed` for existing consumers. The `connection_key` column exposes the same stable ID or natural key used to match each record across snapshots, including when the source query does not provide `connection_id`. Structured automation deltas use this same normalized key in each connection’s `key` field.

## Compare and export

In **Data → SQL regressions**, comparisons default to the immediately preceding capture in the selected series. Use **Compare with** to select any earlier capture in that same series for longer-range analysis; the selected baseline changes the on-screen delta table and CSV/TSV export together.

### CLI snapshot export

Export one immutable SQL capture without opening the desktop app. The output contains the captured connection fields plus snapshot provenance, query, typed parameter values, and skipped-row count on each record. An empty capture still exports its header row. A `.tsv` or `.tab` output path selects tab-separated output; other paths use CSV. Export is atomic and refuses to overwrite the case file or any saved SQLite source.

```sh
cargo run --offline -- --export-sql-snapshot harbor-light.case.json <snapshot-id> capture.csv --json
```

Get a snapshot UUID from `--list-runs --json` or the saved case's snapshot records. The JSON response reports output format, capture counts, source key and paths, series, query, and typed parameter values, so an empty capture still has its complete provenance in the response.

### CLI comparison export

Export a saved SQL comparison without opening the desktop app by passing the baseline and current snapshot UUIDs. The snapshots must be ordered in the same saved database/series. A `.tsv` or `.tab` output path selects tab-separated output; other paths use CSV. Export is atomic and refuses to overwrite a saved SQLite source database.

```sh
cargo run --offline -- --export-sql-comparison harbor-light.case.json <baseline-snapshot-id> <current-snapshot-id> comparison.csv --json
```

The JSON response reports the output format, snapshot IDs, and number of changed records. Get snapshot UUIDs from `--list-runs --json` or the case’s saved snapshot records.

## Exchange data

CSV/TSV and comparison reports are written to a temporary sibling file and atomically replace the destination only after the full export succeeds. **Import GraphML** stages nodes and relationships through the validated import path; it combines multiple top-level graphs into one case, flattens nested subgraphs while retaining each child entity’s `arch_notebook_parent_id` attribute, and imports recognized IDs, aliases, entity attributes, per-graph direction defaults, evidence, notes, and chart styling. **Export GraphML** works with graph tools such as Gephi and Cytoscape and includes endpoint IDs, direction, labels, evidence, notes, confidence, source provenance, entity attributes, and chart styling. Graph-level `source_catalog_json` maps each ingest-source key to its database and series, latest snapshot, exported link and distinct entity counts, and capture IDs with timestamps, connection counts, and skipped-row counts. Valid source-catalog entries survive GraphML import into the case file; re-export unions distinct capture IDs and prefers the newest capture metadata when a source key appears more than once. GraphML is an interchange format; CSV/TSV carries the full Arch Notebook case fields for round-trips. For scripts, import into an existing case or export without opening the desktop app:

```sh
arch-notebook --import-graphml case.json network.graphml --json
arch-notebook --export-graphml case.json chart.graphml --json
```

GraphML import uses the same validation and entity reconciliation as desktop import, locks the case against concurrent automation writes, and atomically saves only after the full import succeeds. The JSON response reports total entities and relationships in the updated case. GraphML export’s schema-versioned JSON response reports the output path and exported entity and relationship counts.

## Stable identity and safety

Connections with a unique, stable `connection_id` are matched across snapshots by that ID. A query result with duplicate IDs is rejected so ambiguous rows cannot distort regression comparisons. Without an ID, the app uses a natural key from source, target, relationship, types, and direction; duplicate natural keys in one result are rejected, so include a unique ID when the source contains parallel connections with otherwise identical fields. During CSV or SQL ingestion, entities are reconciled by name, a unique compatible alias, supplied type, and stable SQL source IDs. This keeps a merged entity connected to its imported records on refresh; changed source names for a stable ID are retained as searchable aliases without overwriting the chart label; ambiguous aliases remain separate for analyst review. Same-name entities of different types remain distinct while generic entities can gain a type from later imports.

### Read-only ingestion

SQLite databases open in read-only mode, and SQLite-classified writable statements are rejected before execution. The query cannot write to or create a database. Each capture reads its timestamp and query results in one SQLite read transaction, so a concurrent commit cannot make the timestamp and rows describe different database states. Each attempt waits up to five seconds for a SQLite write lock; transient `SQLITE_BUSY` or `SQLITE_LOCKED` errors retry up to three total capture attempts with two-second pauses, helping scheduled captures run through short source updates. A capture query is interrupted after 15 minutes, and automation records the timeout as a failed run so a runaway query cannot hold the case lock indefinitely. Scheduled Linux batches have no aggregate service timeout, so every enabled job can finish even when the total batch takes longer than an hour; each individual query remains capped at 15 minutes. The fictional seed files are archived under `archive/research.databases/`; `sample_connections.sql` creates baseline/current records demonstrating a removed connection and confidence decline. To build a disposable copy without overwriting the tracked archive, run:

CSV/TSV exports also refuse to replace a database used by a saved snapshot, automation job, or current SQL editor path.

```sh
## Sample database

sqlite3 /tmp/sample_connections.sqlite < archive/research.databases/sample_connections.sql
cargo run --offline
```

Select `/tmp/sample_connections.sqlite` as the source for the disposable copy, or open the archived `archive/research.databases/sample_connections.sqlite` directly. For the sample comparison, ingest once with the parameter `snapshot_id=baseline`, then change it to `snapshot_id=current` and ingest again with the same snapshot series.
