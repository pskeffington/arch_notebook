#!/usr/bin/env python3
"""Check Arch Notebook's supported ANX export/import round trip on a fictional chart."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import tempfile
import xml.etree.ElementTree as ET
from collections import Counter, defaultdict
from pathlib import Path


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def run(binary: Path, label: str, *arguments: str) -> dict:
    process = subprocess.run(
        [str(binary), *arguments], text=True, capture_output=True, check=False
    )
    if process.returncode != 0:
        raise RuntimeError(
            f"{label} failed with {process.returncode}: {process.stderr or process.stdout}"
        )
    try:
        return json.loads(process.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"{label} did not return JSON: {error}") from error


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--fixture", required=True, type=Path)
    parser.add_argument("--work-dir", type=Path, help="retain artifacts in this new directory")
    args = parser.parse_args()
    binary = args.binary.resolve()
    fixture = args.fixture.resolve()
    if not binary.is_file():
        parser.error(f"Arch Notebook binary does not exist: {binary}")
    if not fixture.is_file():
        parser.error(f"ANX fixture does not exist: {fixture}")

    temporary = None
    if args.work_dir:
        work = args.work_dir.resolve()
        work.mkdir(parents=True, exist_ok=False)
    else:
        temporary = tempfile.TemporaryDirectory(prefix="arch-notebook-anx-roundtrip-")
        work = Path(temporary.name)

    source = json.loads(fixture.read_text(encoding="utf-8"))
    source_case = work / "source.case.json"
    destination_case = work / "destination.case.json"
    anx_file = work / "roundtrip.anx"
    source_case.write_text(json.dumps(source, indent=2) + "\n", encoding="utf-8")
    destination_case.write_text(
        json.dumps(
            {
                "format_version": source.get("format_version", 19),
                "title": "FICTIONAL ANX ROUND-TRIP ACCEPTANCE",
                "entities": [],
                "links": [],
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    exported = run(
        binary, "anx_export", "--export-anx", str(source_case), str(anx_file), "--json"
    )
    if not exported.get("ok") or exported.get("encoding") != "UTF-16LE with BOM":
        raise RuntimeError(f"ANX export metadata is unexpected: {exported}")
    anx_bytes = anx_file.read_bytes()
    if not anx_bytes.startswith(b"\xff\xfe"):
        raise RuntimeError("ANX export is missing its UTF-16LE byte-order mark")
    root = ET.fromstring(anx_bytes.decode("utf-16"))
    if root.tag != "Chart" or root.find("ChartItemCollection") is None:
        raise RuntimeError("ANX export is missing Chart/ChartItemCollection")

    imported = run(
        binary,
        "anx_import",
        "--import-anx",
        str(destination_case),
        str(anx_file),
        "--json",
    )
    if not imported.get("ok") or not imported.get("saved"):
        raise RuntimeError(f"ANX import result is unexpected: {imported}")
    destination = json.loads(destination_case.read_text(encoding="utf-8"))

    source_entities = {entity["id"]: entity for entity in source["entities"]}
    destination_entities = {entity["id"]: entity for entity in destination["entities"]}
    if source_entities.keys() != destination_entities.keys():
        raise RuntimeError("entity UUIDs changed or were lost during ANX round trip")
    entity_fields = (
        "name",
        "kind",
        "representation",
        "detail",
        "attributes",
        "aliases",
        "external_ids",
        "merged_records",
        "position_locked",
        "x",
        "y",
        "color",
    )
    for entity_id, expected in source_entities.items():
        actual = destination_entities[entity_id]
        for field in entity_fields:
            if actual.get(field) != expected.get(field):
                raise RuntimeError(
                    f"entity {entity_id} field {field} changed: "
                    f"{expected.get(field)!r} -> {actual.get(field)!r}"
                )
    if destination.get("active_chart_title") != source.get("active_chart_title"):
        raise RuntimeError("chart title changed during ANX round trip")

    source_links = source["links"]
    destination_links = destination["links"]
    if len(source_links) != len(destination_links):
        raise RuntimeError(
            f"relationship count changed: {len(source_links)} -> {len(destination_links)}"
        )
    link_fields = (
        "from",
        "to",
        "label",
        "date",
        "confidence",
        "directed",
        "notes",
        "source_notes",
        "source_reference",
        "attributes",
    )
    for index, (expected, actual) in enumerate(zip(source_links, destination_links, strict=True)):
        expected_connection_id = expected.get("connection_id", "")
        actual_connection_id = actual.get("connection_id", "")
        if expected_connection_id and actual_connection_id != expected_connection_id:
            raise RuntimeError(
                f"relationship {index} connection_id changed: "
                f"{expected_connection_id!r} -> {actual_connection_id!r}"
            )
        if not expected_connection_id and not actual_connection_id:
            raise RuntimeError(f"anonymous relationship {index} did not receive a stable ID")
        for field in link_fields:
            if actual.get(field) != expected.get(field):
                raise RuntimeError(
                    f"relationship {index} field {field} changed: "
                    f"{expected.get(field)!r} -> {actual.get(field)!r}"
                )

    repeated_ids = {
        connection_id
        for connection_id, count in Counter(
            link["connection_id"] for link in source_links if link.get("connection_id")
        ).items()
        if count > 1
    }
    scoped_sources: dict[str, set[str]] = defaultdict(set)
    for link in destination_links:
        if link.get("connection_id") in repeated_ids:
            source_scope = link.get("ingest_source")
            if not source_scope:
                raise RuntimeError("repeated ANX connection ID has no source scope")
            scoped_sources[link["connection_id"]].add(source_scope)
    for connection_id in repeated_ids:
        expected_count = sum(link["connection_id"] == connection_id for link in source_links)
        if len(scoped_sources[connection_id]) != expected_count:
            raise RuntimeError(
                f"repeated connection ID {connection_id!r} did not receive distinct record scopes"
            )

    source_labels = source.get("labels", [])
    destination_labels = destination.get("labels", [])
    if len(source_labels) != len(destination_labels):
        raise RuntimeError("standalone chart labels were lost during ANX round trip")
    label_fields = (
        "id",
        "text",
        "x",
        "y",
        "font_size",
        "color",
        "is_legend",
        "legend_entries",
        "legend_entries_truncated",
    )
    for index, (expected, actual) in enumerate(
        zip(source_labels, destination_labels, strict=True)
    ):
        for field in label_fields:
            if actual.get(field) != expected.get(field):
                raise RuntimeError(
                    f"chart label {index} field {field} changed: "
                    f"{expected.get(field)!r} -> {actual.get(field)!r}"
                )

    malformed_anx = work / "unsupported-shape.anx"
    malformed_xml = (
        "<?xml version='1.0' encoding='utf-16'?>"
        "<Chart><ChartItemCollection><ChartItem Id='UNSUPPORTED-1' Label='Unsupported item'>"
        "<End X='0' Y='0'><Entity EntityId='1' Identity='UNSUPPORTED-1' LabelIsIdentity='false'>"
        "<UnsupportedShape /></Entity></End></ChartItem></ChartItemCollection></Chart>"
    )
    malformed_anx.write_bytes(malformed_xml.encode("utf-16"))
    failure_case = work / "failure.case.json"
    failure_case.write_text(json.dumps(source, indent=2) + "\n", encoding="utf-8")
    failure_case_before = failure_case.read_bytes()
    rejected = subprocess.run(
        [str(binary), "--import-anx", str(failure_case), str(malformed_anx), "--json"],
        text=True,
        capture_output=True,
        check=False,
    )
    if rejected.returncode == 0:
        raise RuntimeError("ANX importer silently accepted an unsupported entity shape")
    try:
        rejection_report = json.loads(rejected.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"rejected ANX import did not return JSON: {error}") from error
    if "unsupported or missing shape" not in rejection_report.get("error", ""):
        raise RuntimeError(f"unsupported shape failed for an unexpected reason: {rejection_report}")
    if failure_case.read_bytes() != failure_case_before:
        raise RuntimeError("failed ANX import changed the preexisting case file")

    report = {
        "schema_version": 1,
        "result": "PASS",
        "fixture": fixture.name,
        "fixture_sha256": sha256(fixture),
        "anx_sha256": sha256(anx_file),
        "encoding": "UTF-16LE with BOM",
        "entities": len(destination["entities"]),
        "relationships": len(destination_links),
        "relationships_with_preserved_repeated_ids": len(repeated_ids),
        "labels": len(destination_labels),
        "unsupported_shape_rejected": True,
        "failed_import_case_unchanged": True,
        "local_sql_source_keys_included": exported.get("local_sql_source_keys_included"),
        "commands": [
            {"label": "anx_export", "exit_code": 0},
            {"label": "anx_import", "exit_code": 0},
            {"label": "unsupported_shape_import", "exit_code": rejected.returncode},
        ],
        "ibm_schema_or_application_validation": "NOT PERFORMED",
        "work_dir": str(work),
    }
    (work / "acceptance-report.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(json.dumps(report, indent=2, sort_keys=True))
    if temporary is not None:
        temporary.cleanup()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
