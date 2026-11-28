# Clinton corpus QA — installed release recheck

- **Task / owner:** Revalidate read-only SQL automation and redacted topology export on the retained Clinton corpus; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source:** `7811d5256b50bc319c7c78b2a5e6e68a76ef41bc`.
- **Tested installed executable SHA-256:** `979aeb9aef259dbbc410088f96a5241da659f07998e7d8e011ca71b924042976`.
- **Data scope:** Existing TRANSCRIBE SQLite/case snapshot and the associated Clinton release PDF. This run did not process the separate ten-topic PDF QA bundle.

## Command

```sh
python3 scripts/test-clinton-corpus-acceptance.py \
  --binary /home/sysadmin/.local/bin/arch-notebook \
  --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf \
  --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json \
  --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite \
  --report /tmp/clinton-corpus-qa-resume-20261128.json
```

**Exit code: 0 — PASS.** The aggregate-only report is [CLINTON-CORPUS-QA-03.json](CLINTON-CORPUS-QA-03.json).

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Read-only source and case integrity | **PASS** | PDF SHA-256 `00a9b9e879ffb1835cb216b69f2488e776726f6c239acccae5025b860d4805e9`; case SHA-256 `6143c16d3f66964991f80184d6e2a4d165553bf9443ac3cb6bfbeea930fa255b`; SQLite SHA-256 `fe9dcdf26f3810a0df3905ddf217cfd45704fd412f1509e1e379d2bb1483e071`. All three input hashes matched before and after the run. |
| Saved SQL automation | **PASS** | Query-only preflight passed; saved-run replay returned 6,173 rows, zero skipped rows, and zero added, removed, or changed-record deltas. Both retained batches are `COMPLETE`; both saved SQL runs are `PASS`. |
| Default-redacted analysis packet | **PASS** | 2,813 entities and 6,173 relationship records; 643 detected communities, 945 cross-community contact rows, and 291 proxy candidates. All entities appear in community membership; no names, relationship labels, IDs, or evidence references are included. |
| Rendered export geometry | **PASS** | 2,813 entity centers across 704 rows; largest row has 4 entities; zero labels outside the SVG viewport. |
| Saved chart positions | **INFO / follow-up** | 2,793 of 2,813 saved entity positions fall outside the current chart bounds. The redacted exported layout remaps them into the viewport; this acceptance does not verify native chart interaction or saved-position cleanup. |
| Full-source text completeness | **NOT ESTABLISHED** | This is SQL snapshot replay and analysis export acceptance, not a fresh extraction audit of the 7,121-page PDF. |
| Qwen semantic analysis and human content review | **NOT RUN** | No model output or human review is included. |
| Native visual and pointer review | **NOT RUN** | The headless export geometry check does not establish native UI behavior. |

This evidence contains aggregate counts and input hashes only. It does not establish the accuracy or completeness of the underlying email extraction, legal findings, or IBM Analyst's Notebook interoperability.
