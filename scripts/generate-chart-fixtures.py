#!/usr/bin/env python3
"""Generate deterministic, fictional Arch Notebook chart cases for UI acceptance."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import uuid
from collections import Counter, deque
from pathlib import Path


NAMESPACE = uuid.UUID("9613f5a0-8893-4b2b-9c8c-2618f356b3df")
CASE_FORMAT_VERSION = 19
MAX_EDGES = 100_000
ENTITY_KINDS = ("PERSON", "ORGANIZATION", "LOCATION", "ACCOUNT", "DEVICE")


def stable_id(kind: str, value: int | str) -> str:
    return str(uuid.uuid5(NAMESPACE, f"{kind}:{value}"))


def entity(index: int, *, x: float | None = None, y: float | None = None) -> dict:
    # A fixed modular sequence gives reproducible, bounded positions without PRNG state.
    if x is None:
        x = 0.05 + ((index * 6_187 + 97) % 9_000) / 10_000
    if y is None:
        y = 0.05 + ((index * 7_919 + 313) % 9_000) / 10_000
    return {
        "id": stable_id("entity", index),
        "name": f"Fictional Entity {index:05d}",
        "kind": ENTITY_KINDS[index % len(ENTITY_KINDS)],
        "representation": "icon",
        "detail": f"Generated fictional chart fixture entity {index}.",
        "attributes": [["fixture_index", str(index)]],
        "sql_attribute_values": [],
        "sql_attribute_baselines": [],
        "aliases": [],
        "external_ids": [],
        "merged_records": [],
        "x": round(x, 6),
        "y": round(y, 6),
        "position_locked": False,
        "color": [55, 128, 162],
    }


def link(
    index: int,
    source: int,
    target: int,
    *,
    connection_id: str = "",
    ingest_source: str | None = None,
    label: str | None = None,
    directed: bool | None = None,
    date: str | None = None,
) -> dict:
    return {
        "from": stable_id("entity", source),
        "to": stable_id("entity", target),
        "connection_id": connection_id,
        "label": label or f"FICTIONAL RELATIONSHIP {index % 11:02d}",
        "date": date or f"2025-{(index % 12) + 1:02d}-{(index % 27) + 1:02d}",
        "confidence": (index % 3) + 1,
        "directed": index % 2 == 0 if directed is None else directed,
        "notes": "Generated fictional relationship record.",
        "source_notes": "Fixture provenance only.",
        "source_reference": f"FIXTURE-{index:06d}",
        "ingest_source": ingest_source,
        "attributes": [["fixture_record", str(index)]],
    }


def correctness_case() -> dict:
    names = [
        "Avery North",
        "Avery North Coincident",
        "Cedarline Fictional Cooperative",
        "Pier 19 Test Location",
        "Mailbox 0042",
        "Device 17",
        "Isolated Test Entity",
        "Long Label Fixture Entity",
    ]
    entities = [entity(index) for index in range(len(names))]
    for item, name in zip(entities, names, strict=True):
        item["name"] = name
    # Coincident chart positions must not imply that two entity IDs are one entity.
    entities[1]["x"], entities[1]["y"] = entities[0]["x"], entities[0]["y"]
    entities[5]["position_locked"] = True
    entities[7]["name"] = "A deliberately long fictional chart label target " * 3
    entities[7]["x"], entities[7]["y"] = 0.99, 0.99
    links = [
        link(0, 0, 2, connection_id="shared-connection-01", ingest_source="fixture-source-a", label="ASSOCIATED WITH", directed=True, date="2025-01-02"),
        link(1, 0, 2, connection_id="shared-connection-01", ingest_source="fixture-source-b", label="ASSOCIATED WITH", directed=True, date="2025-01-02"),
        link(2, 2, 0, connection_id="reversed-connection-01", ingest_source="fixture-source-a", label="REVERSED TEST RECORD", directed=True, date="2025-01-03"),
        link(3, 2, 3, connection_id="path-step-02", ingest_source="fixture-source-a", label="TRANSFERRED TO", directed=True, date="2025-01-04"),
        link(4, 3, 3, connection_id="loop-01", label="SELF LOOP", directed=False, date="2025-01-05"),
        link(5, 0, 2, label="ANONYMOUS DUPLICATE", directed=False, date="2025-02-01"),
        link(5, 0, 2, label="ANONYMOUS DUPLICATE", directed=False, date="2025-02-01"),
        link(7, 7, 0, connection_id="long-edge-01", label="A VERY LONG FICTIONAL RELATIONSHIP LABEL " * 2, directed=True),
    ]
    label_id = stable_id("label", "long-offscreen")
    return {
        "format_version": CASE_FORMAT_VERSION,
        "title": "VECTOR UI CORRECTNESS FIXTURE",
        "description": "Fictional UI-00 fixture; contains no operational or personal data.",
        "entities": entities,
        "links": links,
        "labels": [{
            "id": label_id,
            "text": "FICTIONAL LONG-VIEWPORT ANNOTATION",
            "x": 0.93,
            "y": 0.92,
            "font_size": 13.0,
            "color": [225, 232, 237],
            "is_legend": False,
            "legend_entries": [],
            "legend_entries_truncated": False,
        }],
        "active_chart_id": stable_id("chart", "correctness"),
        "active_chart_title": "Correctness Chart",
        # Zoom pushes the long edge endpoints beyond the normal viewport while
        # its center curve still crosses the visible chart area.
        "active_chart_zoom": 2.0,
        "active_chart_pan": [0.0, 0.0],
        "chart_tabs": [],
        "chart_templates": [],
        "active_chart_template": None,
        "snapshots": [],
        "automation_jobs": [],
        "automation_runs": [],
        "automation_batches": [],
        "import_specifications": [],
        "graphml_source_catalogs": [],
        "saved_paths": [],
        "selection_sets": [],
    }


def minimal_case(entity_count: int) -> dict:
    if entity_count not in (0, 1, 2):
        raise ValueError("minimal fixtures support exactly 0, 1, or 2 entities")
    entities = [entity(index) for index in range(entity_count)]
    links = [link(0, 0, 1, label="TWO ENTITY FIXTURE")] if entity_count == 2 else []
    case = correctness_case()
    case.update(
        title=f"VECTOR UI MINIMAL {entity_count}",
        description="Fictional minimal chart fixture; no operational or personal data.",
        entities=entities,
        links=links,
        labels=[],
        active_chart_id=stable_id("chart", f"minimal:{entity_count}"),
        active_chart_title=f"Minimal {entity_count}",
    )
    return case


def generated_edges(family: str, node_count: int) -> list[dict]:
    edge_count = {
        "sparse": 3 * node_count,
        "hub": 3 * node_count,
        "disconnected": 2 * node_count,
        "dense": min(10 * node_count, MAX_EDGES),
    }[family]
    edges = []
    if family == "sparse":
        for index in range(edge_count):
            source = index % node_count
            target = (source + 1 + (index // node_count) * 17) % node_count
            edges.append(link(index, source, target))
    elif family == "hub":
        for index in range(edge_count):
            if index < edge_count // 2:
                source, target = 0, 1 + (index % (node_count - 1))
            else:
                source = 1 + ((index - edge_count // 2) % (node_count - 1))
                target = 1 + (source % (node_count - 1))
                if target == source:
                    target = 1 + (target % (node_count - 1))
            edges.append(link(index, source, target))
    elif family == "disconnected":
        component_count = min(8, max(2, node_count // 10))
        active_nodes = node_count - 1  # Reserve one isolated entity as a separate component.
        widths = [active_nodes // component_count] * component_count
        for index in range(active_nodes % component_count):
            widths[index] += 1
        starts, cursor = [], 0
        for width in widths:
            starts.append(cursor)
            cursor += width
        for index in range(edge_count):
            component = index % component_count
            width, start = widths[component], starts[component]
            local_source = (index // component_count) % width
            local_offset = 1 + ((index // (component_count * width)) % (width - 1))
            source = start + local_source
            target = start + ((local_source + local_offset) % width)
            edges.append(link(index, source, target))
    else:  # dense: concentrated local clusters, not an all-pairs graph.
        cluster_width = min(100, node_count)
        for index in range(edge_count):
            cluster = (index // cluster_width) % math.ceil(node_count / cluster_width)
            start = cluster * cluster_width
            width = min(cluster_width, node_count - start)
            source = start + (index % width)
            target = start + ((index * 37 + 1) % width)
            edges.append(link(index, source, target))
    return edges


def workload_case(family: str, node_count: int) -> dict:
    if node_count < 10:
        raise ValueError("workload fixtures require at least 10 entities")
    if node_count > 10_000:
        raise ValueError("workload fixtures support at most 10,000 entities")
    entities = [entity(index) for index in range(node_count)]
    links = generated_edges(family, node_count)
    return {
        "format_version": CASE_FORMAT_VERSION,
        "title": f"VECTOR UI {family.upper()} {node_count}",
        "description": "Generated fictional UI benchmark workload; no operational data.",
        "entities": entities,
        "links": links,
        "labels": [],
        "active_chart_id": stable_id("chart", f"{family}:{node_count}"),
        "active_chart_title": f"{family.title()} {node_count}",
        "active_chart_zoom": 1.0,
        "active_chart_pan": [0.0, 0.0],
        "chart_tabs": [],
        "chart_templates": [],
        "active_chart_template": None,
        "snapshots": [],
        "automation_jobs": [],
        "automation_runs": [],
        "automation_batches": [],
        "import_specifications": [],
        "graphml_source_catalogs": [],
        "saved_paths": [],
        "selection_sets": [],
    }


def summarize(case: dict, output: Path) -> dict:
    entities, links = case["entities"], case["links"]
    index_by_id = {item["id"]: index for index, item in enumerate(entities)}
    direction_counts = Counter((edge["from"], edge["to"]) for edge in links if edge["directed"])
    undirected_counts = Counter(
        tuple(sorted((edge["from"], edge["to"]))) for edge in links if not edge["directed"]
    )
    groups = Counter(
        (
            edge["from"] if edge["directed"] else min(edge["from"], edge["to"]),
            edge["to"] if edge["directed"] else max(edge["from"], edge["to"]),
            edge["label"],
            edge["directed"],
        )
        for edge in links
    )
    parallel_groups = sum(count > 1 for count in groups.values())
    parallel_records = sum(count for count in groups.values() if count > 1)
    graph = [set() for _ in entities]
    for edge in links:
        a, b = index_by_id[edge["from"]], index_by_id[edge["to"]]
        graph[a].add(b)
        graph[b].add(a)
    visited = set()
    components = 0
    for start in range(len(entities)):
        if start in visited:
            continue
        components += 1
        queue = deque([start])
        visited.add(start)
        while queue:
            for neighbor in graph[queue.popleft()]:
                if neighbor not in visited:
                    visited.add(neighbor)
                    queue.append(neighbor)
    payload = (json.dumps(case, ensure_ascii=False, indent=2, separators=(",", ": ")) + "\n").encode()
    output.write_bytes(payload)
    return {
        "path": str(output),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "entities": len(entities),
        "relationship_records": len(links),
        "directed_records": sum(edge["directed"] for edge in links),
        "undirected_records": sum(not edge["directed"] for edge in links),
        "direction_categories": {
            "directed_parallel_groups": sum(count > 1 for count in direction_counts.values()),
            "directed_parallel_records": sum(count for count in direction_counts.values() if count > 1),
            "undirected_parallel_groups": sum(count > 1 for count in undirected_counts.values()),
            "undirected_parallel_records": sum(count for count in undirected_counts.values() if count > 1),
        },
        "weak_components": components,
        "parallel_record_groups": parallel_groups,
        "parallel_records_in_groups": parallel_records,
        "self_loop_records": sum(edge["from"] == edge["to"] for edge in links),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--family", choices=("correctness", "minimal", "sparse", "hub", "disconnected", "dense"), required=True)
    parser.add_argument("--nodes", type=int, default=1_000)
    parser.add_argument("--minimal-count", type=int, choices=(0, 1, 2), default=0)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.family == "correctness":
        case = correctness_case()
    elif args.family == "minimal":
        case = minimal_case(args.minimal_count)
    else:
        case = workload_case(args.family, args.nodes)
    if len(case["links"]) > MAX_EDGES:
        raise SystemExit(f"fixture exceeds edge cap ({MAX_EDGES})")
    summary = summarize(case, args.output)
    print(json.dumps({"schema_version": 1, "family": args.family, "seed": "uuid5-fixed-namespace-v1", **summary}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
