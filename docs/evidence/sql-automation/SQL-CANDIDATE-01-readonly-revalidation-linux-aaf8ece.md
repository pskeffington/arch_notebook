# SQL-CANDIDATE-01 local candidate read-only revalidation

- **Task / owner:** Revalidate the retained TRANSCRIBE candidate with the current Arch Notebook CLI / Arch Notebook maintainer
- **Repository source commit:** `aaf8ece8f57d17890c2797a68d624b4927c920a2`
- **Branch:** `codex/ui-00-chart-baseline`
- **Binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `178554daa2fa799da921c51551e8d46796296a1ca5681237819f8002d27e0058`.
- **Local inputs:** `/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json` and `clinton-email-analysis.sqlite`. Neither file nor any row-level values are included in Git.
- **Result:** **PASS** for saved-case read compatibility, saved-query preflight, and snapshot comparison export. This does not establish source completeness or semantic validity.

## Results

The current Rust CLI read the saved case and listed two `PASS` runs and two `COMPLETE` batches. Aggregate saved-state counts were 2,813 entities, 6,173 chart relationships, and two SQL snapshots, each retaining 6,173 query rows with zero skipped rows.

The saved SQL job is disabled. `--validate-enabled-jobs` therefore returned exit 1 with `case contains no enabled automation jobs`; it performed no writes. The saved job was then validated directly in query-only mode with `--validate-job <case> <saved-job-name> --query-only --json`; it returned exit 0, `ok=true`, `passed=1`, and `saved=false`.

The CLI exported the saved snapshot comparison to a disposable CSV under `/tmp`. Export returned exit 0 and contained three `DATE CHANGED` records, with no other change categories. The script summarized counts only; it did not print names, addresses, subjects, message bodies, or row-level records.

The latest saved run was then replay-validated with `--validate-run <case> <run-id> --json`. It returned exit 0, `status=PASS`, `saved=false`, 6,173 captured relationships, zero skipped rows, and zero additions, removals, or changes against the saved baseline. The CLI reported a 332 ms run duration for this host and query; this is one observation, not a performance target.

SHA-256 values for the case and SQLite file matched before and after query validation, history listing, comparison export, and saved-run replay. The export remained under `/tmp`; no source or case write was requested.

## Commands and evidence

| Check | Result |
| --- | --- |
| `arch-notebook --validate-job <case> <saved-job-name> --query-only --json` | PASS; exit 0, one job passed, `saved=false` |
| `arch-notebook --list-runs <case> --json` | PASS; exit 0, two runs with status `PASS` |
| `arch-notebook --list-batches <case> --json` | PASS; exit 0, two batches with status `COMPLETE` |
| `arch-notebook --export-sql-comparison <case> <baseline-id> <current-id> /tmp/.../saved-comparison.csv --json` | PASS; exit 0, three rows, all `DATE CHANGED` |
| `arch-notebook --validate-run <case> <latest-run-id> --json` | PASS; exit 0, 6,173 rows, zero skipped, zero deltas, `saved=false` |
| Case and SQLite SHA-256 before/after | PASS; both unchanged |

## Limits and next action

The earlier enabled-job preflight cannot apply while the saved job is disabled; single-job query-only validation is the applicable check. The saved candidate remains review-required. Its extraction coverage, human verification, released TRANSCRIBE approval, and Qwen analysis remain unverified; no claim of 7,000 validated emails is made.
