# Clinton corpus QA 07 — coordinate normalization candidate release

## Handoff

- **Task / owner:** Re-run the primary real-corpus SQL regression against the release containing chart coordinate normalization; implementation session.
- **Repository / branch:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `4b8d1cd59d46605f2717256089589d722f26dcde`.
- **Candidate product source Git blob:** `src/main.rs` `0313cf6f951678557ec14a7f8c5bfdfe77e3be17`.
- **Installed Rust binary:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `6ec6622ef41c14675a1e702185cc8b8548eaaad1eae860cffd063370c37a70a0`.
- **Aggregate report:** [JSON](CLINTON-CORPUS-QA-07-normalized-coordinates.json), SHA-256 `d4505a509a397a1da4450d1bb507a0e84f23361a640bbc978bc14a988983099b`.
- **Data boundary:** The PDF, candidate case, and SQLite source remain local and unchanged. The committed report contains hashes and aggregate counts only.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Primary Clinton QA harness | PASS | `python3 scripts/test-clinton-corpus-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/CLINTON-CORPUS-QA-07-normalized-coordinates.json` exited 0. |
| Saved SQL query-only preflight | PASS | One job passed; zero failed; case not saved. |
| Saved-run replay | PASS | 6,173 rows, zero skipped rows, zero added/removed/changed-record deltas. |
| Default-redacted topology packet | PASS | 2,813 entities; 6,173 relationships; 643 communities; 945 cross-community contact rows; 291 proxy candidates; all 2,813 entity centers rendered; zero labels outside the SVG viewport. No names, IDs, relationship labels, or evidence references were included. |
| Read-only source immutability | PASS | PDF, case, and SQLite SHA-256 values match their pre-run values. |
| Position data result | OPEN | 2,793 saved entity positions remain outside `[0,1]`; the harness does not activate or save the new UI normalization action. |
| Qwen analysis, PDF completeness, human content review | NOT RUN / NOT ESTABLISHED | SQL replay and redacted topology export do not establish document transcription or semantic analysis. |

## Next action

Evaluate coordinate normalization interactively on a writable case copy. Do not treat this SQL regression as visual acceptance or a content review.
