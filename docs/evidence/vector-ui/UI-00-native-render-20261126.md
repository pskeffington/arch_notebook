# UI-00: Native Linux render-only recheck

- **Result:** PASS for native window launch/render and visible chart/navigation shell; pointer/keyboard interaction remains BLOCKED.
- **Task / owner:** Continue native desktop acceptance for Arch Notebook; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source:** `b637d965ae5ed4aeea9ec4300860523dfb95b71b`.
- **Installed binary SHA-256:** `6c801cf7f307c56a7c4060e48fbeb19f89ba54f9a9f4d19fdf4ff966dd542ae9`.
- **Fixture:** `fixtures/chart/correctness.case.json`, fictional UI-00 chart.
- **Screenshot:** [UI-00-native-window-20261126.png](UI-00-native-window-20261126.png), SHA-256 `8396c9afdd7c42d7489941949201431fa9654aa9f12534f63fb68eb7e51ceb24`.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| `env -u WAYLAND_DISPLAY /home/sysadmin/.local/bin/arch-notebook /home/sysadmin/github/arch_notebook/fixtures/chart/correctness.case.json` | PASS for launch; process remained live at capture | The native window title reads `Arch Notebook - Link Analysis`; the case title reads `VECTOR UI CORRECTNESS FIXTURE`. The app process had not exited when the screenshot was captured. |
| `spectacle --new-instance --activewindow --background --nonotify --no-decoration --output /tmp/arch-notebook-app-window.png` | PASS, exit 0 | Captured the Arch Notebook window only at 1440×900. The chart, navigator, analysis sidebar, and geodesic-layer explanation are visible. Screenshot SHA-256 is recorded above. |
| `cargo fmt --all -- --check && git diff --check && CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` on `e07bfc7` | PASS, exit 0 | 23 tests passed, 0 failed, 0 ignored. Includes successful validated ANX background-worker parsing with a fictional chart payload. |
| Native pointer/keyboard interaction sequence | BLOCKED | Input-event injection is unavailable in the current sandbox. No chart selection, geodesic computation, layout gesture, or ANX dialog interaction was performed. |
| Display service via `wayland-info` / X11 event injection | BLOCKED | `wayland-info` and `xdotool` cannot connect through this sandbox, even though Spectacle's capture integration can retrieve the host window image. |

The image confirms rendering only. It does not establish responsiveness under repeated interaction, visual parity with IBM Analyst's Notebook, or completion of UI-00. The case is the committed fictional correctness fixture; no operational case or SQL source was opened.
