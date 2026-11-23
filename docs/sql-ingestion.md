# SQL snapshot ingestion

## Map relational records into a network

For a common transaction schema with `accounts(id, owner_name, institution)` and `transfers(id, sender_account_id, recipient_account_id, transfer_type, occurred_at, amount, status)`, join each event to its endpoint records and return one row per relationship. A disposable synthetic database matching this schema is provided in `archive/research.databases/relational_transfers.sql`:

```sh
sqlite3 /tmp/relational-transfers.sqlite < archive/research.databases/relational_transfers.sql
arch-notebook --inspect-sqlite /tmp/relational-transfers.sqlite --json
```

The schema builder recognizes the transfer table’s sender/recipient account foreign keys and uses `owner_name` for endpoint labels, so it can suggest a joined query from this fixture.

The query keeps account identity stable across refreshes, gives each transfer its own connection identity, and exposes fields for chart inspection and regression comparison:

```sql
SELECT
    sender.owner_name AS source,
    CAST(sender.id AS TEXT) AS source_entity_id,
    'Account' AS source_type,
    sender.institution AS "source_attribute:institution",
    recipient.owner_name AS target,
    CAST(recipient.id AS TEXT) AS target_entity_id,
    'Account' AS target_type,
    recipient.institution AS "target_attribute:institution",
    CAST(transfer.id AS TEXT) AS connection_id,
    transfer.transfer_type AS relationship,
    transfer.occurred_at AS date,
    3 AS confidence,
    1 AS directed,
    transfer.status AS notes,
    transfer.amount AS "connection_attribute:amount"
FROM transfers AS transfer
JOIN accounts AS sender
  ON sender.id = transfer.sender_account_id
JOIN accounts AS recipient
  ON recipient.id = transfer.recipient_account_id
WHERE transfer.occurred_at >= :start_date
  AND transfer.occurred_at < :end_date
ORDER BY transfer.occurred_at, transfer.id;
```

Set **SQL parameters** to `start_date=text:2025-01-01` and `end_date=text:2025-02-01` (one per line). The half-open interval includes the start and excludes the end, so adjacent monthly captures do not overlap. Reuse the query and snapshot series while changing the parameter values to compare periods. The transfer ID matches parallel transfers independently; the account IDs preserve endpoint identity even if an owner name changes. If source IDs are only unique within a tenant or database partition, include that scope in the entity ID expression, such as `CAST(transfer.tenant_id AS TEXT) || ':' || CAST(sender.id AS TEXT)`. Check that the source columns use compatible date formatting and that each connection ID is unique in the selected period before capture.

To walk through two periods, capture January with the values above, then capture February using `start_date=text:2025-02-01` and `end_date=text:2025-03-01` with the same database and series. To turn the query into a recurring job, save it from **Automation** with a stable series, then replace the two parameter predicates with SQLite's UTC calendar-month window. Remove the `start_date` and `end_date` values from **SQL parameters** before validating, because the recurring query no longer uses those bindings:

```sql
WHERE transfer.occurred_at >= date('now', 'start of month', '-1 month')
  AND transfer.occurred_at < date('now', 'start of month')
```

Each scheduled capture then reads the previous complete UTC month without changing the saved SQL text or parameters. Choose a schedule that runs after the source has finished loading that month; run history retains the capture time and immutable query result for review.

SQL queries can also capture custom relationship properties by aliasing values as `connection_attribute:<name>`, for example `transaction.amount AS "connection_attribute:amount"`. They persist with immutable snapshots, appear in validation and before/after comparisons, and are included in snapshot and comparison CSV/TSV exports. Use the **Relationship properties** filter to isolate captures where these values changed. Properties appear in chart search and the selected-link inspector; SQL-owned values refresh from each query and are read-only there. The schema query builder recognizes columns prefixed `connection_attribute_` or `link_attribute_` and aliases them for capture.

1. Open **SQL Review** and choose a SQLite database file.
2. Enter a stable **Snapshot series** label and a read-only query that returns `source` and `target` columns. Optional columns include `connection_id`, `source_entity_id`/`target_entity_id` (stable source-system entity keys), `source_type`, `target_type`, `relationship`, `date`, `confidence`, `directed`, `source_reference`, and `notes`. Custom endpoint properties can be returned by aliasing expressions as `source_attribute:<name>` or `target_attribute:<name>`, for example `source.latitude AS "source_attribute:latitude"` and `target.longitude AS "target_attribute:longitude"`. Non-empty values update chart entity attributes, persist in immutable snapshot facts, appear in structured deltas when changed, and are included in snapshot CSV/TSV exports. Snapshot captures retain SQLite `schema_version` from the same read transaction as the query. SQL Review flags when the selected baseline and current snapshot have different schema versions so structural source changes are visible alongside row-level deltas; the value is a drift signal, not a full schema fingerprint. Snapshot comparisons classify source or target attribute drift as `entity_properties_changed`, separately from connection-level `connection_properties_changed`; use the **Entity properties** or **Relationship properties** filter to inspect either category. JSON delta counts and comparison CSV/TSV expose these categories separately. On refresh, that source’s previous property values are removed before the latest result is applied, so omitted columns and NULL/blank values do not leave stale chart data. Values from other SQL sources remain in effect; when the last SQL source stops supplying a property, its prior chart value is restored if one existed. The entity inspector marks SQL-owned properties read-only and identifies the supplying series and database path; the entity table shows the same provenance on hover; CSV imports update the fallback chart value without changing the active SQL capture; reimporting the same visible SQL value preserves the existing fallback. Repeated rows for one chart entity must agree on each non-empty property value; conflicts fail the staged capture. Column aliases are matched without case or surrounding whitespace; if a query returns multiple aliases for the same field (such as both `source` and `from`), ingestion reports the ambiguity instead of silently choosing one. Rows with a blank or NULL source or target are skipped and their count is retained with the snapshot and shown in validation, run history, and snapshot history. Validation also offers up to eight skipped-row examples with the connection ID (when present) and endpoint values, so a query can be adjusted before capture. Those examples exist only in the transient validation preview; they are not added to the case or snapshot history. SQL `notes` are retained as source notes; analyst notes are stored separately and restored on refresh by stable connection ID, including for parallel links. If an existing case has duplicate SQL refresh keys, refresh stops with an error instead of assigning one analyst note ambiguously. Missing confidence defaults to 2; supplied confidence must be an integer from 1 to 3. Direction accepts true/false, 1/0, yes/no, directed/undirected, or →/↔. In either the SQL regressions editor or automation job editor, **Build query from schema** inspects the selected database read-only and drafts a `SELECT` when it recognizes source and target columns. It inspects table columns through SQLite metadata and view columns through a prepared zero-row query, so analyst views can be used as query-builder sources without reading their records. The schema inventory search matches object type/name, object SQL definitions, column names, foreign-key target tables, index names and SQL definitions, and unavailable-view errors, making it easier to trace dependencies and identify drift. A view that cannot be prepared because its definition is stale or depends on unavailable schema is reported as unavailable and skipped; other tables and views remain available for template generation. When endpoint ID columns declare foreign keys, the draft joins every column in each declared key. It selects a recognized display field when available and otherwise uses the table-scoped key as the endpoint label; stable endpoint IDs are scoped to the referenced table and encode SQLite storage types and bytes. This keeps equal local IDs from different entity tables distinct and supports composite keys. Without a declared foreign key, it uses the endpoint columns directly. When a joined display field is selected, an unmatched endpoint can evaluate to NULL and be counted as a skipped row; if no display field is recognized, the query uses the endpoint key as its label. Review the generated mapping and validate to inspect any skipped examples before ingestion.
   Bind changing query values with the SQL parameters field: enter one `name=value` per line and use `:name`, `@name`, or `$name` in the query. SQLite binds the values instead of interpolating them into SQL text. Unprefixed values bind as text; use `text:`, `int:`, `real:`, `bool:`, or `null:` to select a SQLite value type. Parameter values are saved with automation jobs, captures, and run history so an outcome can be reproduced.
3. Ingest a baseline query result. Update the query or database to the next period and ingest again using the same series label.
4. Review removed connections, confidence declines, changed event dates, additions, and confidence increases. Each changed connection appears once, with separate change labels for confidence, date, connection details, evidence references, and source notes. Filter the review by regression, entity properties, relationship properties, direction, evidence, source-note, or other changes, and search across IDs, endpoint names/types, relationship labels, evidence, and notes. Comparison rows show typed endpoints, direction, evidence references, and notes before and after the change, including for removed connections. The **Show** action jumps to a relationship if it remains on the current chart; removed or otherwise absent records are marked Removed or Not on chart. Use **Export comparison CSV/TSV** to share the complete selected baseline/current delta report with other analysis tools; the export includes before/after facts, both snapshot IDs and capture times, and the skipped-row count for each capture, regardless of the on-screen filter. Use **Export full series CSV/TSV** to hand off every capture in the selected database/series as one longitudinal file. Snapshot history can be searched by capture UUID, capture date, query text, SQL parameter values, result quality, and skipped-row count. Search filters the history list; if the selected comparison is outside the matches, the app identifies that selection while keeping its comparison visible. Switching saved series clears the search and comparison filters and selects the latest capture in the new series. The **Review saved series** picker shows the number of chart links and endpoint entities from each SQL source. **Highlight source on chart** clears the selected item and temporary path, group, and timeline filters, then fits the chart to and emphasizes that source’s links and entities while dimming unrelated chart records; the chart toolbar identifies the SQL series and database, and **View source records** returns to the Connections table with that source filter applied; choose **Review SQL snapshots** there to return to that source’s comparison history. The fields under **Ingest new snapshot** configure a separate new capture. The latest SQL snapshot also updates the chart for that database/series while preserving manually created and other-source links.

## Review snapshots

The saved-series review table can filter regressions, endpoint identity changes, direction changes, evidence changes, source-note changes, or other updates, and search both sides of a change by connection ID/key, endpoint, type, date, evidence, notes, or custom endpoint properties. Filtering only changes the on-screen rows; the comparison export always includes every change. The export retains a readable `change` label and adds a boolean column for each change type, so spreadsheet filters and downstream tools can select exact regression categories. Before/current endpoint attribute JSON is included in comparison CSV/TSV and structured run deltas; the SQL Review table shows those properties side by side. Endpoint ID changes have a dedicated `entity_identity_changed` kind and filter while remaining included in `connection_updated`/`connection_details_changed` for existing consumers; before/current ID columns show the reassignment. Direction changes similarly have their own `direction_changed` category while remaining included in `connection_details_changed`. The `connection_key` column exposes the same stable ID or natural key used to match each record across snapshots, including when the source query does not provide `connection_id`. Structured automation deltas use this same normalized key in each connection’s `key` field.

## Compare and export

In **SQL Review**, comparisons default to the immediately preceding capture in the selected series. Use **Compare with** to select any earlier capture in that same series for longer-range analysis; the selected baseline changes the on-screen delta table and CSV/TSV export together.

### CLI snapshot export

Export one immutable SQL capture without opening the desktop app. The output contains the captured connection fields plus snapshot provenance, query, typed parameter values, SQLite schema version, and skipped-row count on each record. An empty capture still exports its header row. A `.tsv` or `.tab` output path selects tab-separated output; other paths use CSV. Export is atomic and refuses to overwrite the case file or any saved SQLite source.

```sh
cargo run --offline -- --export-sql-snapshot harbor-light.case.json <snapshot-id> capture.csv --json
```

Get a snapshot UUID from `--list-runs --json` or the saved case's snapshot records. The JSON response reports output format, capture counts, source key and paths, series, query, and typed parameter values, so an empty capture still has its complete provenance in the response.

Export a complete saved series by selecting any capture in that database/series. The CSV/TSV combines rows from every capture in case order, while the JSON response retains metadata for zero-connection captures too:

```sh
cargo run --offline -- --export-sql-series harbor-light.case.json <snapshot-id> series-history.csv --json
```

The output is atomic and refuses to replace the case or any saved SQLite source. This is useful for loading longitudinal capture rows into spreadsheet or analysis tools without exporting each capture separately.

### CLI comparison export

Export a saved SQL comparison without opening the desktop app by passing the baseline and current snapshot UUIDs. The snapshots must be ordered in the same saved database/series. A `.tsv` or `.tab` output path selects tab-separated output; other paths use CSV. Export is atomic and refuses to overwrite a saved SQLite source database.

```sh
cargo run --offline -- --export-sql-comparison harbor-light.case.json <baseline-snapshot-id> <current-snapshot-id> comparison.csv --json
```

The JSON response reports the output format, snapshot IDs, and number of changed records. Get snapshot UUIDs from `--list-runs --json` or the case’s saved snapshot records.

## Exchange data

CSV/TSV and comparison reports are written to a temporary sibling file and atomically replace the destination only after the full export succeeds. Delimited imports detect comma, semicolon, tab, or pipe separators from the first record outside quoted values; `.tsv` and `.tab` paths always use tabs. Imports also strip a UTF-8 BOM from the first header before column detection, so files saved by spreadsheet tools still recognize standard fields and saved import designs.

### IBM i2 Analyst’s Notebook

Use CSV/TSV as the supported handoff: create or select an Analyst’s Notebook import specification and map the exported source/target, type, relationship, date, confidence, direction, evidence, notes, entity IDs, and property columns. For repeatable ingestion, use SQL result aliases such as `source_attribute:phone` and `connection_attribute:role`; those fields can be captured directly from SQLite or written to CSV/TSV and consumed by an import specification.

For a saved SQL capture, run `arch-notebook --export-sql-chart-csv case.json <snapshot-id> analyst-chart.csv --json` (use a `.tsv` destination for tab-separated output). This emits relationship rows in the chart import layout, expands endpoint and relationship properties into `source_attribute:<key>`, `target_attribute:<key>`, and `connection_attribute:<key>` columns, and stores stable SQL endpoint IDs as source-scoped external IDs. Repeated exports from the same SQL series can therefore be reconciled by the receiving import design. The command exports the selected immutable capture; it does not rerun the query or modify SQLite.

In the desktop app, select a capture in **SQL Review**, enter a destination in **CSV/TSV export path**, and choose **Export chart import CSV/TSV**. The same button works for historical captures; it exports the selected capture rather than always using the latest one.

IBM documents `.anx` as its standardized XML chart representation ([format guide](https://docs.i2group.com/anb/10.1.0/import_formats.html)). IBM distributes the ANX XSD and examples with the installed Analyst’s Notebook application ([schema location](https://www.ibm.com/support/pages/node/546669)). Arch Notebook does not currently read or write ANX, and GraphML is not a substitute for ANX. Add ANX support against an IBM-provided schema and fixture so chart validation and round-trip fidelity can be checked.

### GraphML and KML

**Import GraphML** stages nodes and relationships through the validated import path; it combines multiple top-level graphs into one case, flattens nested subgraphs while retaining each child entity’s `arch_notebook_parent_id` attribute, and imports recognized IDs, aliases, entity attributes, per-graph direction defaults, evidence, notes, and chart styling. GraphML key-level `<default>` values are applied to nodes and edges that omit that property; explicit record data takes precedence. When GraphML omits `ingest_source`, the canonical input path identifies its source; edges without a connection ID or GraphML edge ID receive a stable ID from their source, endpoints, relationship, direction, and occurrence among identical parallel edges. This keeps ordinary edge identities steady when unrelated edges are reordered in the file. **Export GraphML** assigns edge IDs from source and connection identity and supplies a generated connection ID for local links that lack one, so exported relationships remain identifiable after reordering. It works with graph tools such as Gephi and Cytoscape and includes endpoint IDs, direction, labels, evidence, notes, confidence, source provenance, entity attributes, and chart styling. **Export KML** writes KML 2.2 points for entities with valid `latitude`/`lat` and `longitude`/`lon` (also `lng`, `long`, and `geo:lat`/`geo:lon`) attributes and line strings for links where both endpoints have coordinates; it carries entity attributes, external IDs, and link metadata. **Import KML** accepts KML 2.2 `Point` and `LineString` placemarks through the validated import path. Point coordinates become latitude/longitude attributes. `ExtendedData` can carry entity or connection attributes, external IDs, direction, evidence, and endpoint IDs. For external files without endpoint IDs, line endpoints are matched to unique point coordinates; unmatched or ambiguous coordinates fail with an actionable error. Other geometry types are reported as unsupported. Add coordinates through CSV/TSV entity attributes or SQL aliases such as `source_attribute:latitude` and `source_attribute:longitude`, then use the Chart toolbar or Data path controls, or run `arch-notebook --export-kml case.json chart.kml --json`. The JSON result reports mapped entities, exported relationships, and links lacking coordinates. Graph-level `source_catalog_json` maps each ingest-source key to its database and series, latest snapshot and SQLite schema version, exported link and distinct entity counts, and capture IDs with timestamps, SQLite schema versions, connection counts, and skipped-row counts. Valid source-catalog entries survive GraphML import into the case file; re-export unions distinct capture IDs and prefers the newest capture metadata when a source key appears more than once. GraphML/KML are cross-tool interchange formats; CSV/TSV carries the full Arch Notebook case fields for round-trips. For scripts, import into an existing case or export without opening the desktop app:

```sh
arch-notebook --import-graphml case.json network.graphml --json
arch-notebook --import-kml case.json locations.kml --json
arch-notebook --export-graphml case.json chart.graphml --json
arch-notebook --export-sql-chart-csv case.json <snapshot-id> analyst-chart.csv --json
```

GraphML and KML imports use the same validation and entity reconciliation as desktop import, lock the case against concurrent automation writes, and atomically save only after the full import succeeds. Their JSON response reports total entities and relationships in the updated case. GraphML export’s schema-versioned JSON response reports the output path and exported entity and relationship counts.

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
