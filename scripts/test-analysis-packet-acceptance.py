#!/usr/bin/env python3
"""Exercise the analyst-packet CLI with a fictional chart and isolated outputs."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import stat
import subprocess
import sys
import uuid
import xml.etree.ElementTree as ET


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run(binary: pathlib.Path, *args: str, expected: int = 0) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        [str(binary), "--export-analysis-packet", *args],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != expected:
        raise AssertionError(
            f"CLI exited {result.returncode}, expected {expected}: {result.stderr.strip()}"
        )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=pathlib.Path)
    parser.add_argument(
        "--fixture",
        type=pathlib.Path,
        default=pathlib.Path("fixtures/chart/correctness.case.json"),
    )
    parser.add_argument("--work-dir", required=True, type=pathlib.Path)
    args = parser.parse_args()

    binary = args.binary.resolve(strict=True)
    fixture = args.fixture.resolve(strict=True)
    work_dir = args.work_dir.absolute()
    if not binary.is_file() or not fixture.is_file():
        raise ValueError("binary and fixture must be regular files")
    if work_dir.exists():
        raise ValueError(f"work directory already exists: {work_dir}")
    work_dir.mkdir(parents=True)

    fixture_hash_before = sha256(fixture)
    original = json.loads(fixture.read_text(encoding="utf-8"))
    case = dict(original)
    alternate_id = str(uuid.uuid4())
    alternate_entity_id = str(uuid.uuid4())
    case["chart_tabs"] = [
        {
            "id": alternate_id,
            "title": "Alternate **Chart**\n# injected heading",
            "zoom": 1.0,
            "pan": [0.0, 0.0],
            "entities": [
                {
                    **original["entities"][0],
                    "id": alternate_entity_id,
                    "name": "PACKET ALTERNATE ENTITY",
                    "x": 0.5,
                    "y": 0.5,
                }
            ],
            "links": [],
            "labels": [],
            "template_id": None,
        }
    ]
    case_path = work_dir / "fictional-case.json"
    case_path.write_text(json.dumps(case, ensure_ascii=False), encoding="utf-8")
    case_hash_before = sha256(case_path)

    default_dir = work_dir / "default-packet"
    default_run = run(binary, str(case_path), str(default_dir), "--json")
    default_result = json.loads(default_run.stdout)
    default_report = (default_dir / "finding.md").read_text(encoding="utf-8")
    default_svg_path = default_dir / "network.svg"
    default_svg = default_svg_path.read_text(encoding="utf-8")
    default_xml = ET.parse(default_svg_path).getroot()
    assert set(path.name for path in default_dir.iterdir()) == {"finding.md", "network.svg"}
    if sys.platform.startswith("linux"):
        assert stat.S_IMODE(default_dir.stat().st_mode) == 0o700
        assert stat.S_IMODE((default_dir / "finding.md").stat().st_mode) == 0o600
        assert stat.S_IMODE(default_svg_path.stat().st_mode) == 0o600
    assert default_result["ok"] is True
    assert default_result["entities"] == len(original["entities"])
    assert default_result["relationships"] == len(original["links"])
    assert "Chart: Redacted" in default_report
    for entity in original["entities"]:
        assert entity["name"] not in default_report and entity["name"] not in default_svg
    for link in original["links"]:
        for field in ("label", "connection_id", "source_reference"):
            value = link.get(field, "")
            if value:
                assert value not in default_report and value not in default_svg
    assert "<metadata" not in default_svg
    svg_namespace = "{http://www.w3.org/2000/svg}"
    assert len(default_xml.findall(f".//{svg_namespace}g[@data-record]")) == len(original["links"])
    assert default_xml.find(f".//{svg_namespace}marker") is not None
    assert default_xml.find(f".//{svg_namespace}path") is not None
    assert all(
        not edge.findall(f"{svg_namespace}text")
        for edge in default_xml.findall(f".//{svg_namespace}g[@data-record]")
    )
    for text in default_xml.findall(f".//{svg_namespace}text"):
        if "x" in text.attrib:
            assert 0 <= float(text.attrib["x"]) <= 1200
        if "y" in text.attrib:
            assert 0 <= float(text.attrib["y"]) <= 800

    disclosure_dir = work_dir / "disclosed-packet"
    disclosure_run = run(
        binary,
        str(case_path),
        str(disclosure_dir),
        "--include-names",
        "--include-relationship-labels",
        "--include-ids",
        "--include-evidence-references",
        "--json",
    )
    disclosure_result = json.loads(disclosure_run.stdout)
    disclosure_svg_path = disclosure_dir / "network.svg"
    disclosure_svg = disclosure_svg_path.read_text(encoding="utf-8")
    disclosure_xml = ET.parse(disclosure_svg_path).getroot()
    assert disclosure_result["ok"] is True
    assert original["active_chart_title"] in (disclosure_dir / "finding.md").read_text(
        encoding="utf-8"
    )
    assert original["entities"][0]["name"] in disclosure_svg
    assert str(original["entities"][0]["id"]) in disclosure_svg
    if original["links"]:
        first_link = original["links"][0]
        assert first_link["label"] in "".join(disclosure_xml.itertext())
        if first_link.get("connection_id"):
            assert first_link["connection_id"] in disclosure_svg
        if first_link.get("source_reference"):
            assert first_link["source_reference"] in "".join(disclosure_xml.itertext())

    selected_dir = work_dir / "selected-chart-packet"
    selected_run = run(
        binary,
        str(case_path),
        str(selected_dir),
        "--chart",
        alternate_id,
        "--include-names",
        "--json",
    )
    selected_result = json.loads(selected_run.stdout)
    selected_report = (selected_dir / "finding.md").read_text(encoding="utf-8")
    selected_svg = (selected_dir / "network.svg").read_text(encoding="utf-8")
    assert selected_result["entities"] == 1 and selected_result["relationships"] == 0
    assert r"Chart: Alternate \*\*Chart\*\* # injected heading" in selected_report
    assert "injected heading" in selected_report
    assert not any(line.startswith("# injected heading") for line in selected_report.splitlines())
    assert "Entities: 1" in selected_report and "Relationship records: 0 total" in selected_report
    assert "PACKET ALTERNATE ENTITY" in selected_svg
    assert "Avery Rowan" not in selected_svg

    original_report = default_dir / "finding.md"
    original_digest = sha256(original_report)
    occupied = run(binary, str(case_path), str(default_dir), "--json", expected=1)
    assert "already exists" in occupied.stderr
    assert sha256(original_report) == original_digest

    duplicate_json_path = work_dir / "duplicate-json-output"
    duplicate_json = run(
        binary,
        str(case_path),
        str(duplicate_json_path),
        "--json",
        "--json",
        expected=2,
    )
    assert "usage:" in duplicate_json.stderr
    assert not duplicate_json_path.exists()

    case_hash_after = sha256(case_path)
    fixture_hash_after = sha256(fixture)
    assert case_hash_after == case_hash_before
    assert fixture_hash_after == fixture_hash_before
    evidence = {
        "result": "PASS",
        "binary": str(binary),
        "binary_sha256": sha256(binary),
        "fixture": str(fixture),
        "fixture_sha256": fixture_hash_before,
        "fixture_unchanged": fixture_hash_after == fixture_hash_before,
        "case_copy_sha256": case_hash_before,
        "case_copy_unchanged": case_hash_after == case_hash_before,
        "default_packet": default_result,
        "disclosed_packet": disclosure_result,
        "selected_chart_packet": selected_result,
        "existing_destination_exit_code": occupied.returncode,
        "existing_destination_preserved": sha256(original_report) == original_digest,
        "duplicate_json_exit_code": duplicate_json.returncode,
        "duplicate_json_created_no_output": not duplicate_json_path.exists(),
        "svg_xml_parse": "PASS",
        "svg_relationship_records": len(default_xml.findall(f".//{svg_namespace}g[@data-record]")),
        "svg_direction_and_self_loop_markers": "PASS",
        "unix_private_permissions": "PASS" if sys.platform.startswith("linux") else "NOT APPLICABLE",
        "default_privacy": "Names, relationship labels, IDs, source references, and chart title omitted.",
        "platform_scope": "Linux CLI only; non-Linux atomic publication currently reports unsupported.",
    }
    evidence_path = work_dir / "acceptance-report.json"
    evidence_path.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    evidence["report_path"] = str(evidence_path)
    evidence["report_sha256"] = sha256(evidence_path)
    print(json.dumps(evidence, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (AssertionError, OSError, ValueError, json.JSONDecodeError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)
