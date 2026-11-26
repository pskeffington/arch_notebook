# SQL-CANDIDATE-03 current-head read-only verification

- **Task / owner:** Revalidate the retained TRANSCRIBE candidate with the current Arch Notebook release / implementation session
- **Repository source commit:** `561d52f86f4ffb40b0bb69d2de78c8f103bb9be9` on `codex/ui-00-chart-baseline`
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `e42240f90edfa058d78304ab549e45f8237d426dcba7e25ae8995bb6c7124e1a`
- **Inputs:** local candidate case and SQLite source under `/home/sysadmin/Research.Databases/TRANSCRIBE/`; row-level values and files remain outside Git.
- **Result:** **PASS** for current-source saved-query validation and saved-run revalidation. No completeness or semantic-validity claim is made.

## Results

The current release listed one saved SQL job (disabled) and two saved runs (both `PASS`). Query-only validation of the saved job returned exit 0, `ok=true`, one passed, zero failed, `saved=false`. It reported 6,173 relationships and zero skipped rows.

The latest saved run was revalidated against the current SQLite source with `--validate-run`. It returned exit 0, `ok=true`, one passed, zero failed, and `saved=false`. The outcome matched all 6,173 relationship records, skipped zero rows, and reported no added records, relationship-property changes, or entity-property changes. Its comparison listed zero record changes and no SQLite schema fingerprint or schema-version drift.

The case SHA-256 was `6143c16d3f66964991f80184d6e2a4d165553bf9443ac3cb6bfbeea930fa255b` before and after. The SQLite source SHA-256 was `fe9dcdf26f3810a0df3905ddf217cfd45704fd412f1509e1e379d2bb1483e071` before and after. The verification output was summarized in-process; names, email addresses, subjects, message bodies, SQL text, and row-level values were not printed or committed.

## Checks and limits

| Check | Result |
| --- | --- |
| `arch-notebook --list-jobs <case> --json` | **PASS**, exit 0; one disabled job |
| `arch-notebook --list-runs <case> --json --limit 10` | **PASS**, exit 0; two saved PASS runs |
| `arch-notebook --validate-job <case> <job-id> --query-only --json` | **PASS**, exit 0; 6,173 relationships, zero skipped, `saved=false` |
| `arch-notebook --validate-run <case> <latest-run-id> --json` | **PASS**, exit 0; 6,173 relationships, zero skipped, zero record/property deltas, `saved=false` |
| Case and SQLite source SHA-256 before/after | **PASS**, unchanged |

The saved job is disabled, so enabled-job preflight and scheduler dispatch were not exercised. This check does not verify extraction coverage, message-level accuracy, human review, release approval, Qwen analysis, IBM Analyst's Notebook interoperability, native Automation UI, or live scheduler delivery. It verifies only that the saved query currently reproduces its retained capture against the unchanged local SQLite source.
