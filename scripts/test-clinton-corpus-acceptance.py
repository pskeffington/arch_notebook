#!/usr/bin/env python3
"""Run privacy-preserving, read-only QA against the local Clinton corpus case."""

from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tempfile


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def run_json(binary: Path, *arguments: str) -> tuple[int, dict[str, object]]:
    process = subprocess.run(
        [str(binary), *arguments],
        capture_output=True,
        text=True,
        check=False,
    )
    try:
        payload = json.loads(process.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError("Arch Notebook did not return valid JSON") from error
    if not isinstance(payload, dict):
        raise RuntimeError("Arch Notebook returned an unexpected JSON shape")
    return process.returncode, payload


def packet_counts(markdown: str) -> dict[str, int]:
    communities = 0
    members = 0
    interactions = 0
    proxies = 0
    in_proxy_table = False
    for line in markdown.splitlines():
        if line.startswith("- **Community "):
            communities += 1
            match = re.search(r" · (\d+) entit(?:y|ies):", line)
            if match:
                members += int(match.group(1))
        elif line.startswith("| Community ") and " ↔ " in line:
            interactions += 1
        elif line.startswith("### Cross-community proxies"):
            in_proxy_table = True
        elif in_proxy_table and line.startswith("### "):
            in_proxy_table = False
        elif in_proxy_table and re.match(r"^\|\s*\d+\s*\|", line):
            proxies += 1
    return {
        "detected_communities": communities,
        "community_member_entities": members,
        "cross_community_contact_rows": interactions,
        "cross_community_proxy_candidates": proxies,
    }


def packet_geometry(svg: str) -> dict[str, int]:
    centers: list[tuple[float, float]] = []
    labels_outside = 0
    for match in re.finditer(r'<g data-entity="[^"]+">(.*?)</g>', svg, re.DOTALL):
        group = match.group(1)
        circle = re.search(r'<circle cx="([\d.-]+)" cy="([\d.-]+)"', group)
        label = re.search(
            r'<rect x="([\d.-]+)" y="([\d.-]+)" width="([\d.-]+)" height="([\d.-]+)"',
            group,
        )
        if circle is None or label is None:
            continue
        centers.append((float(circle.group(1)), float(circle.group(2))))
        left, top, width, height = (float(value) for value in label.groups())
        if left < 0 or top < 0 or left + width > 1200 or top + height > 800:
            labels_outside += 1
    same_y = Counter(round(y, 2) for _, y in centers)
    return {
        "rendered_entity_centers": len(centers),
        "unique_rendered_y_rows": len(same_y),
        "largest_same_y_row": max(same_y.values(), default=0),
        "labels_outside_svg_viewport": labels_outside,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--pdf", required=True, type=Path, help="source PDF from the local TRANSCRIBE share")
    parser.add_argument("--case", required=True, type=Path, help="review-required Arch Notebook candidate case")
    parser.add_argument("--database", required=True, type=Path, help="read-only source SQLite database")
    parser.add_argument("--report", required=True, type=Path, help="write aggregate-only QA JSON here")
    args = parser.parse_args()

    binary, pdf, case, database = (path.resolve(strict=True) for path in (args.binary, args.pdf, args.case, args.database))
    if not os.access(binary, os.X_OK):
        parser.error(f"binary is not executable: {binary}")
    for label, path in (("PDF", pdf), ("case", case), ("SQLite database", database)):
        if not path.is_file():
            parser.error(f"{label} is not a file: {path}")

    input_hashes_before = {
        "pdf_sha256": sha256(pdf),
        "case_sha256": sha256(case),
        "database_sha256": sha256(database),
    }
    case_data = json.loads(case.read_text(encoding="utf-8"))
    jobs = case_data.get("automation_jobs", [])
    if not jobs or not isinstance(jobs[0], dict) or not jobs[0].get("id"):
        raise RuntimeError("candidate case has no saved SQL job to validate")

    run_exit, run_listing = run_json(binary, "--list-runs", str(case), "--json")
    batch_exit, batch_listing = run_json(binary, "--list-batches", str(case), "--json")
    if run_exit != 0 or batch_exit != 0:
        raise RuntimeError("case run or batch history could not be read")
    runs = run_listing.get("runs")
    batches = batch_listing.get("batches")
    if not isinstance(runs, list) or not runs or not isinstance(batches, list) or not batches:
        raise RuntimeError("candidate case has no retained SQL run and batch history")
    latest_run = max(
        (run for run in runs if isinstance(run, dict)),
        key=lambda run: str(run.get("finished_at") or run.get("started_at") or ""),
    )
    if not latest_run.get("run_id"):
        raise RuntimeError("latest SQL run has no replay identity")
    if any(not isinstance(run, dict) or run.get("status") != "PASS" for run in runs):
        raise RuntimeError("candidate run history contains a non-passing SQL run")
    if any(not isinstance(batch, dict) or batch.get("status") != "COMPLETE" for batch in batches):
        raise RuntimeError("candidate batch history contains an incomplete SQL batch")

    snapshot_rows = [
        len(snapshot.get("connections", []))
        for snapshot in case_data.get("snapshots", [])
        if isinstance(snapshot, dict)
    ]
    entity_count = len(case_data.get("entities", []))
    relationship_count = len(case_data.get("links", []))
    positions = [
        (float(entity.get("x", 0.0)), float(entity.get("y", 0.0)))
        for entity in case_data.get("entities", [])
        if isinstance(entity, dict)
    ]
    out_of_range_positions = sum(
        not (0.0 <= x <= 1.0 and 0.0 <= y <= 1.0) for x, y in positions
    )
    if not snapshot_rows or any(rows != relationship_count for rows in snapshot_rows):
        raise RuntimeError("saved SQL snapshots do not reconcile with the saved chart relationships")

    job_exit, job_result = run_json(
        binary,
        "--validate-job",
        str(case),
        str(jobs[0]["id"]),
        "--query-only",
        "--json",
    )
    if (
        job_exit != 0
        or job_result.get("ok") is not True
        or job_result.get("saved") is not False
        or int(job_result.get("passed", 0)) < 1
        or int(job_result.get("failed", 0)) != 0
    ):
        raise RuntimeError("query-only validation of the saved SQL job failed")

    replay_exit, replay = run_json(
        binary,
        "--validate-run",
        str(case),
        str(latest_run["run_id"]),
        "--json",
    )
    outcomes = replay.get("outcomes")
    if replay_exit != 0 or replay.get("ok") is not True or replay.get("saved") is not False:
        raise RuntimeError("latest saved SQL run could not be replay-validated read-only")
    if not isinstance(outcomes, list) or len(outcomes) != 1 or not isinstance(outcomes[0], dict):
        raise RuntimeError("saved-run replay returned an unexpected result shape")
    outcome = outcomes[0]
    delta = outcome.get("delta")
    delta_counts = delta.get("counts", {}) if isinstance(delta, dict) else {}
    if (
        outcome.get("status") != "PASS"
        or int(outcome.get("connection_count", -1)) != relationship_count
        or int(outcome.get("skipped_rows", -1)) != 0
        or any(int(delta_counts.get(name, -1)) != 0 for name in ("added", "removed", "changed_records"))
    ):
        raise RuntimeError("saved-run replay did not match the current source without skipped rows")

    with tempfile.TemporaryDirectory(prefix="arch-notebook-clinton-primary-qa-") as temporary:
        packet_dir = Path(temporary) / "redacted-analysis-packet"
        packet_exit, packet = run_json(
            binary,
            "--export-analysis-packet",
            str(case),
            str(packet_dir),
            "--json",
        )
        if packet_exit != 0 or packet.get("ok") is not True:
            raise RuntimeError("default-redacted network analysis packet export failed")
        if int(packet.get("entities", -1)) != entity_count or int(packet.get("relationships", -1)) != relationship_count:
            raise RuntimeError("redacted analysis packet does not reconcile with the saved chart")
        options = packet.get("options")
        if not isinstance(options, dict) or any(options.values()):
            raise RuntimeError("analysis packet unexpectedly disclosed optional source fields")
        report_path = packet_dir / "finding.md"
        svg_path = packet_dir / "network.svg"
        report_text = report_path.read_text(encoding="utf-8")
        if "Chart: Redacted" not in report_text or not svg_path.is_file():
            raise RuntimeError("analysis packet did not preserve default chart redaction")
        svg_text = svg_path.read_text(encoding="utf-8")
        geometry = packet_geometry(svg_text)
        if (
            geometry["rendered_entity_centers"] != entity_count
            or geometry["labels_outside_svg_viewport"] != 0
            or geometry["largest_same_y_row"] > max(64, int(entity_count * 0.10))
        ):
            raise RuntimeError("analysis packet layout clips labels or collapses most entities onto one row")
        packet_summary = {
            "entities": int(packet.get("entities", -1)),
            "relationship_records": int(packet.get("relationships", -1)),
            **geometry,
            "saved_entities_with_out_of_range_positions": out_of_range_positions,
            **packet_counts(report_text),
            "names_relationship_labels_ids_evidence_references_included": False,
            "output_permissions": {
                "directory": oct(stat.S_IMODE(packet_dir.stat().st_mode)),
                "report": oct(stat.S_IMODE(report_path.stat().st_mode)),
                "svg": oct(stat.S_IMODE(svg_path.stat().st_mode)),
            },
        }
        if packet_summary["community_member_entities"] != packet_summary["entities"]:
            raise RuntimeError("redacted community membership does not cover every exported entity")

    input_hashes_after = {
        "pdf_sha256": sha256(pdf),
        "case_sha256": sha256(case),
        "database_sha256": sha256(database),
    }
    if input_hashes_after != input_hashes_before:
        raise RuntimeError("PDF, saved case, or SQLite source changed during read-only QA")

    snapshots = case_data.get("snapshots", [])
    report = {
        "schema_version": 1,
        "suite": "arch_notebook_clinton_corpus_primary_qa",
        "result": "PASS",
        "binary_sha256": sha256(binary),
        **input_hashes_before,
        "read_only_inputs_unchanged": True,
        "saved_case": {
            "entities": entity_count,
            "relationship_records": relationship_count,
            "snapshots": len(snapshots),
            "snapshot_relationship_rows": snapshot_rows,
            "sql_runs": len(runs),
            "sql_batches": len(batches),
            "sql_run_statuses": sorted(str(run.get("status")) for run in runs if isinstance(run, dict)),
            "sql_batch_statuses": sorted(str(batch.get("status")) for batch in batches if isinstance(batch, dict)),
        },
        "automation": {
            "query_only_preflight_exit_code": job_exit,
            "query_only_jobs_passed": int(job_result.get("passed", 0)),
            "query_only_jobs_failed": int(job_result.get("failed", 0)),
            "latest_saved_run_replay_exit_code": replay_exit,
            "latest_saved_run_status": outcome.get("status"),
            "latest_saved_run_relationship_rows": int(outcome.get("connection_count", 0)),
            "latest_saved_run_skipped_rows": int(outcome.get("skipped_rows", 0)),
            "latest_saved_run_delta_counts": {
                name: int(delta_counts.get(name, 0)) for name in ("added", "removed", "changed_records")
            },
        },
        "default_redacted_analysis_packet": packet_summary,
        "qwen_analysis": "NOT RUN by this local acceptance",
        "human_content_review": "NOT RUN by this local acceptance",
        "pdf_extraction_completeness": "NOT ESTABLISHED by this local acceptance",
    }
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
