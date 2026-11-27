# Local Linux install and launcher sync — current branch

- **Task / owner:** Sync the user-level desktop install with the current Arch Notebook branch; `/root`.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source commit:** `e86814675eaee96ed6f20caff94d6d5b79b9a2c5`.
- **Host:** Linux x86_64; `rustc 1.100.0 (0ef1bc386 2026-11-09)`.
- **Release binary SHA-256:** `d4bfcdc0e8f7fb3c374e92b4ebca2e21b2920e3211550a2b9536b6ac88d35676`.

## Commands and results

| Check | Result | Evidence |
| --- | --- | --- |
| Optimized offline build | **PASS**, exit 0 | `CARGO_TARGET_DIR=/tmp/arch-notebook-release-verify cargo build --release --locked --offline -j 1`; cargo reported the optimized release profile finished. |
| User install | **PASS**, exit 0 | Staged the current binary and repository desktop entry, SVG/PNG icons, MIME definition, and Linux installer under `/tmp/arch-notebook-sync-20261126/arch-notebook-linux-x86_64`, then ran `scripts/install-linux-user.sh` on that directory again after the current release build. |
| Installed executable | **PASS** | `/home/sysadmin/.local/bin/arch-notebook` SHA-256 matches the release binary above. `--version --json` returned `{"name":"arch-notebook","schema_version":1,"version":"0.1.0"}`. |
| Menu and desktop shortcut | **PASS** | Installed menu entry passes `desktop-file-validate`; it compares byte-for-byte with `~/Desktop/Arch Notebook.desktop`. Both point to `/home/sysadmin/.local/bin/arch-notebook` and use `Icon=arch-notebook`. |
| Icon and automation installer assets | **PASS** | Installed SVG and PNG compare byte-for-byte with repository assets (SHA-256 `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8` and `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`). The installed automation installer compares byte-for-byte with `scripts/install-linux-automation.sh`. |
| Case-file MIME association | **PASS** | After `update-desktop-database ~/.local/share/applications`, `gio mime application/vnd.arch-notebook.case+json` names `arch-notebook.desktop` as the default and recommended application; `xdg-mime query default` returns `arch-notebook.desktop`. |
| Native launch and interaction | **NOT RUN** | `DISPLAY=:0 xwininfo -root -tree` could not open the display. The systemd user bus also denied access; no live timer dispatch is claimed. |

The install updates only the current user's executable, launcher, MIME registration, and icons. No case file or SQLite source was opened or changed. This is local installation evidence, not a published release or IBM compatibility validation.
