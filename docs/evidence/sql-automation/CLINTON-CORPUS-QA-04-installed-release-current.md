# CLINTON-CORPUS-QA-04 · current installed release — 2026-11-28

**Task / owner:** Revalidate the primary real-corpus SQL workflow on the current per-user Arch Notebook release / orchestrator.

**Repository / branch:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions`.
**Product code commit:** `5b65bfe38691e501997ca3fd721492717282f408` (current Quick open product build; following commits changed documentation only).
**Installed binary:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `8381bd57bdf84017377d92ef62916bde737e05ebe15dbca4b98317ec4b62ed27`; `--version --json` reports `0.1.0`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Primary Clinton QA harness | PASS | `python3 scripts/test-clinton-corpus-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/CLINTON-CORPUS-QA-04-installed-release-current.json` exited 0. |
| Query-only saved-job preflight | PASS | 1 job passed, 0 failed; 6,173 connection rows; 0 skipped; `saved=false`. |
| Saved-run replay validation | PASS | Latest saved run `PASS`; 6,173 rows, 0 skipped, and delta counts added/removed/changed all 0. |
| Redacted topology packet | PASS | 2,813 entities and 6,173 relationship records; all 2,813 entity centers rendered; 704 y-rows, largest row 4, 0 labels outside the SVG viewport; 643 structural communities and 291 proxy candidates. Names, relationship labels, IDs, and evidence references were not disclosed. Output directory mode `0700`; report and SVG `0600`. |
| Source immutability | PASS | PDF SHA-256 `00a9b9e879ffb1835cb216b69f2488e776726f6c239acccae5025b860d4805e9`; saved case `6143c16d3f66964991f80184d6e2a4d165553bf9443ac3cb6bfbeea930fa255b`; SQLite `fe9dcdf26f3810a0df3905ddf217cfd45704fd412f1509e1e379d2bb1483e071`. Harness confirmed all three hashes unchanged after execution. |
| Aggregate report integrity | PASS | `/tmp/CLINTON-CORPUS-QA-04-installed-release-current.json`, SHA-256 `5c4c69e0e9fee37deb39ce5f0dcc5438b25eb90c2171c42490f64a08e811321d`. The report contains aggregate counts only. |
| PDF extraction completeness, Qwen analysis, human content review | NOT ESTABLISHED / NOT RUN | This acceptance revalidates saved SQL capture/replay and local graph output; it does not process the PDF text, validate email meaning, or assess source completeness. |

The full report and command outputs remain in `/tmp`; no row-level source values were committed.
