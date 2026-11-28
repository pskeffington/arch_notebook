# LOCAL-INSTALL-11 — current Rust release and launcher synchronization

- **Task / owner:** Install the current Arch Notebook release and synchronize this user's launcher, icon artwork, case association, and SQL scheduling helper; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source:** `6091dad7bb646cd3321f2bf91dcfc6774e302b36`.
- **Release binary SHA-256:** `f5ad767d85b6c4d4384a199f689cb896b2b414105c5bf8a8a2f1b2264bb39dd1`.
- **Host:** Linux x86_64; current-user install only.

## Acceptance results

| Gate | Result | Evidence |
| --- | --- | --- |
| Optimized release build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --bin arch-notebook --locked --offline` finished successfully in 1m54s. |
| Isolated installer acceptance | **PASS** | `python3 scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report /tmp/arch-notebook-install-acceptance-6091dad.json` exited 0. Desktop entry validation, menu/shortcut equality, installed binary, SVG/PNG icons, SQL automation helper, and MIME definition all passed. |
| Current-user installation | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-linux-x86_64` exited 0. Installed `/home/sysadmin/.local/bin/arch-notebook`, `/home/sysadmin/.local/bin/arch-notebook-install-automation`, menu entry, `/home/sysadmin/Desktop/Arch Notebook.desktop`, hicolor SVG/PNG, and case MIME definition. |
| Installed release and launcher checks | **PASS** | Installed binary hash matches the release hash above. `--version --json` returned version `0.1.0`; `desktop-file-validate` exited 0 and the menu entry matches the Desktop shortcut byte-for-byte. |
| Icon/theme database refresh | **PARTIAL** | `kbuildsycoca6 --noincremental` exited 0. `gtk-update-icon-cache -f /home/sysadmin/.local/share/icons/hicolor` could not update a cache because that user icon directory has no `index.theme`; the SVG and PNG files themselves match the package assets by SHA-256. |
| Clinton corpus SQL acceptance | **PASS, bounded** | Release binary acceptance exited 0: 6,173 relationship rows, zero skipped rows or deltas, 2,813 entities, and unchanged PDF/case/database inputs. Aggregate report: [CLINTON-CORPUS-QA-04-release.json](../sql-automation/CLINTON-CORPUS-QA-04-release.json). Qwen analysis, human content review, and PDF extraction completeness were not established. |
| Native topology visual/input review | **NOT RUN** | The release is installed, but no clean app screenshot or native pointer interaction of the ranked-entity topology was captured. |

## Remote build

The `build` workflow completed successfully for repaired source commit `cbad7eca7ffacd3d3f3a2d785abfdd9423278a95`, run [46108181993](https://github.com/pskeffington/arch_notebook/actions/runs/46108181993). Linux tests/fixture determinism, Linux x86_64, Linux ARM64, macOS Intel, macOS Apple Silicon, and Arch Linux packaging all passed. Release-asset attachment was skipped because this was a manual workflow dispatch, not a published release.

Two earlier manual runs failed at formatting: [46107715406](https://github.com/pskeffington/arch_notebook/actions/runs/46107715406) and [46107908092](https://github.com/pskeffington/arch_notebook/actions/runs/46107908092). Investigation showed that a GitHub API upload had truncated `src/main.rs`, inserting literal ellipses into the remote Rust source. The source was reconstructed from checked local chunks and uploaded only after its Git blob SHA matched local `6f731733243a96c7f6335bb004ad10500f7e1b17`; GitHub reports the same blob SHA and exact size 2,375,707 bytes. The repaired tree SHA `1a18c871b9b2adb4d7023e1833c7d007e1a6daea` matches the local product/evidence tree at verification. No force-push was used.
