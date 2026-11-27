# LOCAL-INSTALL-08 · synchronized current-user desktop shortcut

- **Task / owner:** Install the current Arch Notebook candidate and keep its Desktop launcher aligned with the application menu; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Installer source candidate:** `1dd1c76b986701fc989b114f306e9dab745c860f`.
- **Installed application source candidate:** `f43da45eed456ffd1aa955c1393e9e553a574502`.
- **Host:** Linux x86_64; current user only.
- **Release and installed executable SHA-256:** `7f789d8fa53fb2fac444e2189767ffd266e3f49c537b2058d6175356fdf176c8`.
- **Isolated installer report:** [LOCAL-INSTALL-08 JSON](artifacts/LOCAL-INSTALL-08-linux.json).

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
| Current-user installer | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-sync-chart-template-20261127/arch-notebook-linux-x86_64` exited 0 and installed the release into the paths listed above. |
| Current-user launcher and Desktop shortcut | **PASS** | Both desktop files have SHA-256 `0997476bbec189ce6c3a98c22670cd4b8aebd989c721bcced4bf6eff5c141f4c`; `cmp` and `desktop-file-validate` exited 0. Shortcut mode is 0755. |
| Binary, graphics, helper and MIME association | **PASS** | Installed executable matches release hash above. Installed SVG hash is `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8`; PNG is `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`; automation helper is `69e4b4b1843b85cce0b63b2e64670768a322006f7d04249550aabf3b328bb9ce`. `gio mime application/vnd.arch-notebook.case+json` resolves to `arch-notebook.desktop`. |
| Exact-head cross-platform CI | **PASS** | Workflow run [45985015963](https://github.com/pskeffington/arch_notebook/actions/runs/45985015963) completed successfully on `fa9002a2bd86425ef9b61179ab6ab1747e1e3106`; the Linux test job ran the isolated install smoke, and all six jobs passed. |
| Native application launch and interaction | **BLOCKED** | This execution sandbox cannot connect to display `:0`; prior Winit launch attempts report `WaylandError(Connection(NoCompositor))`. Install metadata and icon synchronization pass, but this report does not claim that the current binary opened a native window. |

The updated installer uses the configured XDG Desktop directory when available and falls back to `$HOME/Desktop`. No case or SQLite source was opened.
