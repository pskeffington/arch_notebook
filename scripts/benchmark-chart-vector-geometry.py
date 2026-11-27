#!/usr/bin/env python3
"""Run reproducible CPU-only chart geometry measurements on synthetic cases."""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True, help="built arch-notebook executable")
    parser.add_argument("--output", type=Path, required=True, help="JSON report path")
    parser.add_argument("--iterations", type=int, default=7)
    parser.add_argument("--sizes", type=int, nargs="+", default=[100, 1_000, 10_000])
    parser.add_argument("--pan", type=float, nargs=2, default=(0.0, 0.0), metavar=("X", "Y"))
    parser.add_argument("--zoom", type=float, default=None)
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
    if args.iterations < 5:
        parser.error("--iterations must be at least 5")
    if any(size < 10 or size > 10_000 for size in args.sizes):
        parser.error("sizes must be between 10 and 10,000")

    results = []
    with tempfile.TemporaryDirectory(prefix="arch-notebook-vector-bench-") as temporary:
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
                if args.zoom is not None or args.pan != (0.0, 0.0):
                    case = json.loads(fixture.read_text(encoding="utf-8"))
                    if args.zoom is not None:
                        case["active_chart_zoom"] = args.zoom
                    case["active_chart_pan"] = list(args.pan)
                    fixture.write_text(
                        json.dumps(case, ensure_ascii=False, indent=2) + "\n",
                        encoding="utf-8",
                    )
                completed = subprocess.run(
                    [
                        str(binary),
                        "--benchmark-chart-vectors",
                        str(fixture),
                        "--iterations",
                        str(args.iterations),
                    ],
                    check=True,
                    capture_output=True,
                    text=True,
                )
                report = json.loads(completed.stdout)
                report["fixture_family"] = family
                results.append(report)

    output = {
        "schema_version": 1,
        "benchmark_suite": "arch_notebook_chart_vector_cpu_geometry",
        "host": {"platform": platform.platform(), "machine": platform.machine()},
        "binary": str(binary),
        "binary_sha256": binary_sha256,
        "results": results,
        "view_transform": {"zoom": args.zoom, "pan": list(args.pan)},
        "limitations": [
            "Synthetic fictional graph fixtures only.",
            "Measures CPU geometry preparation and nearest-link hit testing; viewport cull counts estimate records omitted from link paint submission.",
            "Does not measure painting, text shaping, GPU work, compositor or presentation latency, or FPS.",
            "Timing values are host-specific and are not universal performance targets.",
        ],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(args.output.resolve()), "cases": len(results)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
