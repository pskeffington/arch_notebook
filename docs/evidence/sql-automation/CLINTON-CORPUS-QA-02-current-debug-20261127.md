# CLINTON-CORPUS-QA-02 — current-source debug revalidation

- **Task / owner:** Re-run the primary real-corpus SQL and graph packet QA against the current-source binary; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source:** `ab6e29318fca406d748f6694c89b3e25cd06b347`. The subsequent `f8fe1ca` commit changes documentation only.
- **Candidate:** Linux debug binary, SHA-256 `74c900167f450e28f391506d043bfb17fe177382afe19d6f8fdd16bcab24e606`.
- **Aggregate report:** [CLINTON-CORPUS-QA-02 JSON](artifacts/CLINTON-CORPUS-QA-02-current-debug-20261127.json), SHA-256 `dc7c0f2074ed7f6dcd4eaf41a384dc769af07cfb4cc07641bb87f87230724b45`.

## Results

- **PASS — source inputs unchanged:** PDF, case, and SQLite hashes match before and after the harness. No corpus contents or row-level values are included in the report.
- **PASS — SQL query preflight:** one query-only job passed, zero failed; no case save occurred.
- **PASS — saved-run replay:** 6,173 relationship rows, zero skipped rows, and zero added, removed, or changed records.
- **PASS — redacted graph packet:** 2,813 entities and 6,173 relationships; default output includes no names, relationship labels, IDs, or evidence references. All entity centers and labels fit within the SVG viewport; largest same-y row is four.
- **NOT RUN — Qwen and human content review.**
- **NOT ESTABLISHED — PDF extraction completeness.**
- **NOT TESTED — Clinton native-window interaction and IBM Analyst's Notebook/XSD compatibility.**

The saved SQL history contains two passing runs, two complete batches, and two snapshots with 6,173 rows each. The packet's aggregate structural analysis found 643 communities, 945 cross-community contact rows, and 291 proxy candidates. These counts describe the saved candidate and do not validate email meaning.

## Reproduction

```sh
python3 scripts/test-clinton-corpus-acceptance.py \
  --binary /tmp/arch-notebook-current-target/debug/arch-notebook \
  --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf \
  --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json \
  --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite \
  --report /tmp/clinton-corpus-qa-current-debug.json
```

This reuses the read-only acceptance harness documented in [CLINTON-CORPUS-QA-01](CLINTON-CORPUS-QA-01-linux.md); its per-run packet is temporary and default-redacted.
