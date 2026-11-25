# Chart acceptance fixtures

All records in this directory and produced by `scripts/generate-chart-fixtures.py` are fictional and generated deterministically. The script uses only Python's standard library and writes case files to the requested output path. Small checked-in fixtures are for correctness; larger performance workloads belong under `/tmp` and are not committed.

## Workload matrix

| Family | Entities | Relationships | Shape |
| --- | ---: | ---: | --- |
| sparse | 100 / 1,000 / 10,000 | 3N | modular low-degree pattern |
| hub | 100 / 1,000 / 10,000 | 3N | concentrated high-degree hub plus ring-like tail |
| disconnected | 100 / 1,000 / 10,000 | 2N | eight weak components and one isolated entity |
| dense | 100 / 1,000 / 10,000 | min(10N, 100,000) | local clusters capped at 100, edge cap 100,000 |

Run with, for example:

```sh
python3 scripts/generate-chart-fixtures.py --family correctness --output fixtures/chart/correctness.case.json
python3 scripts/generate-chart-fixtures.py --family minimal --minimal-count 0 --output /tmp/chart-empty.case.json
python3 scripts/generate-chart-fixtures.py --family sparse --nodes 10000 --output /tmp/chart-sparse-10000.case.json
```

Each invocation prints a JSON manifest summary with SHA-256, entity and relationship counts, directed/undirected counts, weak components, parallel records and self-loops. IDs, placement, and relationships are deterministic. Invalid workload sizes are rejected. No random or real-world data is used.

The correctness case covers coincident positions with separate IDs, a pinned entity, a long off-canvas entity label, a viewport-edge chart annotation, parallel records, a shared nonempty connection ID across two sources, reversed directed relationships, a self-loop, anonymous duplicate records, long relationship labels, and disconnected entities. Minimal cases cover zero, one, and two entities.
