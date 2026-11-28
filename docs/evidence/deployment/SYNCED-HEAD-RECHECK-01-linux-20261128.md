# Synchronized feature-head recheck — Linux — 2026-11-28

## Handoff

- **Task / owner:** Recheck the synchronized Rust branch after adding the GPU-demo launcher acceptance gate; orchestrator-owned verification.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Source / candidate:** Product source `c653f3141b224a10dee721542965cc1a42b30866`; synchronized candidate `ec8325b9b0b6bed089dad3e4abc57c57b6a987e6`.
- **Working tree:** clean and identical to `origin/codex/ui-00-chart-nested-conditions` at verification time.
- **Scope:** Offline Rust regression suite, GPU-runner device visibility, and live scheduler observability. Product source is unchanged since `c653f31`; recent commits add documentation, launcher assets, and acceptance tooling.

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| Rust workspace tests | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target CARGO_BUILD_JOBS=1 cargo test --locked --offline` exited 0: 110 passed, 0 failed, 3 ignored. |
| First default-target test attempt | FAIL, environment | `cargo test --locked --offline` exited 101 because the repo-local `target/` exhausted the 16 GiB `/tmp` filesystem. The partial 717 MiB directory created by that command was removed. The retry used the repository-prescribed external build target and completed. |
| GPU device available to llama.cpp | BLOCKED | `llama-completion --list-devices` reports `(none)`; `/dev/dri` and `/dev/kfd` are absent from the execution environment. Vulkan reports only a CPU physical device. |
| Qwen/Ollama process evidence | NOT ESTABLISHED | At 2026-11-28 19:43:22 UTC, host-wide AMD sysfs counters reported `gpu_busy_percent=7` and `mem_info_vram_used=962244608` bytes. Those global counters do not identify a client. No llama inference process was visible in this execution namespace; `ollama ps` could not access `127.0.0.1:11434` (`Operation not permitted`). Monitor activity alone does not prove Qwen model offload. |
| Live user systemd dispatch | BLOCKED | `systemctl --user list-timers --all --no-pager` exited 1 because the user-scope bus is denied. Existing isolated scheduler harness evidence is separate. |
| Remote Actions result for candidate | UNVERIFIED | Manual dispatch request returned success, but the workflow-run listing was unavailable (GitHub API connection error/404); combined commit status contained no checks. |

## Reproduction

```sh
CARGO_TARGET_DIR=/tmp/arch-notebook-target CARGO_BUILD_JOBS=1 cargo test --locked --offline
/home/sysadmin/.local/lib/llama.cpp/b12618/llama-b12618/llama-completion --list-devices
systemctl --user list-timers --all --no-pager
```

Do not infer a GPU inference pass from desktop GPU activity. To run the prepared private Clinton sample, the inference process must expose a llama.cpp Vulkan device, record nonzero GPU layer offload, and pass the launcher’s bounded structure and quote checks. No sample body or model output is stored in this report.

## Next actions

1. Recheck GitHub Actions when the API is available and attach exact-head CI status to this candidate.
2. Run the GPU demo only from an execution environment that exposes a device to llama.cpp.
3. Verify scheduled SQL delivery from a session with user-systemd bus access.
