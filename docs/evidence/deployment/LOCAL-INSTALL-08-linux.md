# LOCAL-INSTALL-08 · synchronized current-user desktop shortcut

- **Task / owner:** Install the current Arch Notebook candidate and keep its Desktop launcher aligned with the application menu; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Installer source candidate:** `1dd1c76b986701fc989b114f306e9dab745c860f`.
- **Installed application source candidate:** topology-rendering changes in the current worktree, based on `94f6e9dadf8243db4a298067f02554125e1b38a8`.
- **Host:** Linux x86_64; current user only.
- **Release and installed executable SHA-256:** `7a7f9ebc6111185c80aeed1b98746b93127738e7eea588567b29bb8a437c3924`.
- **Isolated installer report:** [LOCAL-INSTALL-08 JSON](artifacts/LOCAL-INSTALL-08-linux.json), SHA-256 `1634f25547e2d90e4e2ed0eea943f7cdf9006303bcd0cb1c32c06df8fcdb10ad`.

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
| Current-user launcher and Desktop shortcut | **PASS** | Installer refreshed the current menu and Desktop entries from the matching package files; the shortcut mode is 0755. |
| Binary, graphics, helper and MIME association | **PASS** | Installed executable matches release hash above; SVG and PNG artwork, automation helper, and MIME association were refreshed from repository sources. `--version --json` exited 0. |
| Exact-head cross-platform CI | **PENDING** | Previous candidate CI run [45988117371](https://github.com/pskeffington/arch_notebook/actions/runs/45988117371) passed on `94f6e9dadf8243db4a298067f02554125e1b38a8`; it does not include the current topology changes. |
| Native application launch and render | **PARTIAL, prior candidate only** | The earlier release from `94f6e9d` opened on X11 and rendered the fictional 30-entity/90-record sparse fixture; see [G4-DENSE-LABEL-02](../vector-ui/G4-DENSE-LABEL-02-native-render-20261127.md). That view showed Auto label suppression but clipped cards at the left edge. The current topology candidate has not received a native screenshot; pointer/keyboard interaction and the real Clinton case's native view remain untested. |

The updated installer uses the configured XDG Desktop directory when available and falls back to `$HOME/Desktop`. The new topology change maps anomalous saved entity coordinates across the viewport without changing stored positions; real-corpus aggregate CLI geometry passed separately in [CLINTON-CORPUS-QA-01](../sql-automation/CLINTON-CORPUS-QA-01-linux.md). The native window still needs a Clinton-case visual pass and Fit all interaction check. The current release opened the user's real case only through read-only CLI QA; it did not save any case or SQLite change.
