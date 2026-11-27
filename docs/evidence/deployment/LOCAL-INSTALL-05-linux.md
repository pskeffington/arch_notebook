# LOCAL-INSTALL-05 · community-packet candidate desktop synchronization

- **Task / owner:** Synchronize the current Arch Notebook candidate with the current user's desktop launcher and graphics; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source:** `280f5f0b3b0200170e12e730e4b0cce0358afc20`.
- **Host:** Linux x86_64; current-user install only, no `sudo` or system-wide package changes.
- **Release and installed executable SHA-256:** `4bc1d5b72d623a32293338b459633c3d38c1726db01edd958b136c50e635e4d1`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Release candidate installed | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-sync-280f5f0/arch-notebook-linux-x86_64` exited 0. Installed `/home/sysadmin/.local/bin/arch-notebook` matches the release binary SHA-256 above; `--version --json` returned `{"name":"arch-notebook","schema_version":1,"version":"0.1.0"}`. |
| Launcher and desktop shortcut | **PASS** | `desktop-file-validate` exited 0 for both entries; `cmp` exited 0 for `/home/sysadmin/.local/share/applications/arch-notebook.desktop` and `/home/sysadmin/Desktop/Arch Notebook.desktop`. Both SHA-256: `0997476bbec189ce6c3a98c22670cd4b8aebd989c721bcced4bf6eff5c141f4c`. |
| Deployment graphics and automation helper | **PASS** | Installed SVG and 1024px PNG compare byte-for-byte with repository artwork; SVG SHA-256 `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8`, PNG SHA-256 `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`. Installed SQL automation helper compares byte-for-byte with `scripts/install-linux-automation.sh`. |
| Case-file desktop association | **PASS** | `gio mime application/vnd.arch-notebook.case+json` lists `arch-notebook.desktop` as default and recommended. |
| Native candidate launch and render | **PASS, render only** | Launched `/tmp/arch-notebook-target/release/arch-notebook /home/sysadmin/github/arch_notebook/fixtures/chart/correctness.case.json` with X11. `xwininfo -root -tree` showed an Arch Notebook window at 1440×900; Spectacle captured the active window successfully. See [native screenshot](LOCAL-INSTALL-05-native-window-20261127.png), SHA-256 `702750fbf78acd4cd02c8b428597cf0a05b16e32bc5f369801e50bf8394c4419`. |
| Current-candidate pointer and keyboard interaction | **NOT TESTED** | `xdotool search --onlyvisible --name 'Arch Notebook'` could not create an XDO instance; direct `import` could not open display `:0`; a subsequent Spectacle capture aborted with exit 134. The successful capture is render evidence only. |
| IBM Analyst's Notebook visual/interchange validation | **NOT TESTED** | This install and screenshot do not validate IBM application behavior or ANX schemas. |

The displayed case is the committed fictional correctness fixture. No operational case or source SQLite database was opened or modified. This is a per-user local installation, not a published release or production deployment.
