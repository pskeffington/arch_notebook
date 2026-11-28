# Public political email and records QA intake

- **Task / owner:** Source and stage public political email/records inputs for Arch Notebook QA / orchestrator.
- **Baseline:** repository `pskeffington/arch_notebook`, branch `codex/ui-00-chart-baseline`, remote source commit `2a1aebf7497e09b958e837a1996a9029fedf5316`, tree `fd95cf38042d432c0feeb22435305356933f0d4a`. Local checkout was at `fed9290f37a642b260f935138a7d5cdbc65221cf` with the same tree and a clean worktree before this evidence entry.
- **Scope:** 11 official-source PDFs covering 10 case topics, plus one supplemental Clinton/NARA letter. Raw PDFs and generated manifest remain outside Git at `/home/sysadmin/TRANSCRIBE-QA/political-email-cases-2026-11-28`; no email corpus was committed.

## Source set

| Topic | Local file | Source |
|---|---|---|
| Trump Mar-a-Lago records return: NARA Category 1 email release | `maralago-nara-category-1.pdf` | [NARA release page](https://www.archives.gov/foia/15-boxes) |
| CDC/HHS email deletion allegation during COVID-19 response | `hhs-cdc-email-deletion-allegation-nara.pdf` | [NARA records case index](https://www.archives.gov/records-mgmt/resources/unauthorizeddispositionoffederalrecords) |
| Clinton private email server / FBI and DOJ actions | `clinton-email-server-doj-oig-2018.pdf` | [DOJ OIG report](https://oig.justice.gov/sites/default/files/reports/18-04.pdf) |
| IRS missing email records / targeting allegations | `irs-missing-emails-hearing-govinfo.pdf` | [GovInfo hearing](https://www.govinfo.gov/content/pkg/CHRG-113hhrg91126/html/CHRG-113hhrg91126.htm) |
| White House missing email records / subpoena and campaign-finance inquiries | `white-house-missing-email-records-hearing-govinfo.pdf` | [GovInfo hearing](https://www.govinfo.gov/content/pkg/CHRG-106hhrg69621/html/CHRG-106hhrg69621.htm) |
| Mark Foley email messages / FBI initial response | `mark-foley-email-investigation-doj-oig-govinfo.pdf` | [DOJ OIG FBI reports](https://oig.justice.gov/archives/reports/fbi.htm) |
| Susan Rice records re alleged DNC/Clinton campaign system hacking | `susan-rice-dnc-clinton-hack-emails.pdf` | [Obama Presidential Library finding aid](https://obamalibrary.archives.gov/digital-research-room/finding-aids/emails-sent-or-susan-rice-related-alleged-hacking-democratic) |
| Kavanaugh White House Counsel email records release | `kavanaugh-email-records-release-notification.pdf` | [NARA Kavanaugh records page](https://www.archives.gov/news/topics/kavanaugh-records) |
| DOJ trial attorney use of non-official email and removal of records | `doj-nonofficial-email-records-misconduct-2025.pdf` | [DOJ OIG FOIA entry](https://oig.justice.gov/foia-reading-room/findings-misconduct-then-trial-attorney-violations-department-policy-regarding) |
| Pence vice-presidential records case | `pence-vice-presidential-records-case-nara-2023.pdf` | [NARA records page](https://www.archives.gov/foia/records-related-to-pence-vice-presidential-records-covered-by-the-presidential-records-act-pra) |
| Supplemental Clinton/NARA opening correspondence | `clinton-private-email-records-nara-opening-letter-2015.pdf` | [NARA correspondence collection](https://www.archives.gov/foia/state-formal-correspondence) |

These are a mixture of released email records, redaction/withdrawal indexes, official correspondence, hearing transcripts, and investigative reports. The Susan Rice PDF is a six-page withdrawn/redacted-material index, not the underlying collection; the Kavanaugh PDF is a release notice; the Pence matter is a records-preservation case rather than an email case. The NARA CDC case listing reports the allegation outcome as unfounded. Do not treat allegations in records or hearings as findings.

## Acceptance and evidence

- **Download:** PASS — 11/11 PDFs present; 36,380,445 bytes.
- **Format:** PASS — `file` and `pdfinfo` recognized 11 PDFs; 1,839 pages total.
- **Integrity:** PASS — `sha256sum -c SHA256SUMS` exited 0 for all 11 files. Local `manifest.json` SHA-256: `368782cea5ff390c5800d9274b72d1ab4a95c70430642265c8791ebd00f5f326`.
- **Provenance:** PASS — item-level source URLs, descriptions, byte sizes, and SHA-256 digests are in the local `manifest.json`; digest list is `SHA256SUMS`.
- **Transcribe/Eagle-Eye processing:** NOT RUN.
- **Qwen analysis:** NOT RUN.
- **Arch Notebook PDF import:** NOT RUN. The installed CLI help exposes no raw-PDF ingestion command; process PDFs through the upstream PDF-ingestion/evidence-handoff workflow. No case was created or modified.
- **Archive share copy:** BLOCKED — `/home/sysadmin/Research.Databases/TRANSCRIBE` is mounted read-only. No share files were changed.
- **Rights review:** NOT DONE — source PDFs are retained locally for QA; redistribution rights for embedded third-party material were not independently established.

The direct State OIG ESP-16-03 PDF endpoint returned HTTP 403; it was excluded. The set was completed using other official NARA/DOJ/GovInfo items.

Commands and observed outcomes: individual `curl -fLsS --retry 2 --max-time 120 <official-url> -o <local-file>` downloads exited 0 for the 11 included files; `file`/page-count inspection completed; `sha256sum -c SHA256SUMS` exited 0; `/home/sysadmin/.local/bin/arch-notebook --help` exited 0. No build or product tests were run.

**Changed repository path:** this evidence record only. No product code, raw source PDFs, Qwen output, or extracted email text is included.

The later Poppler check confirmed a text layer on many inputs but did not yield reliable page-aligned output: 2,842 delimited segments versus 2,850 pages reported by `pdfinfo`. This was an in-memory availability diagnostic only; no text was saved, sent to TRANS/Qwen, or imported into Arch Notebook. See the [aggregate text-layer inventory](political-email-text-layer-inventory-2026-11-28.md).
