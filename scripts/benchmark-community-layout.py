#!/usr/bin/env python3
"""Measure deterministic community-aware layout CPU time on synthetic charts."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import platform
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--iterations", type=int, default=20)
    parser.add_argument("--samples", type=int, default=5)
    parser.add_argument("--sizes", type=int, nargs="+", default=[1_000, 5_000, 10_000])
    parser.add_argument(
        "--families",
        nargs="+",
        choices=("sparse", "hub", "disconnected", "dense"),
        default=["sparse", "hub", "disconnected", "dense"],
    )
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    if not binary.is_file():
        parser.error(f"not a file: {binary}")
    binary_sha256 = hashlib.sha256(binary.read_bytes()).hexdigest()
    if not 1 <= args.iterations <= 120 or not 3 <= args.samples <= 100:
        parser.error("iterations must be 1..120 and samples 3..100")
    if any(size < 10 or size > 10_000 for size in args.sizes):
        parser.error("sizes must be between 10 and 10,000")

    results = []
    with tempfile.TemporaryDirectory(prefix="arch-notebook-layout-bench-") as temporary:
        temp = Path(temporary)
        for family in args.families:
            for size in args.sizes:
                fixture = temp / f"{family}-{size}.json"
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
                    check=True,
                    capture_output=True,
                    text=True,
                )
                completed = subprocess.run(
                    [
                        str(binary),
                        "--benchmark-community-layout",
                        str(fixture),
                        "--iterations",
                        str(args.iterations),
                        "--samples",
                        str(args.samples),
                    ],
                    check=True,
                    capture_output=True,
                    text=True,
                )
                result = json.loads(completed.stdout)
                timings = result.get("timings", {})
                required_timings = (
                    "g0_connectivity_layout",
                    "community_force_layout",
                )
                if result.get("benchmark") != "g0_connectivity_vs_community_force_layout_cpu":
                    raise SystemExit("binary returned an unexpected layout benchmark format")
                for timing_name in required_timings:
                    summary = timings.get(timing_name)
                    if not isinstance(summary, dict) or summary.get("samples") != args.samples:
                        raise SystemExit(
                            f"binary omitted {args.samples}-sample timing summary: {timing_name}"
                        )
                    if not all(
                        isinstance(summary.get(key), (int, float))
                        and math.isfinite(summary[key])
                        and summary[key] >= 0
                        for key in ("min_ns", "median_ns", "p95_ns", "max_ns", "mean_ns")
                    ):
                        raise SystemExit(f"binary returned invalid timing values: {timing_name}")
                results.append(result)

    report = {
        "schema_version": 1,
        "benchmark_suite": "arch_notebook_g0_connectivity_vs_community_force_layout_cpu",
        "host": {"platform": platform.platform(), "machine": platform.machine()},
        "binary": str(binary),
        "binary_sha256": binary_sha256,
        "settings": {"iterations": args.iterations, "samples": args.samples},
        "results": results,
        "limitations": [
            "Synthetic fictional charts only.",
            "The production G0 connectivity layout and community force layout run on identical fixture inputs and are timed separately.",
            "Community detection is performed once before the repeated layout timings.",
            "Does not measure native chart painting, input latency, compositor, or presentation.",
            "Timing is host-specific and is not a universal performance target.",
        ],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(args.output.resolve()), "cases": len(results)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
