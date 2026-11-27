# G4 dense-label Auto mode — current topology render

- **Task / owner:** SNG-DELIVERY-002 G4 graph-render follow-up; implementation session.
- **Repository / branch / source:** `arch_notebook` / `codex/ui-00-chart-baseline` / `a9f56cced2ec12c2f0f787520942939a7603b006`.
- **Fixture:** generated fictional `VECTOR UI SPARSE 30`, 30 entities and 90 relationships; SHA-256 `bf88b5df8c2c8e259e41a52486d8f09a12f6a6665d3a60892c802d8c06fa872d`.
- **Candidate:** current-source Linux debug application, SHA-256 `74c900167f450e28f391506d043bfb17fe177382afe19d6f8fdd16bcab24e606`.
- **Capture:** [current topology render](artifacts/G4-DENSE-LABEL-03-current-topology-render-20261127.png), 1440×900, SHA-256 `ae2381cbfbc0b949df8614bd14a1f2b9f6a60ff849c5d0c6f493bade6365876e`.

## Result

- **PASS — current native render:** the current-source chart window opened and rendered the supplied fictional case on the local X11 desktop.
- **PASS — initial chart framing:** all 30 entity cards are visible inside the chart viewport; the left-edge clipping recorded on the earlier candidate was not observed. The chart navigator and analysis sidebar also render.
- **PASS — dense Auto labels:** the screenshot shows relationship routes while dense relationship text labels are suppressed.
- **NOT TESTED — interaction and visual parity:** no native pointer or keyboard actions were sent. This capture does not establish IBM Analyst's Notebook parity, Fit All interaction, chart switching, Arrange/Restore, or current Clinton-corpus visual quality.
- **NOT MEASURED — renderer performance:** no frame rate, GPU, compositor, or input-to-display timing was collected. Fixture is synthetic and the candidate is a debug build.

## Reproduction

Build from the repository root:

```sh
CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-current-target cargo build --locked --offline
```

Generate the fixture:

```sh
python3 scripts/generate-chart-fixtures.py \
  --family sparse --nodes 30 \
  --output /tmp/arch-notebook-current-chart.case.json
```

Launch with `WAYLAND_DISPLAY` unset and capture the active window with:

```sh
env -u WAYLAND_DISPLAY DISPLAY=:0 \
  XDG_DATA_HOME=/tmp/arch-notebook-current-ui-xdg \
  /tmp/arch-notebook-current-target/debug/arch-notebook \
  /tmp/arch-notebook-current-chart.case.json
spectacle --new-instance --activewindow --background --nonotify \
  --no-decoration \
  --output /tmp/arch-notebook-current-topology-render.png
```

The source commit includes the chart topology lifecycle regression. Its offline Rust suite passed with 69 tests, 0 failures, and 1 opt-in benchmark ignored. Formatting and `git diff --check` also passed. See the current synchronized branch entry in [build acceptance](../../build-acceptance.md).
