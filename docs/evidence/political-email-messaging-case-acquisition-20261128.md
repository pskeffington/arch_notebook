# Political email and messaging-record case acquisition QA — 2026-11-28

**Task / owner:** Stage approximately 10 recent public political email or messaging-record cases for private ingestion QA / orchestrator.
**Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
**Source baseline:** local `5e2cf7fad4c9928eae4621e09db564edc6464abb`; tree `dc1db1817fe7a66eaed076e638307b9bb8496bc9`, matching the last verified remote feature tree.
**Scope:** Aggregate provenance and acquisition evidence only. Source PDFs remain in `/home/sysadmin/TRANSCRIBE-QA/political-email-cases-2026-11-28/recent-record-cases/`; no full PDFs or extracted content are added to Git.

## Source selection

All ten documents are official National Archives and Records Administration (NARA) case letters linked from [NARA's Unauthorized Disposition of Federal Records index](https://www.archives.gov/records-mgmt/resources/unauthorizeddispositionoffederalrecords). This is a mixed recent set of email, messaging, and federal record-retention disputes, with one 2020 comparator. The letters are not the underlying mailbox/chat data. Several entries remain allegations or pending investigations. NARA's administrative case classification is not a criminal adjudication.

| Case | Topic | NARA index status |
| --- | --- | --- |
| UD-2020-0001 | Commerce Secretary Wilbur Ross personal email | Founded |
| UD-2022-0055 | DHS senior official phone-reset text-message loss allegation | Pending review/follow-up |
| UD-2023-0052 | DHS OIG Signal and text-message retention | Ongoing investigation |
| UD-2024-0056 | DOJ OneDrive account loss | Founded |
| UD-2024-0075 | DoD OIG SIPRNET/JWICS email-record loss | Pending review/follow-up |
| UD-2024-0079 | USDA potential loss of personal-email records | Pending review/follow-up; source ID discrepancy flagged |
| UD-2025-0019 | Former U.S. Attorney records removal and non-official email/text use | Founded |
| UD-2025-0039 | FEMA Microsoft Teams Pride chat allegation | Unfounded |
| UD-2026-0001 | Interim U.S. Attorney Signal communications with a journalist | Pending review/follow-up |
| UD-2026-0007 | DOJ Working Group auto-deleting messaging allegation | Pending review/follow-up |

The USDA case index labels the case UD-2024-0079, but the direct linked PDF URL basename contains UD-2024-0078. The source was preserved as linked, and the discrepancy is marked in the local manifest for human reconciliation.

## Acceptance results

| Gate | Result | Evidence |
| --- | --- | --- |
| Download official source PDFs | **PASS** | Ten `curl -fL --retry 2 --connect-timeout 15 --max-time 90 <URL> -o <file>` commands exited 0. Direct URLs, source page, and item hashes are in the local manifest. |
| PDF file and page inventory | **PASS** | `pdfinfo` exited 0 for 10/10, totaling 21 pages. Total size: 1,639,861 bytes. |
| Embedded text-layer diagnostic | **PASS, diagnostic only** | `pdftotext -layout -enc UTF-8 <PDF> -` exited 0 for 10/10 and emitted 29,187 bytes total to transient stdout for counting. No extracted text was saved. This does not establish complete/page-aligned extraction or OCR correctness. |
| SHA-256 integrity | **PASS** | `cd /home/sysadmin/TRANSCRIBE-QA/political-email-cases-2026-11-28/recent-record-cases && sha256sum -c SHA256SUMS`; all 10 files returned `OK`. |
| TRANS cleanup/transcription | **BLOCKED** | The accepted local TRANS PDF adapter remains `ocr_stub`; no accepted `trans-document-batch` executable is installed. No processing output was fabricated. |
| Eagle-Eye page evidence | **BLOCKED** | No accepted PDF-to-page worker is installed; the documented pointer worker requires an approved page manifest. |
| Qwen analysis | **BLOCKED / NOT RUN** | `qwen` is absent. `ollama list` exited 1 because the local `127.0.0.1:11434` endpoint is inaccessible (`operation not permitted`). No model analysis ran. |
| Arch Notebook PDF ingestion | **NOT RUN** | This Rust application does not currently expose raw-PDF ingestion. Its Clinton SQL QA remains a separate database workflow. |
| Human legal/content review and redistribution rights | **NOT RUN** | No factual or legal conclusions were made; source materials remain local. |

## Evidence locations and next action

- Local source package: `/home/sysadmin/TRANSCRIBE-QA/political-email-cases-2026-11-28/recent-record-cases/README.md`, `manifest.json`, `SHA256SUMS`.
- No corpus PDFs, page images, or full extracted text were committed.
- Next executable stage: restore an approved general-purpose PDF-to-page and TRANS text/OCR path and an accessible Qwen model; process these source letters into private temporary artifacts, with the USDA identifier discrepancy kept review-required.

**Result:** Acquisition and structural QA **PASS**. Requested TRANS → Qwen → Arch Notebook processing **BLOCKED / NOT RUN**.
