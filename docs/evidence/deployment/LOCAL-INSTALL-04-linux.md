# LOCAL-INSTALL-04 · proxy-focus candidate desktop synchronization

- **Task / owner:** Install the current bridge-proxy navigation candidate and keep the desktop launcher assets synchronized; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source commit:** `e27ad5e842dd1c6d503f05c51844f12790502fa6`.
- **Verification source:** `d0ce0c77e9fcee445536d0f55aceebf70e5fbd84` (test-only; runtime build output unchanged).
- **Host:** Linux x86_64.
- **Release / installed binary SHA-256:** `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.
- **Install scope:** Current user only; no `sudo` or system-wide package changes.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0. |
| User install | PASS | Staged the release executable and repository desktop/MIME/icon/automation assets under `/tmp/arch-notebook-sync-e27ad5e/arch-notebook-linux-x86_64`; `scripts/install-linux-user.sh /tmp/arch-notebook-sync-e27ad5e/arch-notebook-linux-x86_64` exited 0. |
| Installed executable | PASS | Installed SHA-256 matches the release binary above; `--version --json` returned `{"name":"arch-notebook","schema_version":1,"version":"0.1.0"}`. |
| Menu and desktop shortcut | PASS | `desktop-file-validate` passed for both entries; `cmp` confirms `/home/sysadmin/.local/share/applications/arch-notebook.desktop` and `/home/sysadmin/Desktop/Arch Notebook.desktop` are identical. |
| Deployment icon artwork | PASS | Installed SVG and 1024px PNG compare byte-for-byte with repository assets. SVG SHA-256: `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8`; PNG SHA-256: `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`. |
| Automation helper | PASS | Installed helper compares byte-for-byte with `scripts/install-linux-automation.sh`. |
| Case file association | PASS | `gio mime application/vnd.arch-notebook.case+json` lists `arch-notebook.desktop` as default and recommended; `xdg-mime query default` returns `arch-notebook.desktop`. |
| Native launch and visual review | NOT RUN | `wayland-info` returned `failed to create display: Operation not permitted`. Install checks do not establish a rendered or interactive current-candidate window. |

This local install updates the executable, menu entry, desktop shortcut (already byte-identical), MIME registration, automation helper, and icon files. No case file or SQLite database was opened or modified. This is not a published release or IBM interoperability validation.
