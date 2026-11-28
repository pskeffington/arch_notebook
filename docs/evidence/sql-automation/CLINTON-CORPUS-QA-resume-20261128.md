# Clinton primary corpus QA rerun — 2026-11-28

**Task / owner:** Resume the Rust Analyst's Notebook inspired build and revalidate the primary Clinton SQL batch; orchestrator.

**Repository / branch / baseline:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions` / `fbd293cf2612a480c6de59df38095b20ff1466bf`.

**Installed executable:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `8d1c1087cdbe126785ba6ca57721a3f2f704ab08e537d9dcd0a0b11d4b739c04`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Primary Clinton corpus SQL harness | PASS | `python3 scripts/test-clinton-corpus-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/CLINTON-CORPUS-QA-resume-20261128.json` exited 0. |
| Query-only preflight | PASS | One saved job passed; zero failed. |
| Saved-run replay | PASS | 6,173 relationship rows; zero skipped; added, changed, and removed deltas all zero. |
| Default-redacted topology packet | PASS | 2,813 entities and 6,173 relationship records; all 2,813 entity centers rendered; zero labels outside the SVG viewport; 643 communities and 291 proxy candidates. Names, relationship labels, IDs, and evidence references were excluded. Output directory mode `0700`; report and SVG modes `0600`. |
| Input immutability | PASS | Harness verified the PDF, case, and SQLite source hashes were unchanged. |
| Aggregate report integrity | PASS | [`CLINTON-CORPUS-QA-resume-20261128.json`](artifacts/CLINTON-CORPUS-QA-resume-20261128.json), SHA-256 `414723aa877711b4a0265165640ef168c79a6e9f6c6caf1a8ba3b5ff2415fb6d`; aggregate counts and hashes only. |
| PDF extraction completeness, Qwen analysis, human content review | NOT ESTABLISHED / NOT RUN | This rerun validates saved SQL and default-redacted topology output; it does not interpret or verify email contents. |
| IBM ANX compatibility | NOT TESTED | No IBM schema or application was used. |

No source rows, email text, names, or raw corpus content were added to Git.
