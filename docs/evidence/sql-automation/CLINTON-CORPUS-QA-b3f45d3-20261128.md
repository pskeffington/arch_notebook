# CLINTON-CORPUS-QA-b3f45d3 · current candidate SQL revalidation

**Task / owner:** Re-run the primary Clinton corpus acceptance against the current Rust candidate / implementation session.

**Repository / branch:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions`.
**Product source commit:** `b3f45d3d884932f9e443da778e2a29512def741b`.
**Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `459e00f5ea8b8de7397710d3a655a6a36ef3b015d3c05f685aed34a6528e8b3a`.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Primary corpus acceptance harness | PASS | `python3 scripts/test-clinton-corpus-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/CLINTON-CORPUS-QA-b3f45d3.json`, exit 0. |
| Query-only preflight and saved-run replay | PASS | One job passed. The replay returned 6,173 relationship rows, zero skipped rows, and zero added, changed, or removed deltas. |
| Default-redacted topology packet | PASS | 2,813 entities and 6,173 records; all centers rendered over 704 y-rows; zero labels outside the viewport; 643 communities and 291 proxy candidates. Names, relationship labels, IDs, and evidence references were excluded. Packet directory mode `0700`; report and SVG modes `0600`. |
| Input immutability | PASS | Harness confirmed unchanged PDF, case, and SQLite hashes. |
| Aggregate artifact integrity | PASS | [Aggregate-only JSON](artifacts/CLINTON-CORPUS-QA-b3f45d3.json), SHA-256 `7756d5c17f3ddae5585b0a6637f4d0769fc1d8720d1e83494139d742a8d29058`. |
| PDF extraction completeness, Qwen analysis, human content review | NOT ESTABLISHED / NOT RUN | This harness replays saved SQL and exports default-redacted graph aggregates; it does not validate email content. |
| Live timer dispatch or external-update UI with a real scheduled run | NOT TESTED | The separate external-update regression uses a controlled file replacement; systemd was not installed or awaited. |
| IBM ANX compatibility | NOT TESTED | No IBM schema or application was used. |

The aggregate JSON contains counts, digests, statuses, and explicit redaction flags only; it contains no row-level email or chart records.
