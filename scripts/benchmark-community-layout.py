#!/usr/bin/env python3
"""Measure deterministic community-aware layout CPU time on synthetic charts."""

from __future__ import annotations

import argparse
import json
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
                results.append(json.loads(completed.stdout))

    report = {
        "schema_version": 1,
        "benchmark_suite": "arch_notebook_community_force_layout_cpu",
        "host": {"platform": platform.platform(), "machine": platform.machine()},
        "binary": str(binary),
        "settings": {"iterations": args.iterations, "samples": args.samples},
        "results": results,
        "limitations": [
            "Synthetic fictional charts only.",
            "Community detection is performed before timing; the reported timing covers layout coordinates only.",
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
