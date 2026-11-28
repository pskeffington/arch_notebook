# LOCAL-INSTALL-13 · Quick open release on Linux — 2026-11-28

**Task / owner:** Install the verified Quick open case-action release in the current user's desktop and launcher / orchestrator.

**Product repository / branch:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions`.
**Local source commit / tree:** `5b65bfe38691e501997ca3fd721492717282f408` / `5fdca9dc1a3508236db5dd9cf2eac0e35edef925`.
**Remote equivalent commit / tree:** `b446d9249b73200ef706827ad9a137ccf66f3a26` / `5fdca9dc1a3508236db5dd9cf2eac0e35edef925`.
**Host:** Linux x86_64; user `sysadmin`.
**Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `8381bd57bdf84017377d92ef62916bde737e05ebe15dbca4b98317ec4b62ed27`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Rust formatting and offline checks | PASS | `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, and `git diff --check` exited 0. |
| Full Rust workspace tests | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline`: 81 passed, 0 failed, 3 ignored. Headless UI regressions cover Open case, New case, unsaved-edit confirmation, and Enter with an empty search retaining Chart as the default. |
| Optimized release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0 in 2m58s. |
| Isolated Linux installer acceptance | PASS | `python3 scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report /tmp/arch-notebook-user-install-quick-open.json` exited 0. The isolated report confirms binary, menu and Desktop entries, icons, automation helper, MIME file, and version metadata. |
| Current-user installation | PASS | `scripts/install-linux-user.sh /tmp/arch-notebook-current-install-984al_dd/arch-notebook-linux-x86_64` exited 0 and refreshed the executable, menu launcher, Desktop launcher, icons, MIME definition, and SQL automation installer. |
| Installed binary and launcher graphics | PASS | Installed executable SHA-256 equals the release binary; `--version --json` reports Arch Notebook 0.1.0. Menu and Desktop entries are identical and both pass `desktop-file-validate`. SVG and PNG hashes match repository icons. SQL helper and MIME hashes also match their repository sources. |
| Native rendering and pointer/keyboard interaction | NOT RUN | Display access is unavailable in this session. Headless acceptance does not establish live visual or input behavior. |
| Remote CI and IBM compatibility | NOT RUN | The feature branch has no associated check runs. No IBM XSD or Analyst's Notebook application validation was performed. |

The installation is local to this user's home directories. No public release was created.
