# Clinton PDF Qwen content demo — GPU access gate — 2026-11-28

## Handoff

- Task: run a local Qwen demonstration on actual Clinton release PDF text and retain privacy-safe evidence.
- Repository/branch: `arch_notebook`, `codex/ui-00-chart-nested-conditions`.
- Product source: unchanged from `c653f31`; evidence captured after commit `3792420522d179c8a7acded32f19360dde654e63`.
- Source PDF: 7,121 pages; SHA-256 `00a9b9e879ffb1835cb216b69f2488e776726f6c239acccae5025b860d4805e9`.
- Local model: Ollama manifest `qwen3.5:9b`; Qwen 3.5 9B GGUF blob digest `sha256:02d45dc1cf451ba2475ac33b301c2dd8f985abe4c182ce04a1f2f5bf0260278d`.
- Runner: llama.cpp b12618, commit `c66103334`; installed package contains `libggml-vulkan.so`.

## Prepared demonstration input

The SQL database retains email headers, relationships, and page-text hashes, not message bodies. To test actual document text, ten PDF pages were sampled from deciles of the 3,051 pages represented by message candidates: pages 2, 828, 1,499, 2,266, 3,179, 3,936, 4,716, 5,491, 6,308, and 7,121. `pdftotext` produced 17,277 characters in total. The extracted page files and combined prompt remain private under `/tmp/clinton-qwen-demo-20261128` with directory mode `0700` and file mode `0600`.

Combined extracted page text SHA-256: `2784962256cc3ebe81b5f26d02bd395a2432e3d3f967820e07b4249e446f097f`.  
Qwen prompt SHA-256: `9e9de16ee4e22e0f9c80be3e7fe916ddecf5705aae88b7aa921eff09b6331a48`.

The prompt treated document contents as untrusted evidence, asked for narrow extraction-quality checks and short quote strings, and prohibited summaries, identity repetition, and conclusions about intent or wrongdoing. No source text or raw model output is included in Git.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Actual PDF text sample prepared | PASS, bounded | Ten stratified pages; 17,277 extracted characters; hashes above. This is a sample, not the full corpus. |
| Physical GPU detected on host | PASS | `lspci -nn` reports AMD Navi 44 Radeon RX 9050/9060 XT. |
| GPU device exposed to this execution | BLOCKED | `/dev/dri` and `/dev/kfd` are absent; `llama-completion --list-devices` reports `(none)`. With display variables unset, `vulkaninfo --summary` reports only CPU `llvmpipe`. |
| GPU-backed Qwen inference | BLOCKED | No hardware device is available for Vulkan/ROCm offload in this sandbox. This is not a successful GPU run. |
| Initial CPU inference attempt | ABORTED / FAIL | It used `--threads 8` without `--device` or `--gpu-layers`, was interrupted with exit 130 after the user noted the CPU execution, and left 1,965 bytes of incomplete output. The response was invalid JSON with zero page records; it is not accepted as analysis evidence. |
| Email-content analysis result | NOT ESTABLISHED | There is no complete, validated Qwen result for the PDF sample. The previously committed Clinton Qwen review used aggregate metrics only and is not content analysis. |

## Next action

Rerun the same private sample on a host execution that exposes the AMD GPU device nodes to llama.cpp, explicitly select the Vulkan device and all model layers, and verify the backend/offloaded-layer log. Then validate the structured output and quote strings against the sampled page texts before writing aggregate-only results to Git. Do not treat the earlier aggregate-only review or this interrupted CPU attempt as the requested content demo.
