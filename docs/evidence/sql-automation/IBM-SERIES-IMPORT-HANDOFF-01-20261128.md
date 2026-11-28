# IBM Series Import handoff — 2026-11-28

## Scope and source

Task: document an operator workflow from scheduled SQL captures in Arch Notebook to IBM i2 Analyst's Notebook.

Repository: `/tmp/arch-notebook-remote-candidate`

Source commit: `2427f537eb637c19ef1ff0023cb4a0232a52b1d2`
Changed paths: `docs/automation.md`, `docs/build-acceptance.md`, this evidence report.

## Evidence

- Source inspection of `src/main.rs::sql_chart_csv_path_for_job` confirms each exported file is named `<sanitized-job-name>-<stable-job-uuid>.csv` (or `job-<uuid>.csv` when the name has no ASCII alphanumeric characters).
- Source inspection of `src/main.rs::write_sql_snapshot_chart_csv` confirms the SQL capture export contains endpoint names and external IDs, entity types and representations, relationship, date, confidence, direction, source reference/notes, ingest source, and present source/target/relationship attributes.
- `docs/automation.md` documents that scheduled export files refresh atomically from successful captures and gives a Windows Series Import invocation that uses the explicit current CSV path (`-i`) and creates a new chart (`-c`).
- IBM documents text/CSV import specifications and their field mapping in the [import specification wizard](https://docs.i2group.com/anb/10.1.1/importing_a_file_with_a_new_specification.html). It documents Series Import's `-b`, `-i`, and `-c` options in its [batch import examples](https://docs.i2group.com/anb/10.0.2/series_batch_import.html). IBM distinguishes classic `.ximp` from schema-aligned `.oimp` and says they are not convertible in its [specification reference](https://docs.i2group.com/anb/10.1.1/import_specifications.html).

## Results

| Gate | Result | Basis |
| --- | --- | --- |
| Scheduled CSV export contract and stable filename | PASS · source inspection | Functions named above. |
| IBM official import and automation syntax | PASS · documentation review | IBM documentation links above. |
| Operator guidance and caveats | PASS | [Automation guide](../../automation.md#handoff-to-ibm-i2-analysts-notebook). |
| `git diff --check` | PASS | Exit code 0. |
| Native IBM import, mapping correctness, and import report | NOT TESTED | IBM Analyst's Notebook / i2 Analyze was not available in this environment. |

## Limits and next action

This is not a native IBM compatibility pass. The analyst must create the IBM import specification in the IBM application against the export header, verify the field mappings, run Series Import in the target deployment, and retain its import report. Use `.ximp` in classic mode and schema-aligned `.oimp` when connected to i2 Analyze; do not treat the Arch Notebook import-design JSON as an IBM specification. Changes to CSV headers or the target schema require reviewing the saved mapping.
