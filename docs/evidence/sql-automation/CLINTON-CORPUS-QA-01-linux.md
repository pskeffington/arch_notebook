# CLINTON-CORPUS-QA-01 · primary real-corpus acceptance

- **Task / owner:** Run Arch Notebook SQL and graph-analysis QA against the local Clinton email candidate corpus; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source candidate:** topology-rendering changes in the current worktree, based on `eb0f97e32ea2f46d41b5f94f75da86264de34e03`.
- **Installed Rust release SHA-256:** `7a7f9ebc6111185c80aeed1b98746b93127738e7eea588567b29bb8a437c3924`.
- **Source PDF:** `//archive.local/Research/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf`.
- **Local candidate files:** `/home/sysadmin/Research.Databases/TRANSCRIBE/`; the source PDF, case, and SQLite database remain outside Git.
- **Aggregate report:** [CLINTON-CORPUS-QA-01 JSON](artifacts/CLINTON-CORPUS-QA-01-linux.json), SHA-256 `d28ab6062b7840705ea020d8ff0b00554c248d5162fa57a50eca58729fc3a4d3`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Real source, case, and SQL inputs | **PASS** | The report records source hashes only. The files are a local review-required candidate derived from the user-provided TRANSCRIBE PDF. No row-level names, addresses, subjects, message bodies, identifiers, or evidence were written to this repository. |
| Saved candidate shape | **PASS** | 2,813 entities, 6,173 chart relationship records, two SQL snapshots with 6,173 rows each, two `PASS` SQL runs, and two `COMPLETE` batches. |
| Saved SQL query preflight | **PASS** | The saved job passed query-only validation with one passing job, zero failures, and `saved=false`. The disabled saved job was validated directly; it was not enabled or modified. |
| Latest saved-run replay | **PASS** | Read-only replay returned 6,173 relationship rows, zero skipped rows, and zero added, removed, or changed records. |
| Real-corpus network packet | **PASS** | The installed release generated a temporary default-redacted finding packet for all 2,813 entities and 6,173 relationships. Its aggregate report contains 643 detected structural communities, 945 cross-community contact rows, and 291 cross-community proxy candidates; all 2,813 entities are represented in the communities. Output defaults omitted names, relationship labels, IDs, and evidence references; temporary permissions were 0700 for the packet directory and 0600 for both files. |
| Topology rendering in the report SVG | **PASS** | All 2,813 entity centers appear on 704 distinct y rows (largest same-y row: 4); zero entity labels fall outside the 1,200×800 viewport. The saved case has 2,793 entities outside the conventional 0–1 coordinate range. The renderer maps entity bounds to the viewport without changing saved positions; the active frame is stored per chart so drag rendering, packet SVGs, chart switching, and reopened cases use the same bounds. Headless regressions cover drag geometry, annotation-independent bounds, degenerate/extreme coordinates, and frame serialization. |
| Source immutability | **PASS** | PDF, case, and SQLite SHA-256 values were identical before and after query preflight, saved-run replay, and packet export. The packet was created inside a temporary directory and automatically removed. |
| Qwen analysis and human content review | **NOT RUN** | This acceptance verifies Arch Notebook's existing candidate data and SQL query. It does not run Qwen or validate message meaning. |
| PDF extraction completeness | **NOT ESTABLISHED** | Earlier local processing recorded 4,767 header candidates from 7,121 PDF pages. This is not proof that all approximately 7,000 emails were extracted or correctly interpreted. |
| Native interaction on the real case and IBM interoperability | **NOT TESTED** | The packet is a CLI analysis of the real case; it does not establish native-window visual acceptance or IBM XSD/application compatibility. The separate fictional native smoke records left-edge clipping and is not Clinton-case visual acceptance. |

## Reproduction

Run this local-corpus check before treating synthetic fixture CI as evidence for the real workload:

```sh
python3 scripts/test-clinton-corpus-acceptance.py \
  --binary /home/sysadmin/.local/bin/arch-notebook \
  --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf \
  --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json \
  --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite \
  --report /tmp/clinton-corpus-qa.json
```

The script emits aggregate counts and file hashes only. It does not modify the case or database, enable the saved job, run PDF extraction, or include any corpus file in Git. If the local corpus is unavailable, record this gate as **NOT RUN**; synthetic workloads remain supporting regression coverage rather than a substitute for the real batch.
