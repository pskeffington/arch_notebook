# Arch Notebook

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

## Other notebook workflows

- Build and edit entity/link charts, inspect attributes and source references, arrange by connectivity, and review chronological timelines.
- Search and sort entity/relationship records, inspect most-connected entities, and analyze directed shortest paths.
- Review likely duplicate entities before merging. Merge rewires links and preserves aliases, attributes, notes, and the original record in entity history.
- Import/export relationship CSV and save/open local JSON case files. SQL snapshot history is retained with the case.

All bundled case examples are fictional. Arch Notebook does not connect to a service or transmit case data.

## Research basis

IBM i2 Analyst's Notebook documentation describes charts built from typed entities and links, attributes on chart items, confidence and direction on links, highly connected item analysis, and timeline charts. Those concepts inform the chart and investigation workflow:

- [IBM i2 Analyst's Notebook User Help (9.2.4)](https://www.ibm.com/docs/en/SSJSV9_9.2.4/pdf/analysts_notebook_pdf.pdf)
- [IBM Analyst's Notebook Essentials](https://www.ibm.com/training/course/ibm-i2-analysts-notebook-essentials-9I02G)
- [SQLite read-only open flags](https://www.sqlite.org/c3ref/open.html)
- [rusqlite Connection API](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html)
