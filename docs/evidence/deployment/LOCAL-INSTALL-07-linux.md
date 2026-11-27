# LOCAL-INSTALL-07 · path-redacted packet-export review candidate

- **Task / owner:** Install the validated current feature candidate for follow-up review; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source baseline:** `bc9e33de6d4a5b4ea11ac5acc15b5b1f79874dc6` plus the documented uncommitted PACKET-05 patch at validation time.
- **Host:** Linux x86_64; per-user installation only.
- **Release and installed executable SHA-256:** `604e95d768748e8543a91325fdc8917b7aab1dd68770b0a8b0e83ccf58f419fa`.

## Install verification

| Check | Result |
| --- | --- |
| Current-user launcher install | **PASS** — `scripts/install-linux-user.sh /tmp/arch-notebook-security-privacy-02/arch-notebook-linux-x86_64` exited 0. |
| Executable identity | **PASS** — installed `/home/sysadmin/.local/bin/arch-notebook` and release candidate hashes match. |
| Version and desktop entry | **PASS** — `--version --json` reports Arch Notebook 0.1.0; `desktop-file-validate` exited 0. |
| Case-file association | **PASS** — `gio mime application/vnd.arch-notebook.case+json` lists `arch-notebook.desktop` as default and recommended. |
| Native review interaction | **OPEN** — this install does not restart or manipulate the operator's existing review window. A native visual/input pass on the new executable has not been recorded. |

The installed binary was built and passed the local offline checks listed in [PACKET-05](../analyst-packet/PACKET-05-privacy-path-redaction-linux.md). The next screen review should relaunch the candidate with a fictional chart, choose **Fit all**, and inspect relationship-label density and entity-card bounds. No operational case was opened.
