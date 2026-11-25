#!/usr/bin/env python3
"""Collect bounded, opt-in CPU timings for generated chart preparation kernels."""

from __future__ import annotations

import argparse
import importlib.util
import json
import statistics
import time
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "generate_chart_fixtures", Path(__file__).with_name("generate-chart-fixtures.py")
)
assert SPEC is not None and SPEC.loader is not None
FIXTURES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FIXTURES)
generated_edges = FIXTURES.generated_edges
stable_id = FIXTURES.stable_id


def percentile(values: list[int], percent: float) -> int:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int((len(ordered) - 1) * percent))]


def prepare_chart(entity_count: int, links: list[dict]) -> tuple[dict[str, int], list[int]]:
    positions = [
        (0.05 + ((index * 6_187 + 97) % 9_000) / 10_000,
         0.05 + ((index * 7_919 + 313) % 9_000) / 10_000)
        for index in range(entity_count)
    ]
    index_by_id = {stable_id("entity", i): i for i in range(entity_count)}
    lanes: dict[tuple[int, int], list[int]] = {}
    degree = [0] * entity_count
    for position, edge in enumerate(links):
        source, target = index_by_id[edge["from"]], index_by_id[edge["to"]]
        degree[source] += 1
        if target != source:
            degree[target] += 1
        lanes.setdefault((min(source, target), max(source, target)), []).append(position)
    lane_offsets = [0.0] * len(links)
    for positions_in_lane in lanes.values():
        center = (len(positions_in_lane) - 1) / 2.0
        for lane, position in enumerate(positions_in_lane):
            lane_offsets[position] = (lane - center) * 18.0
    return {"entities": len(positions), "links": len(links), "labels": 0,
            "lane_groups": len(lanes), "navigator_points": len(positions)}, degree


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--family", choices=("sparse", "hub", "disconnected", "dense"), required=True)
    parser.add_argument("--nodes", type=int, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--sample-count", type=int, default=30)
    parser.add_argument("--max-samples", type=int, default=64)
    args = parser.parse_args()
    if not 1 <= args.sample_count <= args.max_samples <= 64:
        parser.error("require 1 <= --sample-count <= --max-samples <= 64")

    links = generated_edges(args.family, args.nodes)
    samples: list[int] = []
    timings: dict[str, list[int]] = {"chart_prepare_ns": [], "hit_test_scan_ns": [], "navigator_projection_ns": []}
    work: dict[str, int] = {}
    degrees: list[int] = []
    for sample in range(args.sample_count):
        start = time.perf_counter_ns()
        work, degrees = prepare_chart(args.nodes, links)
        timings["chart_prepare_ns"].append(time.perf_counter_ns() - start)
        # Bounded scan of a stable probe point against generated normalized positions.
        start = time.perf_counter_ns()
        nearest = min(((0.5 - (0.05 + ((i * 6_187 + 97) % 9_000) / 10_000)) ** 2
                       + (0.5 - (0.05 + ((i * 7_919 + 313) % 9_000) / 10_000)) ** 2, i)
                      for i in range(args.nodes))
        _ = nearest
        timings["hit_test_scan_ns"].append(time.perf_counter_ns() - start)
        start = time.perf_counter_ns()
        _ = [8.0 + x * 138.0 for x in (0.05 + ((i * 6_187 + 97) % 9_000) / 10_000 for i in range(args.nodes))]
        timings["navigator_projection_ns"].append(time.perf_counter_ns() - start)
        samples.append(sample)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    document = {
        "schema_version": 1,
        "measurement_type": "synthetic_cpu_kernel_timing_not_frame_rate",
        "family": args.family,
        "entities": args.nodes,
        "relationships": len(links),
        "sample_count": len(samples),
        "sample_cap": args.max_samples,
        "work_counts": work,
        "degree": {"max": max(degrees, default=0), "mean": statistics.fmean(degrees) if degrees else 0.0},
        "timings": {name: {"p50_ns": percentile(values, 0.50), "p95_ns": percentile(values, 0.95),
                           "p99_ns": percentile(values, 0.99), "max_ns": max(values)}
                    for name, values in timings.items()},
        "cache": {"geometry_cache": "not instrumented", "hits": None, "misses": None},
        "paint_shape_count": None,
        "notes": "Offscreen CPU kernel measurements only. No display, rasterization, GPU, or frame-rate claim.",
    }
    args.output.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"path": str(args.output), "sample_count": len(samples), "entities": args.nodes,
                      "relationships": len(links), "timings": document["timings"]}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
