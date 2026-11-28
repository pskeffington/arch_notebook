# Current-user Linux install — 2026-11-27

**Task / owner:** Synchronize the current Arch Notebook executable, desktop launcher, and icons on this machine; implementation session.
**Repository / branch / source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `57a714e7fe1c909e1eeb10b66283262c140fbcb3`.
**Installed release binary SHA-256:** `b3456f71520b084234b281a9056cea74b94aa1d9da7527ac3de4a125d79f574a`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Locked offline release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` completed successfully. |
| Linux user installer | PASS | `scripts/install-linux-user.sh /tmp/arch-notebook-install-20261128-01/arch-notebook-linux-x86_64` installed the executable, automation helper, menu entry, Desktop shortcut, SVG/PNG icons, and MIME definition. |
| Installed executable provenance | PASS | Installed executable SHA-256 equals the release binary above; `arch-notebook --version --json` reports version `0.1.0`. |
| Desktop entry and shortcut | PASS | Menu and Desktop entries are byte-identical and both pass `desktop-file-validate`. |
| Graphics and support assets | PASS | Installed scalable SVG and 1024×1024 PNG match repository assets; installed MIME XML and SQL automation installer match repository files. |
| Cross-platform CI | PASS | Exact-head workflow run [46095125130](https://github.com/pskeffington/arch_notebook/actions/runs/46095125130) passed all six Linux test, Linux x86_64/ARM64, macOS Intel/Apple Silicon, and Arch packaging jobs. |
| Native visual launch/review | NOT TESTED | The host reports X11 display `:0`, but `xwininfo -root -tree` cannot open it. Launcher synchronization does not establish visible rendering or pointer interaction. |

Build artifacts were staged under `/tmp/arch-notebook-install-20261128-01`; installation writes were limited to the current user's application paths. The retained Clinton case and IBM ANX schema were not present in the local workspace/share during this session, so neither was part of this install check.
