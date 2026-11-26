# SQL-CANDIDATE-02 saved snapshot CSV round trip

- **Task / owner:** Revalidate saved SQL snapshot export/import identity on the retained local candidate / Arch Notebook maintainer
- **Repository source commit:** `f6c5994ad4890eaac9a8deda7d8de5b7052cd658`
- **Branch:** `codex/ui-00-chart-baseline`
- **Binary:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `178554daa2fa799da921c51551e8d46796296a1ca5681237819f8002d27e0058`.
- **Inputs:** The local review-required case and SQLite database in `/home/sysadmin/Research.Databases/TRANSCRIBE/`; both stayed local and unchanged.
- **Temporary outputs:** `/tmp/arch-notebook-clinton-csv-exchange-01/`.
- **Result:** **PASS** for Arch Notebook's own SQL-to-chart-CSV and saved-design import path.

## Acceptance results

The latest saved SQL snapshot exported 6,173 chart-import CSV rows. Arch Notebook generated and imported a CSV design into a fresh disposable case, then imported all rows with 6,173 created, 0 updated, and 0 skipped. The result contained 2,813 entities and 6,173 relationship records. All connection IDs were distinct. For every row, endpoint names, relationship labels, source references, and notes matched the imported graph.

The original candidate case and SQLite source had matching SHA-256 values before and after the workflow. Row-level names, identifiers, and evidence values were compared in process but were not printed or committed.

## Commands and results

| Operation | Result |
| --- | --- |
| `arch-notebook --export-sql-chart-csv <case> <latest-snapshot-id> /tmp/.../candidate-snapshot.csv --json` | PASS; exit 0, 6,173 rows |
| `arch-notebook --create-import-design /tmp/.../candidate-snapshot.csv /tmp/.../candidate-import.design.json --name <verification-name> --json` | PASS; exit 0 |
| `arch-notebook --import-import-design /tmp/.../empty.case.json /tmp/.../candidate-import.design.json --name <verification-name> --json` | PASS; exit 0 |
| `arch-notebook --import-delimited /tmp/.../empty.case.json /tmp/.../candidate-snapshot.csv --design <verification-name> --json` | PASS; exit 0; 6,173 created, 0 updated, 0 skipped |
| Original case and source SQLite SHA-256 before/after | PASS; both unchanged |

This validates Arch Notebook's internal CSV/import-design handoff on the local candidate. It does not validate IBM Analyst's Notebook's receiving import specification or UI. PDF extraction completeness, human review, release approval, and Qwen analysis remain unverified.
