# Clinton Qwen GPU demo launcher — Linux deployment preparation — 2026-11-28

## Handoff

- **Task / owner:** Prepare an identifiable desktop icon and launcher for the bounded Clinton Qwen GPU demo; deployment preparation owned by this implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `9af72fcc8f3feb827e7d3c42ec4aa2de55d1983a`.
- **Scope:** Add a distinct GPU-demo SVG icon, desktop entry, per-user installer, GPU-only runner, and private-output structure/quote validator. This does not change the Arch Notebook product icon or application binary.
- **Data boundary:** The PDF text sample, prompt, model weights, backend logs, and inference output remain in the private local sample/model paths. None are added to Git.

## Result

The launcher automatically selects the first enumerated GPU backend (or accepts an explicit `LLAMA_GPU_DEVICE` override), requests all model layers for offload, and exits without starting CPU inference when no GPU backend is exposed. A completed run is accepted only if llama.cpp reports nonzero GPU-layer offload and the output contains the exact ten sampled page IDs with bounded fields and quote strings traceable to the private page text. A passing structure/quote check is still only a bounded extraction-quality demonstration and requires human review.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Demo icon render | PASS | `rsvg-convert` rendered the 512×512 SVG successfully; the icon was visually inspected. |
| Desktop entry validity | PASS | `desktop-file-validate packaging/arch/arch-notebook-qwen-gpu-demo.desktop` exited 0. |
| Launcher syntax | PASS | `bash -n` exited 0 for both shell scripts; Python AST parsing and SVG XML parsing passed. |
| Per-user deployment | PASS | Installer exited 0. Installed icon and launcher hashes match repository assets; application-menu and Desktop entries are identical and validated. |
| Synthetic launcher acceptance | PASS | `python3 scripts/test-qwen-gpu-demo-launcher.py` exercised automatic backend selection, explicit device override, layer flags, private output permissions, ten-page structure and quote validation, missing-device fail-closed behavior, missing-offload rejection, and untraceable-quote rejection. It uses synthetic pages and a fake runner; it does not invoke Qwen or a GPU. |
| GPU-only guard on this host | PASS, guard behavior | Running the installed launcher printed `Available devices: (none)` and exited 1 with `no CPU fallback was started`; inference did not start. |
| Qwen GPU inference | BLOCKED | The current environment exposes no llama.cpp GPU device. No output is accepted as Qwen analysis evidence. |

Installed paths:

- `/home/sysadmin/.local/bin/arch-notebook-clinton-qwen-gpu-demo`
- `/home/sysadmin/.local/share/applications/arch-notebook-clinton-qwen-gpu-demo.desktop`
- `/home/sysadmin/.local/share/icons/hicolor/scalable/apps/arch-notebook-qwen-gpu-demo.svg`
- `/home/sysadmin/Desktop/Arch Notebook — Qwen GPU Demo.desktop`

The desktop entry embeds the absolute installed executable path so it does not depend on the graphical session's `PATH`. Both launcher copies use icon ID `arch-notebook-qwen-gpu-demo`.

## Reproduction

From the repository root:

```sh
tools/install-clinton-qwen-gpu-demo-launcher.sh
desktop-file-validate packaging/arch/arch-notebook-qwen-gpu-demo.desktop
/home/sysadmin/.local/bin/arch-notebook-clinton-qwen-gpu-demo
```

On this host, the final command is expected to exit 1 until llama.cpp exposes a GPU backend. The helper selects the first recognized GPU backend automatically; set `LLAMA_GPU_DEVICE` to an exact enumerated backend ID to select another one. It will not use a CPU fallback.

## Changed paths

- `assets/icons/arch-notebook-qwen-gpu-demo.svg`
- `packaging/arch/arch-notebook-qwen-gpu-demo.desktop`
- `tools/install-clinton-qwen-gpu-demo-launcher.sh`
- `tools/run-clinton-qwen-gpu-demo.sh`
- `tools/validate-clinton-qwen-gpu-output.py`
- `scripts/test-qwen-gpu-demo-launcher.py`
- `.github/workflows/build.yml`
- `docs/build-acceptance.md`
- this evidence record

**Next action:** Run the installed launcher where `llama-completion --list-devices` exposes the AMD Vulkan backend, then retain aggregate validation results only after GPU offload and quote-trace checks pass.
