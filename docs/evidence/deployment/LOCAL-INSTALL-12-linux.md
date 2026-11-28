# LOCAL-INSTALL-12 · current Rust release installed on Linux

- **Task / owner:** Synchronize the authorized per-user Arch Notebook installation with the verified current Rust release / orchestrator.
- **Product source:** `pskeffington/arch_notebook` branch `codex/ui-00-chart-nested-conditions`, product code at `5bb459309785a241e36aeffe47df68eafa6f4a1c`. Subsequent commits through `c324985` changed documentation only.
- **Host:** Linux x86_64; user `sysadmin`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `d5fcb5b6bba6f6b53f1bddf3bca3105a0ae638555cb00931080bcd2131c706c3`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Isolated installer acceptance | PASS | `python3 scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report /tmp/arch-notebook-user-install-current.json` exited 0. The installed executable matched the release, menu/Desktop entries matched and validated, SVG/PNG, SQL helper, and MIME file matched the package. |
| Per-user install | PASS | Staged release files in `/tmp/arch-notebook-install-root.hG09NB/arch-notebook-linux-x86_64`; `scripts/install-linux-user.sh` exited 0 and refreshed the user executable, application menu entry, Desktop shortcut, icons, case MIME handler, and SQL automation installer. |
| Installed executable | PASS | `/home/sysadmin/.local/bin/arch-notebook` SHA-256 matches the release binary: `d5fcb5b6bba6f6b53f1bddf3bca3105a0ae638555cb00931080bcd2131c706c3`; `--version` reports `Arch Notebook 0.1.0`. |
| Launcher and graphics synchronization | PASS | Application menu and `/home/sysadmin/Desktop/Arch Notebook.desktop` are byte-identical; both passed `desktop-file-validate`. Installed SVG hash: `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8`; PNG hash: `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`; both match repository assets. |
| Automation helper / MIME | PASS | Installed SQL automation helper hash `a04d9a7cf11746058b4862e5616f63c165669deac3ef8aba17edc79dc617dbf8` matches source; installed MIME XML hash `c5086e63e8cdbf0b8e3a43e359227bbec1e5936c7079abe0c9bad65d52e108db` matches package. |
| Native visual and pointer review | NOT RUN | Installation and file validation do not establish launch, rendering, keyboard/pointer behavior, or SQL workspace usability. |
| Public release / IBM compatibility | NOT CLAIMED | This is a local current-user installation. No public GitHub Release, IBM XSD validation, or IBM application import was performed. |

No Arch Notebook process was running when the replacement began. The installer used atomic temporary files and replaced only this user's application files. Isolated acceptance report: `/tmp/arch-notebook-user-install-current.json`.
