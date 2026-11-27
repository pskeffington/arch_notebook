# CLINTON-CORPUS-EXTRACT-01 — PDF text-layer coverage

- **Task / owner:** Measure local PDF text-layer coverage and Qwen availability; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source PDF:** user-provided TRANSCRIBE candidate; SHA-256 `00a9b9e879ffb1835cb216b69f2488e776726f6c239acccae5025b860d4805e9`.
- **Aggregate report:** [text-layer coverage JSON](artifacts/CLINTON-CORPUS-EXTRACT-01-text-layer-coverage-20261127.json), containing counts and status only.

## Results

- **PASS — page-level extraction:** `pdfinfo` reports 7,121 pages. `pdftotext -layout -enc UTF-8` produced a nonempty text layer for every page (7,121/7,121); zero pages were blank in the extracted text.
- **NOT ESTABLISHED — email extraction completeness:** an internal count found header-field text on 4,658 pages (41,208 occurrences of From/Sent/To/Cc/Subject/Date labels). Page fields are not unique-message IDs and do not prove that every intended email was extracted or interpreted correctly. This count is not reconciled with the earlier 4,767 extraction-candidate count because the detectors differ.
- **BLOCKED — Qwen analysis:** no Qwen model tool is exposed. `ollama list` could not connect to `127.0.0.1:11434` (`socket: operation not permitted`), and `systemctl --user` could not access the user service bus (`Operation not permitted`). No corpus content was sent to a remote model.
- **PASS — temporary-content cleanup:** the generated 12,307,325-byte text dump was deleted after aggregate counts were computed. No extracted PDF text was added to Git.

## Reproduction

```sh
pdfinfo /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf
pdftotext -layout -enc UTF-8 \
  /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf \
  /tmp/clinton-emails-text-layer.txt
```

Run this aggregate-only scan against the extracted text, then remove the temporary text file. It emits counts only and does not display page text:

```sh
python3 - <<'PY'
from pathlib import Path
import re

text = Path("/tmp/clinton-emails-text-layer.txt").read_text(
    encoding="utf-8", errors="replace"
)
pages = text.split("\f")
if pages and not pages[-1].strip():
    pages.pop()
headers = re.compile(r"^\s*(?:From|Sent|To|Cc|Subject|Date):", re.I | re.M)
print({
    "text_layer_pages": len(pages),
    "nonempty_text_pages": sum(bool(page.strip()) for page in pages),
    "pages_with_email_header_field_text": sum(bool(headers.search(page)) for page in pages),
    "email_header_field_occurrences": sum(len(headers.findall(page)) for page in pages),
    "extracted_text_bytes": len(text.encode("utf-8")),
})
PY
rm /tmp/clinton-emails-text-layer.txt
```

Text-layer presence is a coverage measure, not an OCR-accuracy or email-completeness test.
