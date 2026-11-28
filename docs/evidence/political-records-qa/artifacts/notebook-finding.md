# Analyst finding packet

Chart: Redacted

> Simplified SVG overview of all persisted entities and relationship records; standalone chart labels are omitted. This export does not apply unsaved selection, search, or timeline filters. Stored chart positions are for layout only; visual distance is not a measure.

> Privacy: topology and aggregate counts may still identify people even when labels are pseudonymous. Review locally before sharing.

## Export options

- Names: false
- Relationship labels: false
- IDs: false
- Evidence references: false

## Analysis method card

ARCH NOTEBOOK · ANALYSIS METHOD CARD
Scope: active chart only; includes all chart entities and every relationship record.
Entities: 10 (10 isolates under the non-self, known-endpoint connectivity rule).
Relationship records: 0 total; 0 directed; 0 undirected.
Endpoint pairs: 0 unique unordered pairs with both endpoints present (direction ignored; self-pairs included); 0 additional complete-endpoint records on repeated pairs; 0 records have at least one endpoint missing from the chart.
Dates: 0 recognized by the chart timeline parser; 0 blank; 0 not recognized.
Components and articulation points: direction, relationship type, date, and confidence are ignored; self-links and records with missing endpoints are ignored; parallel records collapse to one neighboring entity for topology. Component analysis includes singleton entities, though the sidebar lists only groups with more than one entity.
Detected communities: deterministic, single-level greedy modularity local moves at resolution 1 with at most 50 sweeps and UUID-ordered traversal; undirected unweighted unique pairs; direction, relationship type, date, and confidence ignored; self-links and missing endpoints ignored; isolates remain singleton. This is an exploratory structural partition, not verified real-world group membership.
Geodesic layers: direction, relationship type, date, and confidence are ignored; each traversable neighboring pair is one unweighted hop; self-links, missing endpoints, and parallel records do not add distance.
Shortest path: direction is ignored; directed records traversable either way; date order is any date order; active timeline range is not applied; relationship type and confidence are ignored; every eligible relationship record with both endpoints present costs one hop; records with a missing endpoint are ignored. When date order or a timeline range is applied, records without a timeline-recognized date are excluded.

## Detected structural communities and recorded interactions

Detected structural communities are exploratory partitions, not asserted real-world groups or affiliations. This summary uses all persisted entities and relationships in the exported chart; unsaved search, selection, and timeline filters are not applied. The partition treats the graph as undirected and unweighted, collapses repeated/reversed endpoint pairs, ignores relationship type and date, excludes self-links and missing endpoints, and keeps isolates as singleton communities. The dependency-free single-level greedy modularity local-move heuristic uses resolution γ = 1.0, UUID-ordered traversal, at most 50 sweeps, and splits disconnected fragments. Charts with missing relationship endpoints are refused by packet export.

### Community membership

- **Community 1** · 1 entity: Entity 001
- **Community 2** · 1 entity: Entity 002
- **Community 3** · 1 entity: Entity 003
- **Community 4** · 1 entity: Entity 004
- **Community 5** · 1 entity: Entity 005
- **Community 6** · 1 entity: Entity 006
- **Community 7** · 1 entity: Entity 007
- **Community 8** · 1 entity: Entity 008
- **Community 9** · 1 entity: Entity 009
- **Community 10** · 1 entity: Entity 010

### Cross-community relationship records

Counts below retain repeated records. Distinct endpoint pairs collapse reversed/parallel records; direction counts describe records, not unique pairs.

| Community pair | Relationship records | Distinct endpoint pairs | First → second | Second → first | Undirected |
| --- | ---: | ---: | ---: | ---: | ---: |
| None | 0 | 0 | 0 | 0 | 0 |

### Cross-community proxies

Ranks entities that directly contact at least two other detected communities by distinct other communities, distinct neighboring entities, then all incident relationship records (including records within their own community). This is a structural contact indicator, not a claim of intent or influence.

| Rank | Entity | Distinct other communities | Distinct neighboring entities | All incident relationship records |
| ---: | --- | ---: | ---: | ---: |
| — | None | 0 | 0 | 0 |


## Analyst-authored fields

### Question


### Findings (software generated no findings)


### Evidence selected/references


### Limitations and alternative explanations


### Audience and privacy review


### Analyst / date / version


