# Political email/records QA bundle processing follow-up

- **Task / owner:** Reconcile staged public-record PDFs with the Arch Notebook ingestion acceptance record / orchestrator.
- **Arch Notebook source:** local candidate `184cf3894d5be6698f30fae1f0828c5927d1fe89`; product code unchanged in this follow-up.
- **Related repositories:** accepted Eagle-Eye `main` `c68e9e5de5f16e2b71cb71583a68f789c8dbe178`; accepted trans `main` `f59e9578e26540a87610e82009e5b877835b6632` (reverified from GitHub refs). Eagle-Eye contains an optional local `LocalOcrVisualBackend` (`pypdfium2` + `pytesseract`, default 10-page cap) that returns bounded public-notice observations, not full text or a general PDF transcription packet. Those Python modules are unavailable in the current system interpreter. The local trans checkout is a dirty, behind feature branch and was not used or modified.
- **Source directory:** `/home/sysadmin/TRANSCRIBE-QA/political-email-cases-2026-11-28`; PDFs, manifests, extraction outputs, and page images are not committed.

## Current bundle integrity

| Gate | Result | Evidence |
| --- | --- | --- |
| Base source checksums | PASS | `cd /home/sysadmin/TRANSCRIBE-QA/political-email-cases-2026-11-28 && sha256sum -c SHA256SUMS`; all 11 base PDFs returned `OK`. |
| Recent NARA supplement checksums | PASS | From `recent-nara-supplements/`, `sha256sum -c SHA256SUMS`; both PDFs returned `OK`. |
| File/page inventory | PASS, bounded | 13 PDFs, 2,850 total pages according to `pdfinfo`. The inputs span 10 topics and mix released emails, reports, correspondence, hearings, and release notices; they are not ten complete mailboxes or uniformly recent cases. |
| Supplement 1 parser/render check | PASS with warning | Poppler `pdfinfo` and `pdftoppm` both emit `Syntax Error: Expected the optional content group list, but wasn't able to find it, or it isn't an Array`; `pdfinfo` still reports 496 pages. Ghostscript `gs -q -dNOPAUSE -dBATCH -sDEVICE=nullpage <PDF>` exited 0 with no output while reading the full 31,112,222-byte file. `pdftoppm -f 1 -l 1 -r 72 -singlefile -png <PDF> /tmp/nara-pdf-check-page` exited 0, produced a 34,293-byte PNG, and its first page was visually inspected. The source hash stayed `6d2846a86cd2e375a4b8f27790bcabfb4d1ba132f87c7bf72195aa217a04d88b`. The Poppler warning remains recorded; no content completeness claim is made. |

## Processing gates

| Stage | Result | Evidence |
| --- | --- | --- |
| Eagle-Eye PDF-to-page processing | BLOCKED | No `eagle-eye-ingestion-worker` executable is installed. Accepted `main` documents a pointer-only request that requires an already approved page manifest; it is not a PDF-to-page runner. No page manifest was fabricated for these sources. |
| TRANS PDF text/OCR | BLOCKED | No `trans-document-batch` executable is installed. Accepted `trans` `src/tran/pdf/ocr.py` identifies its `PdfOcrAdapter` as a deterministic `ocr_stub`; without an input text hint it emits a placeholder candidate. It is not suitable for extracting these PDFs. |
| Qwen analysis of this bundle | NOT RUN | No `qwen` executable was found. The prior local Qwen run analyzed only aggregate metrics from the separate Clinton SQLite candidate; it did not read these PDFs. |
| Poppler embedded-text availability diagnostic | PASS, diagnostic only | `pdftotext -layout -enc UTF-8` exited 0 for 13/13 PDFs and emitted 5,579,267 bytes. Its form-feed output had 2,842 segments versus 2,850 `pdfinfo` pages; one 31-page report yielded only 23 segments. It does not establish page alignment, complete emails, OCR accuracy, or semantic correctness. No text was written to disk. See [text-layer inventory](political-email-text-layer-inventory-2026-11-28.md). |
| Arch Notebook PDF ingestion | NOT RUN | Arch Notebook has no raw-PDF import command. The Clinton SQL case and its replay are separate from this 13-PDF bundle. |
| Human content/rights review | NOT RUN | No factual finding, allegation assessment, or redistribution-rights conclusion was made. |

The Linux scheduler bus is inaccessible in this session, so native systemd timer delivery remains untested; stub harness results remain the available scheduler evidence. No PDFs or rendered page images were copied to Git or the archive share, and no source files were modified.

**Next executable gate:** provide an approved general-purpose PDF-to-page adapter and real TRANS OCR/text backend on accepted branches, then process selected public-record sources into private temporary artifacts with source/page/render digests and review-required status. The public-notice-specific Eagle-Eye backend and the Poppler diagnostic are not sufficient for these email/records cases.
