# CLINTON-CORPUS-QA-05 · desktop XML/XSLT release SQL revalidation — 2026-11-28

**Task / owner:** Re-run the primary real-corpus SQL acceptance on the latest installed Rust release / orchestrator.

**Product repository / branch:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions`.
**Product code commit:** `9293242198483edb977b7e0d49e778cfa149ad85` (desktop XML/XSLT import workflow).
**Installed binary:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `4f37a78a91230b8c1aeafc16d0d3aedc18638dd83bb56eaf2172d504cf24c08d`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Primary Clinton QA harness | PASS | `python3 scripts/test-clinton-corpus-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/CLINTON-CORPUS-QA-05-desktop-xslt-current.json` exited 0. |
| Query-only saved-job preflight | PASS | 1 job passed, 0 failed; 6,173 connection rows; 0 skipped; no case save. |
| Saved-run replay | PASS | Latest saved run `PASS`; 6,173 rows, 0 skipped, and added/removed/changed-record deltas all 0. |
| Default-redacted topology packet | PASS | 2,813 entities and 6,173 relationship records; all entity centers rendered; 704 y-rows, largest row 4; zero labels outside the SVG viewport; 643 communities and 291 proxy candidates. Names, relationship labels, IDs, and evidence references were excluded. Packet directory mode `0700`; report/SVG modes `0600`. |
| Input immutability | PASS | Harness verified that the source PDF (`00a9b9e879ffb1835cb216b69f2488e776726f6c239acccae5025b860d4805e9`), case (`6143c16d3f66964991f80184d6e2a4d165553bf9443ac3cb6bfbeea930fa255b`), and SQLite database (`fe9dcdf26f3810a0df3905ddf217cfd45704fd412f1509e1e379d2bb1483e071`) hashes were unchanged. |
| Aggregate report integrity | PASS | `/tmp/CLINTON-CORPUS-QA-05-desktop-xslt-current.json`, SHA-256 `cd18d79a1d0a661208132753ad566f691c3ee5b35234f443425fdd85a554549c`; contains aggregate counts only. |
| PDF completeness, Qwen analysis, human review | NOT ESTABLISHED / NOT RUN | This harness replays saved SQL and exports default-redacted graph aggregates; it does not interpret or validate email content. |
| IBM ANX compatibility | NOT TESTED | No IBM schema or application was used. |

The installed release passes its primary Clinton SQL regression. This does not validate PDF transcription, email completeness, Qwen interpretation, or human conclusions. No source rows or raw email content were added to Git.
