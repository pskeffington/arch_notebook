# Clinton Qwen GPU log-channel fix — Linux — 2026-11-28

## Handoff

- **Task / owner:** Ensure the GPU demo helper captures llama.cpp backend evidence and retains rejected responses privately; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `9de2cd81b7891bd9e0934ee8a2f023ffd22167a4`.
- **Input evidence:** `/tmp/clinton-qwen-demo-20261128/gpu-qwen-output-20261128T193914Z.log`, SHA-256 `7da8e5f7a25e1ae4e79244988ef316ea8b269d39272726547fdde8e0330a65bb` (local-only; no source text or model output).
- **Data boundary:** The Clinton prompt, extracted pages, and model response remain private under `/tmp`; synthetic fixtures are used for acceptance.

## Finding

The 19:39:14 UTC historical demo log was created after the then-installed helper passed its `Vulkan0` device-list check and launched llama.cpp. It contains a RadV warning and llama generation performance counters. However, it does not contain the required `offloaded N/N layers to GPU` line. The helper removed the generated JSON after this verification failed, so the run did not produce an accepted content result. The log supports that a Vulkan-capable path was reached; it does not prove how many model layers ran on the GPU or validate the lost response.

The current Codex process is a separate execution context: llama.cpp lists `(none)` here and `/dev/dri/renderD128` is not exposed. Host-wide GPU counters cannot attribute activity to the historical llama process.

## Change

The demo runner now passes llama.cpp's supported `--log-file` option and keeps that backend log separate from standard error and generated JSON. It checks the explicit backend log for nonzero GPU layer offload. If inference, offload confirmation, validator availability, or structure/quote checks fail, the response is renamed to a private `.unverified.json` file instead of being deleted or left under the accepted-result filename. The helper still reports failure and does not accept or link unverified output.

Changed paths:

- `tools/run-clinton-qwen-gpu-demo.sh`
- `scripts/test-qwen-gpu-demo-launcher.py`
- `README.md`
- `docs/build-acceptance.md`
- this evidence record

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| llama.cpp supports explicit log routing | PASS | Installed b12618 `llama-completion --help` lists `--log-file FNAME`. |
| GPU helper synthetic acceptance | PASS | `python3 scripts/test-qwen-gpu-demo-launcher.py` exited 0. It verifies device selection, explicit backend-log path, nonzero offload requirement, private files, quote validation, and private retention with an unverified filename on offload/quote failure. The fake runner does not invoke llama.cpp or a GPU. |
| Shell syntax and diff whitespace | PASS | `bash -n tools/run-clinton-qwen-gpu-demo.sh` and `git diff --check` exited 0. |
| Current-user launcher synchronization | PASS | `tools/install-clinton-qwen-gpu-demo-launcher.sh` exited 0; `cmp tools/run-clinton-qwen-gpu-demo.sh /home/sysadmin/.local/bin/arch-notebook-clinton-qwen-gpu-demo` exited 0. |
| New real GPU run | BLOCKED | Re-running the installed helper exited 1 after listing `(none)`; it reported `/dev/dri/renderD128` is not exposed to this process and did not start inference. The historical generated run lacks layer-offload evidence and has no retained response. |

## Next action

Run the installed desktop GPU demo from the machine context that previously listed `Vulkan0`. Confirm that the new `.log` file contains a nonzero offloaded-layer count and that the command reports quote validation passed. If either gate fails, keep the response only as `.unverified.json`; do not import it as an accepted analysis result.
