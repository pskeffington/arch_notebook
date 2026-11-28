#!/usr/bin/env bash
set -euo pipefail

demo_dir="${CLINTON_QWEN_DEMO_DIR:-/tmp/clinton-qwen-demo-20261128}"
model="${CLINTON_QWEN_MODEL:-/home/sysadmin/.ollama/models/blobs/sha256-02d45dc1cf451ba2475ac33b301c2dd8f985abe4c182ce04a1f2f5bf0260278d}"
runner="${LLAMA_COMPLETION:-/home/sysadmin/.local/lib/llama.cpp/b12618/llama-b12618/llama-completion}"
device="${LLAMA_GPU_DEVICE:-Vulkan0}"
prompt="$demo_dir/demo-prompt.txt"
pages=(2 828 1499 2266 3179 3936 4716 5491 6308 7121)

fail() {
  printf 'Clinton Qwen GPU demo: %s\n' "$1" >&2
  exit 1
}

[[ -x "$runner" ]] || fail "llama.cpp runner is missing or not executable: $runner"
[[ -r "$model" ]] || fail "local Qwen model blob is unavailable"
[[ -s "$prompt" ]] || fail "private demo prompt is unavailable: $prompt"

devices="$("$runner" --list-devices 2>&1)" || fail "could not enumerate llama.cpp devices"
printf '%s\n' "$devices"
grep -Fq "$device" <<<"$devices" || fail "GPU device '$device' is unavailable; no CPU fallback was started"

umask 077
output="$demo_dir/gpu-qwen-output-$(date -u +%Y%m%dT%H%M%SZ).json"
log="${output%.json}.log"
printf 'Starting Qwen GPU inference on %s. Output and backend log stay private in %s.\n' "$device" "$demo_dir"
"$runner" \
  --model "$model" \
  --file "$prompt" \
  --n-predict 900 \
  --ctx-size 8192 \
  --threads 8 \
  --temperature 0.1 \
  --seed 42 \
  --device "$device" \
  --gpu-layers all \
  --no-display-prompt \
  --no-conversation \
  --simple-io \
  >"$output" 2>"$log" || fail "inference failed; inspect private backend log $log"

if ! grep -Eq 'offloaded [1-9][0-9]*/[1-9][0-9]* layers to GPU' "$log"; then
  rm -f "$output"
  fail "backend log did not confirm nonzero GPU layer offload; output discarded"
fi
runner_dir="$(dirname -- "${BASH_SOURCE[0]}")"
if [[ -x "$runner_dir/arch-notebook-validate-clinton-qwen-gpu-output" ]]; then
  validator=("$runner_dir/arch-notebook-validate-clinton-qwen-gpu-output")
elif [[ -r "$runner_dir/validate-clinton-qwen-gpu-output.py" ]]; then
  validator=(python3 "$runner_dir/validate-clinton-qwen-gpu-output.py")
else
  rm -f "$output"
  fail "the private result validator is not installed"
fi
"${validator[@]}" --output "$output" --pages-dir "$demo_dir" "${pages[@]}" || {
    rm -f "$output"
    fail "structured output or quote validation failed; output discarded"
  }
printf 'Inference and bounded structure/quote checks passed. Human review is still required:\n  %s\n  %s\n' "$output" "$log"
