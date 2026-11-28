# Clinton corpus QA 06 — current feature branch

## Handoff

- **Task / owner:** Re-run the primary real-corpus SQL acceptance against the installed Rust release; implementation session.
- **Repository / branch:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Branch head:** `4b8d1cd59d46605f2717256089589d722f26dcde`.
- **Product source:** `src/main.rs` Git blob `d233d2496380ab6008b671b2e77db7c0d07ff46f`, last changed at `2387e5a0bb994499ecff62f34f9f4e2e2bbb50d5`.
- **Installed Rust binary:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `1b918b2a9e94143467eef1da8905bfe06de080fe738228c6a9f853204850010f`.
- **Aggregate-only report:** [JSON](CLINTON-CORPUS-QA-06-current-branch-20261128.json), SHA-256 `81c9509fc0edb2f951858a98b6e8d35563296c4ef906a2f319c3d91ec1fe7f95`.
- **Data boundary:** The PDF, case, and SQLite source remain local. The committed JSON contains hashes and aggregate counts only; it contains no source names, record IDs, relationship labels, or email text.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Primary Clinton QA harness | PASS | `python3 scripts/test-clinton-corpus-acceptance.py --binary /home/sysadmin/.local/bin/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/CLINTON-CORPUS-QA-06-current-feature-branch.json` exited 0. |
| SQL query-only preflight | PASS | One saved job passed; zero failed; no case save. |
| Saved-run read-only replay | PASS | 6,173 rows, zero skipped rows, zero added/removed/changed-record deltas; no case save. |
| Default-redacted analysis packet | PASS | 2,813 entities, 6,173 relationships, 643 communities, 945 cross-community contact rows, 291 proxy candidates; all 2,813 entity centers rendered, 704 rows, largest row 4, zero labels outside the SVG viewport. Names, labels, IDs, and evidence references were not included. |
| Source immutability | PASS | Harness verified the PDF, case, and SQLite hashes were unchanged. |
| PDF completeness, Qwen analysis, human content review | NOT ESTABLISHED / NOT RUN | This SQL and aggregate packet check does not validate email extraction or semantic content. |
| Position normalization follow-up | OPEN | The case still contains 2,793 saved entity positions outside the normalized `[0,1]` range. Packet geometry remaps them for export; this run does not clean or rewrite saved chart positions. |

## Next action

Keep the real corpus as a read-only SQL regression fixture. Separately investigate and review the 2,793 out-of-range saved positions through a disposable case copy before proposing any case migration or interactive position rewrite. Keep Qwen GPU inference and human content review as distinct gates.
