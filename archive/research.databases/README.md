# Research database archive

This directory holds small, fictional fixtures used to demonstrate SQLite ingestion and connection regression review. Do not place live, personal, customer, or otherwise sensitive data here.

## Files

- `sample_connections.sql` is the editable source of the baseline/current example. It intentionally drops and recreates only `connection_observations` in the destination database.
- `sample_connections.sqlite` is the ready-to-open SQLite copy of that example. The app opens source databases read-only; regenerate this fixture only as an explicit archive maintenance change.
- `relational_transfers.sql` creates a synthetic `accounts`/`transfers` schema for the relational SQL ingestion walkthrough. It intentionally replaces only those two tables in its destination database; build it in `/tmp` as shown in the guide.
- `sample_relationships.csv` is the matching tabular import/export example.

To rebuild the SQLite fixture from the repository root, write to a temporary file first, inspect it, then replace the tracked fixture as part of a reviewed change:

```sh
sqlite3 /tmp/sample_connections.sqlite < archive/research.databases/sample_connections.sql
sqlite3 -readonly /tmp/sample_connections.sqlite 'PRAGMA integrity_check; SELECT snapshot_id, COUNT(*) FROM connection_observations GROUP BY snapshot_id ORDER BY snapshot_id;'
```

Expected verification: `integrity_check` returns `ok`, with four baseline and four current rows. Never run the seed SQL against an operational research database: its `DROP TABLE` is intended only for a disposable fixture.

The app itself never modifies a configured SQLite source. Keep larger source databases outside the repository and retain only synthetic, redistributable examples here.
