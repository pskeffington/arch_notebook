# Political email/records PDF text-layer inventory — 2026-11-28

**Task / owner:** Establish whether the staged public-record PDFs expose embedded text suitable for a later approved TRANS processing run / orchestrator.

**Source directory:** `/home/sysadmin/TRANSCRIBE-QA/political-email-cases-2026-11-28` (13 PDFs; source files remain outside Git).
**PDF QA manifest:** `political-email-qa-inputs/v1`; source and supplement SHA-256 manifests were rechecked and passed.
**Processor:** Poppler `pdftotext -layout -enc UTF-8`, with stdout captured in memory and reduced immediately to counts. No extracted text or per-page text files were written.

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| PDF command availability | PASS | `/usr/bin/pdftotext` and `/usr/bin/pdfinfo` are installed. |
| Text-layer command | PASS, diagnostic only | `pdftotext -layout -enc UTF-8 <PDF> -` exited 0 for all 13 files; combined stdout was 5,579,267 bytes. |
| Extracted nonempty segments | OBSERVED, not a page count | 2,560 nonempty form-feed-delimited text segments were observed. The output had 2,842 segments while `pdfinfo` reports 2,850 pages. Empty pages can collapse in text output; the segments cannot be treated as page-aligned evidence. |
| Concrete page alignment gap | FAIL for page-level completeness | `mark-foley-email-investigation-doj-oig-govinfo.pdf` reports 31 pages in `pdfinfo` but produced 23 text segments. The aggregate parser therefore cannot prove a segment-to-page mapping or blank-page coverage. |
| Candidate email extraction / semantic completeness | NOT ESTABLISHED | The inputs mix emails, correspondence, reports, indexes, release notices, and hearing transcripts. Character counts and text presence do not establish unique messages, complete email headers, OCR accuracy, case findings, or representation of all records. |
| TRANS processing and Qwen analysis | NOT RUN | This was a Poppler availability diagnostic only. No extracted text was sent to TRANS or Qwen, and no Arch Notebook case was created from these PDFs. |

The evidence narrows the next work: several PDFs have a text layer that a future accepted TRANS adapter may consume, but it must preserve real page identity rather than infer it from form-feed segments. Scanned/blank pages need page-bounded inspection or an approved render/OCR route. Keep candidate text review-required and preserve source/page digests in the owning pipeline.
