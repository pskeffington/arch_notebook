# LOCAL-INSTALL-01: Linux user install and launcher sync

- **Result:** PASS for release build, user install, launcher registration, and icon synchronization; GUI relaunch remains unverified.
- **Owner:** Arch Notebook implementation session.
- **Branch/source revision:** `codex/ui-00-chart-baseline` at `a832a9bcd10a8fd62d6680684c6c36ef85f64b91`.
- **Rust source SHA-256:** `a572a828f4fe705c5ad3091399c0fd1fb8d0e01aefec27de46b858c2a0e86aef`.
- **Release binary SHA-256:** `0614576294efd356f606081dc437d22672e0e0a48b4612f86b436a3e32f694b9`.
- **Previous installed binary SHA-256:** `178554daa2fa799da921c51551e8d46796296a1ca5681237819f8002d27e0058`.
- **Automation installer SHA-256:** `69e4b4b1843b85cce0b63b2e64670768a322006f7d04249550aabf3b328bb9ce` (repository and installed copy match).
- **Installed application:** `/home/sysadmin/.local/bin/arch-notebook`.
- **Desktop entries:** `/home/sysadmin/.local/share/applications/arch-notebook.desktop` and `/home/sysadmin/Desktop/Arch Notebook.desktop`.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS, exit 0 | Finished optimized release build for the current branch source. |
| `./scripts/install-linux-user.sh /tmp/arch-notebook-local-install-20261126a/arch-notebook-linux-x86_64` | PASS, exit 0 | Installed the executable, automation installer, menu entry, icon assets, and case-file MIME definition for the current user. |
| Installed release binary digest | PASS | Installed `/home/sysadmin/.local/bin/arch-notebook` hash equals the release binary hash above. |
| Installed automation installer | PASS | `/home/sysadmin/.local/bin/arch-notebook-install-automation` is byte-identical to `scripts/install-linux-automation.sh`. |
| Installed icon digests | PASS | SVG `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8` and PNG `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88` match the repository assets byte-for-byte. |
| Desktop entries | PASS | The desktop menu entry and `~/Desktop/Arch Notebook.desktop` are byte-identical; both use `Icon=arch-notebook` and the installed executable path. |
| `desktop-file-validate /home/sysadmin/.local/share/applications/arch-notebook.desktop` and `desktop-file-validate '/home/sysadmin/Desktop/Arch Notebook.desktop'` | PASS, exit 0 | Both entries validate. |
| `xdg-mime query default application/vnd.arch-notebook.case+json` | PASS | Returns `arch-notebook.desktop`. |
| `/home/sysadmin/.local/bin/arch-notebook --version --json` | PASS, exit 0 | `{"name":"arch-notebook","schema_version":1,"version":"0.1.0"}`. |
| Native GUI launch | NOT VERIFIED | Wayland protocol access was denied in the current session; this install check did not claim a rendered or interactive launch. |

The documented user installer was run without `sudo`. Its staging files were under `/tmp/arch-notebook-local-install-20261126a`. No case file or SQLite source was opened or modified during installation. The installer also refreshed the user MIME database.

## Handoff

The local menu entry, desktop shortcut, registered file association, installed executable, and installed icon art now point to the same current Arch Notebook build. Verify the GUI launch and interaction when a stable compositor session is available. This local installation does not change the GitHub release state and does not close the UI-00 native acceptance gate.
