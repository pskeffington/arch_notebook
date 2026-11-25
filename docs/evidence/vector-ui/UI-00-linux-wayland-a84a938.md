# UI-00 Linux Wayland runtime evidence

- **Task / owner:** Record a native release-binary smoke run for UI-00; Codex, Linux Operations.
- **Repository / branch:** `pskeffington/arch_notebook`, `codex/ui-00-chart-baseline`.
- **Source commit:** `a84a938424cc689af4bda8f5ab9acd60acdfff7d` (documentation-only commits after code candidate `1e5a86f1cf0ee34c9cda7adca5b64c6bac6827d4`).
- **Binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `460ad736650ca8288279cf2a0f36ddc28a6436340c160a588dc5a465cbc6ae8b`.
- **Fixture:** `/tmp/arch-notebook-ui00/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`; 8 entities, 8 relationship records, 5 weak components, 2 parallel groups, 1 self-loop.
- **Lockfile:** `Cargo.lock` SHA-256 `4e14e9b55f812bfb1af0d7687039d4bc266a76f4a6f584dee8d7caa1349c01a3`.
- **Host:** Linux x86_64, kernel `7.3.5-zen1-1-zen`; `rustc 1.100.0`, `cargo 1.100.0`; session compositor reports `wayland-1`. Renderer is the repository's locked egui/eframe 0.31.1 Glow build.

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| Formatting | PASS | `cargo fmt --all -- --check`, exit 0. |
| Rust tests | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`, exit 0; 12 passed, 0 failed. |
| Offline check and debug build | PASS | Both locked offline commands exited 0. |
| Release build | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline`, exit 0; optimized build finished in 1m 42s. |
| X11 desktop launch | FAIL | `--open-case` exited 1 with `XOpenDisplayFailed`. |
| Wayland release-binary launch and chart rendering path | PASS (bounded smoke) | The app stayed alive for the 15-second timeout with the isolated fixture and emitted chart diagnostics before the timeout stopped it. This confirms runtime execution, not complete interaction acceptance. |
| Native chart CPU samples | PASS (limited sample) | Two `chart_prepare` samples: 111,014 ns and 54,803 ns. Two `navigator_cpu_submit` samples: 5,740 ns and 5,178 ns. No `chart_pick` sample was emitted because no input action was delivered. These are CPU spans, not frame intervals or FPS; two samples are insufficient for percentile/performance claims. |
| Documented pan/zoom/drag/selection/search/navigator/layout sequence | BLOCKED | No Wayland input-injection utility is installed; `xdotool` cannot connect to this Wayland session. The required interaction trace was not performed. |
| Window screenshot and visual inspection | BLOCKED | `spectacle` aborted during an attempted capture; no screenshot of the test window was inspected or retained. |
| UI-00 native baseline gate / UI-01 prerequisite | BLOCKED | Runtime launch is now demonstrated, but the protocol's interaction sequence, visual inspection and representative repeated workload samples remain outstanding. Do not begin UI-01 on this evidence alone. |

## Commands and observed outcomes

```text
cargo fmt --all -- --check
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline
exit 0 (12 passed, 0 failed)

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline
exit 0 (release profile finished in 1m 42s)

timeout 30s env -u WAYLAND_DISPLAY HOME=/tmp/arch-notebook-ui00/home XDG_DATA_HOME=/tmp/arch-notebook-ui00/xdg-data XDG_CONFIG_HOME=/tmp/arch-notebook-ui00/xdg-config ARCH_NOTEBOOK_CHART_DIAGNOSTICS=/tmp/arch-notebook-ui00/chart-diagnostics.jsonl /tmp/arch-notebook-target/release/arch-notebook --open-case /tmp/arch-notebook-ui00/correctness.case.json
exit 1 (XOpenDisplayFailed)

timeout 15s env DISPLAY= WAYLAND_DISPLAY=wayland-1 XDG_RUNTIME_DIR=/run/user/1000 HOME=/tmp/arch-notebook-ui00/home XDG_DATA_HOME=/tmp/arch-notebook-ui00/xdg-data XDG_CONFIG_HOME=/tmp/arch-notebook-ui00/xdg-config ARCH_NOTEBOOK_CHART_DIAGNOSTICS=/tmp/arch-notebook-ui00/chart-diagnostics.jsonl /tmp/arch-notebook-target/release/arch-notebook --open-case /tmp/arch-notebook-ui00/correctness.case.json
exit 124 (expected timeout after the app remained active for 15 seconds)

spectacle -b -n -o /tmp/arch-notebook-ui00/desktop.png
aborted; no test-window screenshot was retained
```

The diagnostics file contained four records: two `chart_prepare` and two `navigator_cpu_submit`; zero picking records. The fixture retained its recorded SHA-256 after the run. App data/config directories and diagnostics were under `/tmp/arch-notebook-ui00`; no user case was opened or changed.

## Handoff

The candidate release binary now has a verified native Wayland launch path and emits in-app chart CPU diagnostics on a fictional case. The native baseline is still incomplete because input automation and visual capture were unavailable in this session. Resume on a Wayland host with an authorized input and per-window capture path, perform the exact UI-00 action sequence, and collect repeated samples before accepting UI-00 or starting UI-01. No frame-rate claim is made.
