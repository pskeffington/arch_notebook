# SQL scheduler readiness and GPU demo helper — Linux — 2026-11-28

## Handoff

- **Task / owner:** Improve truthful SQL schedule status and GPU backend selection for the Clinton Qwen demo; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `b853e5128839637778ca4be099a978b321abc49a`.
- **Changed product source blob:** `src/main.rs` SHA-1 `dbca413df62dd7ac20c60ad05e9c20ffb535e547`.
- **Changed helper blob:** `tools/run-clinton-qwen-gpu-demo.sh` SHA-1 `afdbe6c4835d0624a0fabb1de47c3343202a81ed`.
- **Installed release binary:** SHA-256 `1b918b2a9e94143467eef1da8905bfe06de080fe738228c6a9f853204850010f`; matches `/home/sysadmin/.local/bin/arch-notebook`.
- **Data boundary:** Tests use only synthetic fixtures; the Clinton sample, prompt, model, and generated output remain local and private.

### Changed paths

- `src/main.rs`
- `tools/run-clinton-qwen-gpu-demo.sh`
- `scripts/test-qwen-gpu-demo-launcher.py`
- `README.md`
- `docs/build-acceptance.md`
- `docs/evidence/deployment/CLINTON-QWEN-GPU-DEMO-LAUNCHER-01-linux.md`
- this evidence record

## Changes

The Automation **Check schedule** result now distinguishes command completion from dispatch readiness. Green means Linux reports `active and enabled` or macOS reports `loaded`. Missing, inactive, or unobservable schedules are amber warnings; command failures remain red.

The GPU demo helper now selects the first enumerated GPU backend automatically and supports `LLAMA_GPU_DEVICE` for an explicit backend ID. It still requires nonzero model-layer offload in the backend log and never falls back to CPU. The current environment cannot expose a device, so the helper stops before inference.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Rust workspace suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target CARGO_BUILD_JOBS=1 cargo test --locked --offline` exited 0: 110 passed, 0 failed, 3 ignored. This includes the scheduler-status readiness cases. |
| Rust formatting and check | PASS | `cargo fmt --all -- --check` and `CARGO_TARGET_DIR=/tmp/arch-notebook-target CARGO_BUILD_JOBS=1 cargo check --locked --offline` exited 0. |
| GPU helper synthetic acceptance | PASS | `python3 scripts/test-qwen-gpu-demo-launcher.py` exited 0: automatic backend choice, explicit override, layer flags, private output, ten-page structure/quote checks, fail-closed missing-device, offload-log requirement, and quote traceability. The fake runner does not invoke Qwen or a GPU. |
| Installed helper synchronization | PASS | Installer exited 0; installed runner compares byte-for-byte with repository source and the Desktop/menu launcher remains synchronized. |
| Current-user app install | PASS | Release build and installed binary have SHA-256 `1b918b2a9e94143467eef1da8905bfe06de080fe738228c6a9f853204850010f`; `scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report /tmp/arch-notebook-sql-schedule-install-acceptance.json` reported `PASS` and installer exit code 0. |
| Actual Qwen GPU inference | BLOCKED | The host has an AMD Radeon and Vulkan driver files, and sysfs reports render node `226:128` and KFD `511:0`; this execution's `/dev` contains neither `/dev/dri` nor `/dev/kfd`. `llama-completion --list-devices` reports `(none)` and Vulkan sees only a CPU device. Installed helper exited 1 before inference and explained that `/dev/dri/renderD128` is not exposed; it explicitly refused CPU fallback. |
| User systemd live schedule | BLOCKED | `systemctl --user list-timers --all --no-pager` is denied by the execution environment. Schedule-state semantics are covered by Rust tests; live dispatch is not claimed. |

Host-wide AMD counters previously read `gpu_busy_percent=7` and `mem_info_vram_used=962244608` bytes at 2026-11-28 19:43:22 UTC. These global counters do not identify a llama.cpp client or prove Qwen offload. Re-running the installed launcher after the helper update again listed `(none)` and exited 1 with the missing `/dev/dri/renderD128` exposure explanation.

## Reproduction

```sh
cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/arch-notebook-target CARGO_BUILD_JOBS=1 cargo check --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target CARGO_BUILD_JOBS=1 cargo test --locked --offline
python3 scripts/test-qwen-gpu-demo-launcher.py
tools/install-clinton-qwen-gpu-demo-launcher.sh
/home/sysadmin/.local/bin/arch-notebook-clinton-qwen-gpu-demo
```

The final command is expected to exit 1 in this sandbox until an accelerator backend is exposed to llama.cpp. No Clinton message body or model response is included here.

## Next actions

Exact-head remote CI status has not been checked for this candidate. Expose `/dev/dri/renderD128` to the inference execution environment (and `/dev/kfd` if using ROCm), then run the installed GPU demo and verify backend offload plus quote checks. Separately observe a live user-systemd timer dispatch from an environment with user-bus access.
