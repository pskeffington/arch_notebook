#!/usr/bin/env bash
set -euo pipefail

demo_dir="${CLINTON_QWEN_DEMO_DIR:-/tmp/clinton-qwen-demo-20261128}"
model="${CLINTON_QWEN_MODEL:-/home/sysadmin/.ollama/models/blobs/sha256-02d45dc1cf451ba2475ac33b301c2dd8f985abe4c182ce04a1f2f5bf0260278d}"
runner="${LLAMA_COMPLETION:-/home/sysadmin/.local/lib/llama.cpp/b12618/llama-b12618/llama-completion}"
requested_device="${LLAMA_GPU_DEVICE:-}"
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
if [[ -n "$requested_device" ]]; then
  device="$requested_device"
  if ! awk -v device="$device" '{ name = $1; sub(/:$/, "", name); if (name == device) found = 1 } END { exit !found }' <<<"$devices"; then
    fail "GPU device '$device' is unavailable; no CPU fallback was started"
  fi
else
  device="$(awk '$1 ~ /^(Vulkan|CUDA|ROCm|HIP|Metal|OpenCL|SYCL|MUSA)[[:alnum:]_.-]*:?$/ { name = $1; sub(/:$/, "", name); print name; exit }' <<<"$devices")"
  if [[ -z "$device" ]]; then
    if [[ -d /sys/class/drm/renderD128 && ! -e /dev/dri/renderD128 ]]; then
      fail "host DRM render device exists at /sys/class/drm/renderD128 but /dev/dri/renderD128 is not exposed to this process; expose the GPU device or run outside the sandbox; no CPU fallback was started"
    fi
    fail "llama.cpp enumerated no GPU backend device; check host device access; no CPU fallback was started"
  fi
fi

umask 077
output="$demo_dir/gpu-qwen-output-$(date -u +%Y%m%dT%H%M%SZ).json"
log="${output%.json}.log"
unverified_output="${output%.json}.unverified.json"
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
  --log-file "$log" \
  --no-display-prompt \
  --no-conversation \
  --simple-io \
  >"$output" 2>"${output%.json}.stderr.log" || {
    [[ ! -e "$output" ]] || mv -- "$output" "$unverified_output"
    fail "inference failed; inspect private backend log $log and stderr log ${output%.json}.stderr.log; any response is retained as $unverified_output"
  }

if ! grep -Eq 'offloaded [1-9][0-9]*/[1-9][0-9]* layers to GPU' "$log"; then
  mv -- "$output" "$unverified_output"
  fail "backend log did not confirm nonzero GPU layer offload; unverified private output retained at $unverified_output; no result is accepted"
fi
runner_dir="$(dirname -- "${BASH_SOURCE[0]}")"
if [[ -x "$runner_dir/arch-notebook-validate-clinton-qwen-gpu-output" ]]; then
  validator=("$runner_dir/arch-notebook-validate-clinton-qwen-gpu-output")
elif [[ -r "$runner_dir/validate-clinton-qwen-gpu-output.py" ]]; then
  validator=(python3 "$runner_dir/validate-clinton-qwen-gpu-output.py")
else
  mv -- "$output" "$unverified_output"
  fail "the private result validator is not installed; unverified private output retained at $unverified_output"
fi
"${validator[@]}" --output "$output" --pages-dir "$demo_dir" "${pages[@]}" || {
    mv -- "$output" "$unverified_output"
    fail "structured output or quote validation failed; unverified private output retained at $unverified_output"
  }
printf 'Inference and bounded structure/quote checks passed. Human review is still required:\n  %s\n  %s\n' "$output" "$log"
