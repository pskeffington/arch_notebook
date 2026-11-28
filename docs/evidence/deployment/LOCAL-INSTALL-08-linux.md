# LOCAL-INSTALL-08 · synchronized current-user desktop shortcut

- **Task / owner:** Install the current Arch Notebook candidate and keep its Desktop launcher aligned with the application menu; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Installer source candidate:** current `scripts/install-linux-user.sh` at `30b8020`.
- **Installed application source candidate:** `30b8020` (`Restore topology line hover preview`), on top of the route-culling and viewport-rendering changes.
- **Host:** Linux x86_64; current user only.
- **Release and installed executable SHA-256:** `330bc4fe2446bf8c7bff685ac4e56ddc5ffabc7b82cb59199b2cf537bd0d09b0`.
- **Installed SVG icon SHA-256:** `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8`.
- **Installed 1024px PNG icon SHA-256:** `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`.
- **Isolated installer report:** [LOCAL-INSTALL-08 JSON](artifacts/LOCAL-INSTALL-08-linux.json), SHA-256 `2e38aebf8dfd745effa5b02980494bb2656264ccc1eb10596189eda171846c58`.

## Installed paths

- Executable: `/home/sysadmin/.local/bin/arch-notebook`
- Application-menu entry: `/home/sysadmin/.local/share/applications/arch-notebook.desktop`
- Desktop shortcut: `/home/sysadmin/Desktop/Arch Notebook.desktop`
- SVG icon: `/home/sysadmin/.local/share/icons/hicolor/scalable/apps/arch-notebook.svg`
- 1024px PNG icon: `/home/sysadmin/.local/share/icons/hicolor/1024x1024/apps/arch-notebook.png`
- SQL automation installer: `/home/sysadmin/.local/bin/arch-notebook-install-automation`

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Isolated installer smoke | **PASS** | `python3 scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report docs/evidence/deployment/artifacts/LOCAL-INSTALL-08-linux.json` exited 0 in a temporary HOME. Menu and Desktop files match byte-for-byte, the shortcut mode is 0755, the installed executable matches its input, both icon assets and automation helper match package sources, MIME definition matches, and `--version --json` works. `desktop-file-validate` passed. |
| Current-user installer | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-linux-x86_64` exited 0 and installed this release into the paths listed above. |
| Current-user launcher and Desktop shortcut | **PASS** | Installer refreshed the current menu and Desktop entries from the matching package files; `cmp` confirms both files match byte-for-byte and both pass `desktop-file-validate`. The shortcut mode is 0755. |
| Binary, graphics, helper and MIME association | **PASS** | Installed executable matches release hash above; SVG and PNG artwork, automation helper, and MIME association were refreshed from repository sources. `--version --json` exited 0. |
| Exact-head cross-platform CI | **PENDING** | Branch pushes do not trigger `.github/workflows/build.yml`; it runs on `main` pushes, pull requests, and manual dispatch. No pull request or manual run was created for this candidate. |
| Native application launch and render | **PARTIAL; visual acceptance BLOCKED** | The installed executable was invoked with `--open-case` for the retained Clinton case and returned without a startup error. The X11 window inventory contains several same-titled Arch Notebook windows, so it does not identify the current window reliably. `spectacle` aborted and ImageMagick `import` could not open the display from this session; no screenshot or native pointer review is claimed. |

The updated installer uses the configured XDG Desktop directory when available and falls back to `$HOME/Desktop`. The current renderer restores contact-line hover highlighting and a fixed-height topology preview while retaining node-click priority; see [PROXIMITY-MAP-HOVER-02](../vector-ui/PROXIMITY-MAP-HOVER-02-linux.md). The native window still needs a Clinton-case visual pass and Fit all interaction check. No IBM compatibility claim is made.
