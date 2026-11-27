# LOCAL-INSTALL-03 · current branch desktop synchronization

- **Task / owner:** Synchronize the current-user desktop install with the current Arch Notebook release; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source commit:** `4c132cf3d2b5f4bee0bd1f9625f1dfa66fd57f52`.
- **Host:** Linux x86_64.
- **Release / installed binary SHA-256:** `1a7331856c8498fa72cad1148a3a31c11a20d1a29b967226d9ee517334879392`.
- **Install scope:** Current user; no `sudo` or system-wide package changes.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0. |
| User install | PASS | Staged the release binary and repository desktop/MIME/icon/automation assets under `/tmp/arch-notebook-sync-4c132cf/arch-notebook-linux-x86_64`; `scripts/install-linux-user.sh /tmp/arch-notebook-sync-4c132cf/arch-notebook-linux-x86_64` exited 0. |
| Installed executable | PASS | SHA-256 matches the release binary above; `/home/sysadmin/.local/bin/arch-notebook --version --json` returned `{"name":"arch-notebook","schema_version":1,"version":"0.1.0"}`. |
| Menu and desktop shortcut | PASS | `desktop-file-validate` passed for both entries; `cmp` confirms `/home/sysadmin/.local/share/applications/arch-notebook.desktop` and `/home/sysadmin/Desktop/Arch Notebook.desktop` are identical. |
| Deployment icon artwork | PASS | Installed SVG and 1024px PNG compare byte-for-byte with repository assets. SVG SHA-256: `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8`; PNG SHA-256: `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`. |
| Automation helper | PASS | Installed helper compares byte-for-byte with `scripts/install-linux-automation.sh`. |
| Case file association | PASS | `gio mime application/vnd.arch-notebook.case+json` reports `arch-notebook.desktop` as default and recommended; `xdg-mime query default` returns `arch-notebook.desktop`. |
| Native launch and visual review | NOT RUN | `WAYLAND_DISPLAY=wayland-1 wayland-info` returned `failed to create display: Operation not permitted`. This install verification does not establish a rendered or interactive current-candidate window. |

The install updates the user executable, launcher, MIME registration and icon files. No case file or SQLite database was opened or modified. The icon artwork is unchanged from the repository design and is synchronized with the launcher. This is a local install result, not a published release or IBM interoperability validation.
