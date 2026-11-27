# Codex execution handoff: real corpus intake to archive SMB

## Mission
Execute on the operator's network-enabled host with the existing archive SMB share mounted. Acquire actual public records in bulk, not synthetic data or merely URL lists. Prioritize Massachusetts energy money flows and project decisions, then political finance, sovereign debt and confirmed corruption controls. Preserve the existing offline Rust application and fictional examples; operational evidence lives outside Git.

## Preflight (mandatory)
- Inspect the repository, existing archive layout, and SQL ingestion contract in docs/sql-ingestion.md.
- Resolve the existing archive SMB mount using host mount information (Linux findmnt -t cifs,smb3; macOS mount / smbutil statshares -a). Require explicit ARCHIVE_SMB_ROOT pointing to the intended existing share if ambiguous; do not guess a server, create a share, mount with guessed credentials, or use a local fallback. Verify actual SMB mount, write permission via a temporary probe, available bytes, and atomic rename behavior. Do not store credentials.
- Use ARCHIVE_SMB_ROOT/arch_notebook/corpus unless an existing canonical corpus directory is found and confirmed. Print resolved path and capacity before writing. Never delete existing archive data. Default first run budget 25 GiB, retain 20 GiB free or 10% of capacity whichever is greater; make configurable. Stop safely when capacity cannot be established.
- Use a single writer lock; maintain a durable JSONL run journal; use temporary files, validate content and checksums, then publish via same-directory rename. Resume only with validated partial download metadata and supported HTTP Range. Retry boundedly with backoff, respect source terms/rate limits and Retry-After. Never bypass access controls.

## Acquisition order (2015 onward where available; retain earlier baselines where relevant)
| Priority | Source | Scope | Discovery URL |
|---|---|---|---|
| P0 | MA CTHRU | full expenditure ledger, vendor and agency, fiscal years | https://cthruspending.mass.gov/ |
| P0 | MA OCPF | full contribution and expenditure bulk archives, all committees | https://www.ocpf.us/Data/Downloads |
| P0 | EIA 860 | annual generator, owner, plant and planned/retired units | https://www.eia.gov/electricity/data/eia860/ |
| P0 | EIA 923 | generation, fuel purchases and costs | https://www.eia.gov/electricity/data/eia923/ |
| P0 | MA DPU | orders, petitions, PPAs, termination schedules, rate cases | https://www.mass.gov/orgs/department-of-public-utilities |
| P0 | MA DOER | procurement rounds, selections, contracts and awards | https://www.mass.gov/orgs/massachusetts-department-of-energy-resources |
| P1 | USAspending | MA awards, transactions, grants and contracts; agency and recipient geography | https://www.usaspending.gov/download_center/award_data_archive |
| P1 | FEC | bulk committee, candidate, contributions and expenditures; include AIPAC and UDP committee IDs C00797670/C00799031 | https://www.fec.gov/data/browse-data/?tab=bulk-data |
| P1 | FERC | Form 1 utility financials and capital costs | https://www.ferc.gov/industries-data/electric/general-information/electric-industry-forms/form-1-electric-utility |
| P1 | MA COMMBUYS | procurement records and award notices | https://www.commbuys.com/ |
| P1 | MA lobbying | client, lobbyist and disclosure records | https://www.sec.state.ma.us/LobbyistPublicSearch/ |
| P1 | BOEM / ISO NE | leases, generation queues, market and grid records | https://www.boem.gov/renewable-energy ; https://www.iso-ne.com/ |
| P2 | SEC / PRIM / MA Treasury / municipal retirement systems | corporate finance, public debt holdings, Israel sovereign bond CUSIP/ISIN, direct vs pooled exposure | https://www.sec.gov/edgar/search/ ; https://www.mapension.com/ |
| P2 | DOJ / SEC enforcement / FARA | FirstEnergy, 1MDB and other confirmed corruption controls; foreign lobbying disclosures | https://www.justice.gov/ ; https://www.sec.gov/ ; https://www.fara.gov/ |

Discovery URLs are not bulk payload URLs. Resolve actual official download links, APIs, pagination, year partitions and schemas. Verify each response is the expected data type, not an HTML error/login page. Download full available partitions within budget; do not prefilter to named individuals or firms. Prioritize institutional and transaction records, exclude unnecessary personal addresses and contact details from analytical exports. For sources with no bulk export, record exact blocker and public records request needs; continue other sources.

## Energy case scope
Build decision timeline from 2015, compare pre-2023 to Healey administration (from Jan 2023). Include Commonwealth Wind, SouthCoast Wind, New England Wind 1, Vineyard Wind 2, Everett LNG, gas infrastructure, hydro imports, solar, storage, Mass Save, EV and grid modernization. Capture proposed/selected/approved/terminated/withdrawn/operating states separately; identify actual decision maker and date from orders. Trace developers, owners, utilities, vendors, lenders, grant agencies, contract amounts, obligations, disbursements, refunds and ratepayer credits. Reconcile existing termination_obligations.csv against original orders before promotion; quarantine discrepancies. No claim of corruption from donation, lobbying, proximity, or timing alone. Use confirmed enforcement controls and lawful comparison networks. Distinguish Israel sovereign debt from earmarked military financing; do not assume holdings or use of proceeds.

## Storage and provenance
raw/<source>/<year>/<sha256>/<original_filename>; manifests/<run_id>.jsonl; normalized/<source>/<year>; sqlite/<capture_id>.sqlite; quarantine/<run_id>; logs/<run_id>. Every manifest record: source ID, discovery and resolved URL, retrieved UTC, HTTP status, ETag/Last-Modified, bytes, SHA256, media type, source period, license/access terms where known, relative path, state and error. Immutable original bytes; never overwrite evidence; deduplicate by hash, retain distinct source observations. Keep all large data and operational logs on SMB, no Git LFS or GitHub artifacts without owner approval. Do not commit personal records, credentials or raw evidence.

## Implementation and actual execution
1. Implement a host-side CLI under scripts/corpus/ with preflight, discover, pull, verify, normalize, status; checked-in machine-readable source registry and tests. Use streaming downloads with bounded concurrency and configurable budgets; dry-run only for preflight, then execute real pull. No new network services or runtime dependency in Rust.
2. Start EIA and OCPF bulk archives and CTHRU partitions, then DPU/DOER records; continue through priorities while budget permits. Record actual counts, bytes, hashes and errors. Validate ZIP paths and expansion limits before extraction; no executable content.
3. Normalize in bounded batches using SQLite or Parquet; build SQLite locally on scratch, integrity_check, close/checkpoint, copy verified immutable database to SMB. Never use writable live SQLite/WAL over SMB. Stage read-only local copies for notebook captures if needed. Use stable source-native IDs and typed monetary fields, currency, amount_kind, event date, source locator, evidence status. Keep unknown distinct from zero; no invented joins.
4. Expose read-only relationship view returning source,target and stable connection IDs per existing mapping; include provenance, date, relationship type and amount. Import bounded samples into Arch Notebook; use SQL filters/aggregation for million-row sets, not unbounded graph rendering. Measure capture latency, memory, dropped rows, stable ID collisions and snapshot differences.
5. Commit code, tests, redacted aggregate run summary and schema to branch. Report actual acquired files/bytes/rows by source, archive path (no credentials), failures, verification results, and next partitions. Do not claim acquisition from a manifest alone. If SMB is absent, stop before downloading and report exact mount requirement; if network blocked, report failure, do not generate synthetic substitutes.

Acceptance: real official files on confirmed SMB, checksum verified, reproducible resume/idempotence test, SQLite integrity and read-only chart capture, documented gaps. Keep existing native acceptance and release restrictions unchanged.
