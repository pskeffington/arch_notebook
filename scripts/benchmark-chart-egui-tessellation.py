#!/usr/bin/env python3
"""Measure headless Arch Notebook UI shape generation and egui CPU tessellation."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BENCHMARK = "headless_chart_ui_and_egui_tessellation_cpu"
TEST_NAME = "tests::benchmark_headless_chart_ui_and_tessellation_cpu"


def find_report(output: str) -> dict:
    decoder = json.JSONDecoder()
    for offset, character in enumerate(output):
        if character != "{":
            continue
        try:
            value, _ = decoder.raw_decode(output[offset:])
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict) and value.get("benchmark") == BENCHMARK:
            return value
    raise RuntimeError("benchmark test did not emit its JSON report")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True, help="machine-readable JSON report")
    parser.add_argument("--iterations", type=int, default=30)
    parser.add_argument("--sizes", type=int, nargs="+", default=[100, 1_000])
    parser.add_argument("--pan", type=float, nargs=2, default=(0.0, 0.0), metavar=("X", "Y"))
    parser.add_argument(
        "--families",
        nargs="+",
        choices=("sparse", "hub", "disconnected", "dense"),
        default=["sparse", "hub", "disconnected", "dense"],
    )
    parser.add_argument("--target-dir", type=Path, default=Path("/tmp/arch-notebook-target"))
    args = parser.parse_args()
    if not 5 <= args.iterations <= 1000:
        parser.error("--iterations must be between 5 and 1000")
    if any(size < 10 or size > 10_000 for size in args.sizes):
        parser.error("--sizes values must be between 10 and 10,000")

    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(args.target_dir.resolve())
    build = subprocess.run(
        [
            "cargo",
            "test",
            "--release",
            "--locked",
            "--offline",
            "--no-run",
            "--message-format=json",
        ],
        cwd=ROOT,
        env=env,
        check=True,
        capture_output=True,
        text=True,
    )
    test_binary = None
    for line in build.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        target = message.get("target", {})
        if (
            message.get("reason") == "compiler-artifact"
            and target.get("test") is True
            and target.get("name") == "arch-notebook"
        ):
            test_binary = Path(message["executable"])
    if test_binary is None or not test_binary.is_file():
        raise RuntimeError("cargo did not report the release benchmark test executable")

    results = []
    with tempfile.TemporaryDirectory(prefix="arch-notebook-egui-bench-") as temporary:
        temp = Path(temporary)
        for family in args.families:
            for size in args.sizes:
                fixture = temp / f"{family}-{size}.case.json"
                subprocess.run(
                    [
                        "python3",
                        str(ROOT / "scripts/generate-chart-fixtures.py"),
                        "--family",
                        family,
                        "--nodes",
                        str(size),
                        "--output",
                        str(fixture),
                    ],
                    cwd=ROOT,
                    check=True,
                    capture_output=True,
                    text=True,
                )
                if args.pan != (0.0, 0.0):
                    case = json.loads(fixture.read_text(encoding="utf-8"))
                    case["active_chart_pan"] = list(args.pan)
                    fixture.write_text(
                        json.dumps(case, ensure_ascii=False, indent=2) + "\n",
                        encoding="utf-8",
                    )
                case_env = env.copy()
                case_env["ARCH_NOTEBOOK_CHART_RENDER_BENCH_CASE"] = str(fixture)
                case_env["ARCH_NOTEBOOK_CHART_RENDER_BENCH_ITERATIONS"] = str(args.iterations)
                completed = subprocess.run(
                    [
                        str(test_binary),
                        "--exact",
                        TEST_NAME,
                        "--ignored",
                        "--nocapture",
                        "--test-threads=1",
                    ],
                    cwd=ROOT,
                    env=case_env,
                    check=True,
                    capture_output=True,
                    text=True,
                )
                report = find_report(completed.stdout + "\n" + completed.stderr)
                report["fixture_family"] = family
                results.append(report)

    binary_hash = hashlib.sha256(test_binary.read_bytes()).hexdigest()
    output = {
        "schema_version": 1,
        "benchmark_suite": "arch_notebook_headless_ui_and_egui_tessellation_cpu",
        "host": {"platform": platform.platform(), "machine": platform.machine()},
        "test_binary": str(test_binary.resolve()),
        "test_binary_sha256": binary_hash,
        "profile": "release test profile",
        "view_transform": {"pan_points": list(args.pan), "zoom": 1.0},
        "results": results,
        "limitations": [
            "Synthetic fictional graph fixtures only; App::update covers the full application UI at a fixed 1440x900 point viewport.",
            "Reports CPU shape generation and egui CPU tessellation separately, plus submitted shape and mesh counts.",
            "Does not measure GPU execution, compositor work, buffer presentation, input-to-display latency, or FPS.",
            "Host-specific characterization; comparisons require the same host, profile, viewport, and fixture hashes.",
        ],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(args.output.resolve()), "cases": len(results)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
