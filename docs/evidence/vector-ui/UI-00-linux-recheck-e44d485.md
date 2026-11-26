# UI-00 Linux native runtime recheck

- **Result:** PASS for release build and limited native chart execution; BLOCKED for the complete native interaction and visual baseline.
- **Task / owner:** Continue UI-00 native acceptance on Linux; Arch Notebook implementation session.
- **Recorded:** 2026-11-26.
- **Source commit:** `e44d48524c4adf1e9d23aea89c67845d716c0d30` on `codex/ui-00-chart-baseline`.
- **Source SHA-256:** `a572a828f4fe705c5ad3091399c0fd1fb8d0e01aefec27de46b858c2a0e86aef`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `0614576294efd356f606081dc437d22672e0e0a48b4612f86b436a3e32f694b9`.
- **Fixture:** disposable copy at `/tmp/arch-notebook-ui00-recheck/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.
- **Diagnostics:** `/tmp/arch-notebook-ui00-recheck/chart-diagnostics.jsonl`, SHA-256 `68a5cc564c8257f97fc77c5076a255687067630032de847a47df8f985fd6b796`.
- **Host:** Linux x86_64, kernel `7.3.5-zen1-1-zen`, `rustc 1.100.0`, `cargo 1.100.0`; Wayland session `wayland-1`. One successful compositor query reported a 2560×1440 output at 143.967 Hz. The session was not consistently reachable by later processes.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS, exit 0 | Optimized release binary built from this source. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS, exit 0 | 18 passed, 0 failed, 0 ignored; recorded for this source in [SQL cancellation evidence](../sql-automation/SQL-CANCEL-01-worker-linux.md). |
| Release app opened the copied correctness fixture and emitted chart diagnostics | PASS, limited sample | Five `chart_prepare` samples and five `navigator_cpu_submit` samples were written. This confirms that the app's chart and navigator CPU paths executed on the native Wayland session; it does not measure frame presentation. |
| X11 window query: `xwininfo -root -tree` | BLOCKED, exit 1 | `unable to open display ":0"`. |
| Wayland access and interaction | BLOCKED / inconsistent | `wayland-info` once listed the compositor and output. Subsequent probes returned `failed to create display: Operation not permitted`, and three app launches returned `WaylandError(Connection(NoCompositor))`. `wtype`, `ydotool`, and `dotool` were not installed. |
| Window screenshot and visual inspection | BLOCKED | No screenshot of the Arch Notebook window was captured or inspected. |
| Full UI-00 native action sequence | BLOCKED | No pan, zoom, drag, lock, selection, search, navigator, or layout interaction trace was collected during this recheck. |

The five observed `chart_prepare` durations were 202,624; 52,505; 51,077; 77,168; and 91,495 ns. The five `navigator_cpu_submit` durations were 13,326; 5,154; 5,110; 7,266; and 7,515 ns. These are small-sample CPU spans from one fixture run, not frame intervals or performance claims. No picking sample was produced. The app command was still running when the tool first returned; its final exit code was not retained. A later process inspection found it had exited. The fixture copy remained byte-identical, and no case was saved.

## Reproduction

```sh
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline
```

The runtime command that produced the samples was:

```sh
timeout 15s env HOME=/tmp/arch-notebook-ui00-recheck/home XDG_DATA_HOME=/tmp/arch-notebook-ui00-recheck/xdg-data XDG_CONFIG_HOME=/tmp/arch-notebook-ui00-recheck/xdg-config ARCH_NOTEBOOK_CHART_DIAGNOSTICS=/tmp/arch-notebook-ui00-recheck/chart-diagnostics.jsonl /tmp/arch-notebook-target/release/arch-notebook --open-case /tmp/arch-notebook-ui00-recheck/correctness.case.json
```

## Handoff

The current source has a passing release build and a limited native chart execution sample. UI-00 remains open until a stable display session permits the documented interaction sequence, repeated workload sampling, and inspected window screenshots. Do not treat this runtime sample as completion of the native gate or as authorization to start UI-01. IBM application interoperability remains a separate acceptance task.
