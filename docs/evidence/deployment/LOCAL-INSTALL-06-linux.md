# LOCAL-INSTALL-06 · dense-label graph review candidate

- **Task / owner:** Install the current graph readability candidate for local review; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source commit:** `993dd560efca942fc4cc4c021d2fbefab1c96eef`.
- **Host:** Linux x86_64; current-user installation only.
- **Release SHA-256:** `d28f2420fa51680046764546af22be7a850827cc615873eb1633267aea1f3676`.
- **Review fixture:** `/tmp/arch-notebook-review-sparse30.case.json`; 30 fictional entities, 90 relationship records; SHA-256 `bf88b5df8c2c8e259e41a52486d8f09a12f6a6665d3a60892c802d8c06fa872d`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Per-user installer | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-sync-993dd56/arch-notebook-linux-x86_64` exited 0. |
| Installed executable identity | **PASS** | `/home/sysadmin/.local/bin/arch-notebook` matches the release SHA-256 above; `--version --json` returned `{"name":"arch-notebook","schema_version":1,"version":"0.1.0"}`. |
| Desktop entry | **PASS** | `desktop-file-validate /home/sysadmin/.local/share/applications/arch-notebook.desktop` exited 0. |
| Case MIME association | **PASS** | `gio mime application/vnd.arch-notebook.case+json` lists `arch-notebook.desktop` as default. |
| Native candidate launch/render | **BLOCKED** | `/home/sysadmin/.local/bin/arch-notebook /tmp/arch-notebook-review-sparse30.case.json` exited 1 with `WaylandError(Connection(NoCompositor))`; no candidate window rendered. |
| Pointer and keyboard interaction | **NOT TESTED** | No candidate window was available. |

The product executable and launcher now point to the same candidate as the headless dense-label regression. The review chart is fictional and stored under `/tmp`. The original app screenshot predates this candidate and does not count as its native render evidence. This local installation is not a published release or proof of interactive acceptance.
