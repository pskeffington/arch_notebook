# G4 dense-label Auto mode — native render review

- **Task / owner:** SNG-DELIVERY-002 G4 visual acceptance; `/root`.
- **Source commit:** `94f6e9dadf8243db4a298067f02554125e1b38a8` on `codex/ui-00-chart-baseline`.
- **Release executable SHA-256:** `fe94ef08e1e9e1583551a535f8dec74614931fd78736658edaeddb002d08f4e1` (`/tmp/arch-notebook-target/release/arch-notebook`).
- **Fixture:** generated with `python3 scripts/generate-chart-fixtures.py --family sparse --nodes 30 --output /tmp/arch-notebook-review-current.case.json`; 30 fictional entities and 90 relationship records; SHA-256 `bf88b5df8c2c8e259e41a52486d8f09a12f6a6665d3a60892c802d8c06fa872d`.
- **Screenshot:** [isolated native chart window](artifacts/G4-DENSE-LABEL-02-native-render-20261127.png), SHA-256 `728002050568d7d6c75df6117fde593cd71b79dc5c83662b934818450470dde7`.
- **Environment:** Linux X11/XWayland desktop at 2560×1440; app launched from the release executable with `WAYLAND_DISPLAY` unset, a temporary `XDG_DATA_HOME`, and the generated fixture. The screenshot shows the 1440×900 chart window plus decorations.

## Observations

- **PASS — native render, Auto label behavior:** relationship routes remain drawn while relationship text labels and attribute callouts are suppressed at the dense overview. The screenshot does not establish an exact visible route count. This matches the documented Auto behavior when more than 36 routes are visible.
- **PASS — no card-to-card overlap observed:** all 30 cards are present in the chart; no collision was observed among their visible portions. At least two cards are clipped by the left viewport edge, so not all names are fully visible.
- **FAIL — initial viewport framing:** at least two cards on the far left are clipped by the chart viewport. This launch did not invoke **Fit all entities**, so it does not establish whether the command corrects the saved/default view.
- **NOT TESTED — interaction:** no pointer or keyboard actions were sent; relationship selection, Auto/All switching, Fit all, Arrange/Restore, resize behavior, and culling of crossing routes were not tested in this run.
- **NOT MEASURED — native performance:** no GPU, compositor, presentation, frame-rate, or input-to-display measurements were collected.

The screenshot is a direct view of the isolated candidate release, not a capture of the operator's existing app process. The isolated process exited through the 15-second timeout after capture (exit 124). No case or application data was saved. This observation upgrades the dense-label gate from headless-only to native-render evidence for Auto's overview behavior; it does not close G3/G4 interaction acceptance.

## Reproduction

```sh
python3 scripts/generate-chart-fixtures.py \
  --family sparse --nodes 30 \
  --output /tmp/arch-notebook-review-current.case.json
timeout 15s env -u WAYLAND_DISPLAY DISPLAY=:0 \
  XDG_DATA_HOME=/tmp/arch-notebook-review-xdg \
  /tmp/arch-notebook-target/release/arch-notebook \
  /tmp/arch-notebook-review-current.case.json
```

Capture was made with `spectacle -b -n -o /tmp/arch-notebook-review-current-live.png`; the app-only image was cropped from that desktop capture for evidence. Exit status 124 is the expected timeout after the render observation, not an application failure.
