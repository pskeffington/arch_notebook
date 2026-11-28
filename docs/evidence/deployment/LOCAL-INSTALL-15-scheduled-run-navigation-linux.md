# LOCAL-INSTALL-15 · scheduled SQL run navigation — 2026-11-28

**Task / owner:** Install the verified Arch Notebook candidate into the current user's Linux launcher and synchronize its deployment graphics.

**Repository / branch:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions`.
**Product source commit:** `c653f3141b224a10dee721542965cc1a42b30866`. Later branch commit `f8529d90ebe052ceb6ecaf2522d457986577e69a` changes documentation only.
**Host:** Linux x86_64; user `sysadmin`.
**Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `46596680af31ba1d92314426e6fd69feb7179a9d8489038cbcca1ee48ac42c1a`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Current-user installer | PASS | Staged current release and repository assets under `/tmp/arch-notebook-linux-x86_64`; `scripts/install-linux-user.sh /tmp/arch-notebook-linux-x86_64` exited 0. |
| Installed binary | PASS | `/home/sysadmin/.local/bin/arch-notebook` SHA-256 equals the release binary; `--version --json` reports `arch-notebook` 0.1.0. |
| Menu and Desktop launchers | PASS | `/home/sysadmin/.local/share/applications/arch-notebook.desktop` and `/home/sysadmin/Desktop/Arch Notebook.desktop` compare byte-for-byte; both pass `desktop-file-validate`. |
| Deployment graphics | PASS | Installed scalable SVG SHA-256 `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8`; installed 1024×1024 PNG SHA-256 `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`. Both match the repository assets. |
| Case-file MIME handler | PASS | Installed XML SHA-256 `c5086e63e8cdbf0b8e3a43e359227bbec1e5936c7079abe0c9bad65d52e108db` matches `packaging/arch/arch-notebook-case.xml`. |
| SQL automation helper | PASS | Installed helper SHA-256 `d87461c6001c7840b193cfc3672fc6c5fdac41827f0b0c70f8fbaa15dd6f28a2` matches `scripts/install-linux-automation.sh`. |
| Native window/pointer review | NOT TESTED | Installer and CLI version checks do not establish visible launch, pointer interaction, or IBM visual parity. |

The application menu entry, Desktop shortcut, case MIME registration, automation helper, and SVG/PNG graphics now point to the same current user installation. This is a local user install, not a public release.
