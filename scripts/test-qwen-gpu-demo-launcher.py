#!/usr/bin/env python3
"""Exercise the GPU-only Qwen demo launcher with private synthetic fixtures."""

from __future__ import annotations

import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parent.parent
RUNNER = ROOT / "tools/run-clinton-qwen-gpu-demo.sh"
PAGES = [2, 828, 1499, 2266, 3179, 3936, 4716, 5491, 6308, 7121]


def write_fake_runner(path: Path) -> None:
    source = '''#!/usr/bin/env python3
import json
import os
from pathlib import Path
import sys

args = sys.argv[1:]
with open(os.environ["FAKE_LLAMA_CALL_LOG"], "a", encoding="utf-8") as log:
    log.write(" ".join(args) + "\\n")
if "--list-devices" in args:
    print(os.environ.get("FAKE_LLAMA_DEVICES", "Available devices:\\n  Vulkan0 (fixture GPU)"))
    raise SystemExit(0)
required = {"--device": os.environ["FAKE_LLAMA_EXPECTED_DEVICE"], "--gpu-layers": "all", "--n-predict": "900"}
for name, value in required.items():
    if args[args.index(name) + 1] != value:
        raise SystemExit(f"missing explicit GPU demo argument: {name} {value}")
root = Path(os.environ["CLINTON_QWEN_DEMO_DIR"])
pages = [2, 828, 1499, 2266, 3179, 3936, 4716, 5491, 6308, 7121]
records = []
for page in pages:
    text = " ".join((root / f"page-{page:04d}.txt").read_text().split())
    quote = " ".join(text.split()[:4])
    if os.environ.get("FAKE_LLAMA_BAD_QUOTE") == "1":
        quote = "not present in source"
    records.append({"page": page, "email_like": True, "header_fields_present": ["from"],
                    "body_text_present": True, "quote_check": quote, "qa_note": None})
print(json.dumps({"records": records, "limitation": "synthetic acceptance output"}))
if os.environ.get("FAKE_LLAMA_OFFLOAD", "1") == "1":
    print("offloaded 8/8 layers to GPU", file=sys.stderr)
'''
    path.write_text(source, encoding="utf-8")
    path.chmod(0o755)


def invoke(root: Path, runner: Path, *, devices: str, call_log: Path,
           device: str | None = None, expected_device: str = "Vulkan0",
           offload: bool = True, bad_quote: bool = False) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.update({
        "CLINTON_QWEN_DEMO_DIR": str(root),
        "CLINTON_QWEN_MODEL": str(root / "fixture-model.gguf"),
        "LLAMA_COMPLETION": str(runner),
        "FAKE_LLAMA_DEVICES": devices,
        "FAKE_LLAMA_EXPECTED_DEVICE": expected_device,
        "FAKE_LLAMA_CALL_LOG": str(call_log),
        "FAKE_LLAMA_OFFLOAD": "1" if offload else "0",
        "FAKE_LLAMA_BAD_QUOTE": "1" if bad_quote else "0",
    })
    if device is not None:
        env["LLAMA_GPU_DEVICE"] = device
    else:
        env.pop("LLAMA_GPU_DEVICE", None)
    return subprocess.run(
        [str(RUNNER)], cwd=ROOT, env=env, capture_output=True, text=True, check=False
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="arch-notebook-qwen-gpu-demo-") as temporary:
        root = Path(temporary)
        (root / "fixture-model.gguf").write_bytes(b"synthetic model placeholder")
        (root / "demo-prompt.txt").write_text("Synthetic pages only.\n", encoding="utf-8")
        for page in PAGES:
            (root / f"page-{page:04d}.txt").write_text(
                f"synthetic quoted phrase page {page} fixture body", encoding="utf-8"
            )
        fake_runner = root / "fake-llama-completion"
        write_fake_runner(fake_runner)

        call_log = root / "gpu-calls.log"
        success = invoke(root, fake_runner, devices="Available devices:\n  Vulkan0: fixture GPU", call_log=call_log)
        if success.returncode != 0:
            raise SystemExit(f"synthetic GPU path failed: {success.stdout}{success.stderr}")
        invocation = call_log.read_text(encoding="utf-8")
        if "--device Vulkan0" not in invocation or "--gpu-layers all" not in invocation:
            raise SystemExit("launcher did not auto-select the enumerated Vulkan device and request GPU layers")
        outputs = list(root.glob("gpu-qwen-output-*.json"))
        logs = list(root.glob("gpu-qwen-output-*.log"))
        if len(outputs) != 1 or len(logs) != 1:
            raise SystemExit("successful path did not create exactly one private result and backend log")
        if stat.S_IMODE(outputs[0].stat().st_mode) != 0o600 or stat.S_IMODE(logs[0].stat().st_mode) != 0o600:
            raise SystemExit("successful path did not keep output and backend log private")
        if "10 page records" not in success.stdout:
            raise SystemExit("successful path omitted output validation confirmation")
        outputs[0].unlink()
        logs[0].unlink()

        call_log.unlink()
        override = invoke(
            root, fake_runner,
            devices="Available devices:\n  Vulkan0: fixture Vulkan GPU\n  CUDA0: fixture CUDA GPU",
            call_log=call_log, device="CUDA0", expected_device="CUDA0",
        )
        if override.returncode != 0 or "--device CUDA0" not in call_log.read_text(encoding="utf-8"):
            raise SystemExit(f"explicit device override failed: {override.stdout}{override.stderr}")
        list(root.glob("gpu-qwen-output-*.json"))[0].unlink()
        list(root.glob("gpu-qwen-output-*.log"))[0].unlink()

        call_log.unlink()
        no_device = invoke(root, fake_runner, devices="Available devices:\n  (none)", call_log=call_log)
        calls = call_log.read_text(encoding="utf-8").splitlines()
        if no_device.returncode == 0 or "no CPU fallback" not in no_device.stderr:
            raise SystemExit("no-device path did not fail closed with a clear message")
        if calls != ["--list-devices"]:
            raise SystemExit("inference runner was invoked despite missing GPU device")

        call_log.unlink()
        no_offload = invoke(root, fake_runner, devices="Available devices:\n  Vulkan0: fixture GPU", call_log=call_log, offload=False)
        if no_offload.returncode == 0 or "output discarded" not in no_offload.stderr:
            raise SystemExit("backend without GPU offload evidence was incorrectly accepted")
        if list(root.glob("gpu-qwen-output-*.json")):
            raise SystemExit("output without backend offload evidence was retained")

        call_log.unlink()
        bad_quote = invoke(root, fake_runner, devices="Available devices:\n  Vulkan0: fixture GPU", call_log=call_log, bad_quote=True)
        if bad_quote.returncode == 0 or "quote validation failed" not in bad_quote.stderr:
            raise SystemExit("untraceable quote output was incorrectly accepted")
        if list(root.glob("gpu-qwen-output-*.json")):
            raise SystemExit("quote-invalid output was retained")

    print("PASS: automatic GPU selection, explicit device override, offload, private output, ten-page validation, fail-closed device/offload/quote checks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
