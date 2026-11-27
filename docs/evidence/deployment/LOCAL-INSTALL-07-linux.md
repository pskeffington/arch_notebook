# LOCAL-INSTALL-07 · path-redacted packet-export review candidate

- **Task / owner:** Install the validated current feature candidate for follow-up review; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source candidate:** `ad737f9abc10e62f1c1eb4bc3d1a617104eb18f2` (the installed release bytes match the validated candidate).
- **Host:** Linux x86_64; per-user installation only.
- **Release and installed executable SHA-256:** `604e95d768748e8543a91325fdc8917b7aab1dd68770b0a8b0e83ccf58f419fa`.

## Install verification

| Check | Result |
| --- | --- |
| Current-user launcher install | **PASS** — `scripts/install-linux-user.sh /tmp/arch-notebook-security-privacy-02/arch-notebook-linux-x86_64` exited 0. |
| Executable identity | **PASS** — installed `/home/sysadmin/.local/bin/arch-notebook` and release candidate hashes match. |
| Version and desktop entry | **PASS** — `--version --json` reports Arch Notebook 0.1.0; `desktop-file-validate` exited 0. |
| Case-file association | **PASS** — `gio mime application/vnd.arch-notebook.case+json` lists `arch-notebook.desktop` as default and recommended. |
| Existing review window | **PRESENT** — `xwininfo -root -tree` exited 0 and listed `Arch Notebook · Link Analysis` (`0x1000003`, 1440×900). |
| Current chart screenshot / interaction | **PARTIAL** — an active-window capture showed a terminal command-approval screen, which was not accepted. A later full-desktop capture confirmed the existing `VECTOR UI SPARSE 30` chart was visible; it still does not identify the binary loaded by that process, and no input was delivered. See [dense-label screen follow-up](../vector-ui/G4-DENSE-LABEL-AUTO-01-linux.md#existing-window-follow-up). |

The installed binary was built and passed the local offline checks listed in [PACKET-05](../analyst-packet/PACKET-05-privacy-path-redaction-linux.md). The existing window may be running a prior candidate; the installed executable hash does not identify a process that was already open. The captured chart needs an input-capable review after **Fit all**, with the candidate process and binary hash identified. No operational case was opened.
