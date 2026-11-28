# LOCAL-INSTALL-14 · Desktop XML/XSLT workflow release — 2026-11-28

**Task / owner:** Install the verified XML/XSLT desktop import workflow and synchronize its launcher assets / orchestrator.

**Product repository / branch:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions`.
**Source commit:** `9293242198483edb977b7e0d49e778cfa149ad85`.
**Host:** Linux x86_64; user `sysadmin`.
**Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `4f37a78a91230b8c1aeafc16d0d3aedc18638dd83bb56eaf2172d504cf24c08d`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Optimized release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0. |
| Isolated Linux user-install acceptance | PASS | `python3 scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report /tmp/arch-notebook-user-install-xslt-desktop.json` exited 0. Binary, menu/Desktop entries, graphics, helper, MIME definition, and version checks passed. |
| Current-user installation | PASS | `scripts/install-linux-user.sh /tmp/arch-notebook-current-install-xslt-desktop/arch-notebook-linux-x86_64` exited 0. |
| Installed executable/version | PASS | `/home/sysadmin/.local/bin/arch-notebook` matches release SHA-256 and `--version --json` reports Arch Notebook 0.1.0. |
| Launcher and icon synchronization | PASS | Menu and Desktop entries compare identically and both pass `desktop-file-validate`; installed SVG and PNG hashes equal `assets/icons/arch-notebook.svg` and `.png`. |
| Automation helper and MIME definition | PASS | Installed helper hash `d87461c6001c7840b193cfc3672fc6c5fdac41827f0b0c70f8fbaa15dd6f28a2` matches the repository script; installed MIME definition matches `packaging/arch/arch-notebook-case.xml`. |
| Native XML/XSLT menu interaction | NOT TESTED | User installation checks do not exercise desktop pointer or keyboard input. |
| IBM schema/application interoperability | NOT TESTED | No IBM XSD or application was available for this install verification. |

The desktop Import menu, launcher, icons, MIME association, and SQL automation helper are synchronized to this release. The installation is local to this user's home directories; no public release was created.
