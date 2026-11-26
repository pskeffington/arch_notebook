# LOCAL-INSTALL-02: proximity build and launcher synchronization

- **Task / owner:** Install the current Arch Notebook proximity-analysis build into the user's desktop launcher; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source commit:** `6b59e32524c1ee487394ca1235cef4716d8937ae`.
- **Release binary SHA-256:** `bd80d18f428655b095484d2f84ddd29ce5c4d369b6169e77d6040766b11d6309`.
- **Install scope:** Current user only; no system-wide install or `sudo`.

## Installed paths and artwork

- Executable: `/home/sysadmin/.local/bin/arch-notebook`
- Application-menu entry: `/home/sysadmin/.local/share/applications/arch-notebook.desktop`
- Desktop shortcut: `/home/sysadmin/Desktop/Arch Notebook.desktop`
- SVG icon: `/home/sysadmin/.local/share/icons/hicolor/scalable/apps/arch-notebook.svg`
- 1024px PNG icon: `/home/sysadmin/.local/share/icons/hicolor/1024x1024/apps/arch-notebook.png`
- Automation installer: `/home/sysadmin/.local/bin/arch-notebook-install-automation`
- Case MIME definition: `/home/sysadmin/.local/share/mime/packages/arch-notebook-case.xml`

## Acceptance results

| Gate | Result | Evidence |
| --- | --- | --- |
| User installation | PASS | `scripts/install-linux-user.sh` installed the release executable, automation installer, menu entry, icons, and MIME package from `/tmp/arch-notebook-local-install-proximity-6b59/arch-notebook-linux-x86_64`. |
| Desktop shortcut and menu entry | PASS | The menu entry and desktop shortcut are byte-identical; both pass `desktop-file-validate`. The entry uses the installed executable and `Icon=arch-notebook`. |
| Installed binary | PASS | Installed binary SHA-256 matches the release artifact above; `--version --json` returns version `0.1.0`. |
| Icon synchronization | PASS | Installed SVG SHA-256 `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8` and PNG SHA-256 `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88` match the repository assets. |
| Automation installer | PASS | Installed launcher helper is byte-identical to `scripts/install-linux-automation.sh`. |
| File association | PASS | `xdg-mime query default application/vnd.arch-notebook.case+json` returns `arch-notebook.desktop`. |
| Native launch and rendered window | BLOCKED | The current session's Wayland connection returned Winit `NoCompositor`; X11 is inaccessible. Install metadata and file hashes pass, but no interactive or visual launch is claimed. |

The install leaves the existing local icon art in place because the current repository icon hashes already match the installed SVG and PNG. No case or SQLite source was opened during installation.
