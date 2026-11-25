#!/usr/bin/env python3
"""Run a disposable SQL automation acceptance flow against the fictional fixture."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import shutil
import sqlite3
import subprocess
import tempfile
from pathlib import Path


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--source-sql", required=True, type=Path)
    parser.add_argument("--work-dir", type=Path, help="retain all acceptance artifacts in this new directory")
    args = parser.parse_args()
    binary = args.binary.resolve()
    source_sql = args.source_sql.resolve()
    if not binary.is_file():
        parser.error(f"Arch Notebook binary does not exist: {binary}")
    if not source_sql.is_file():
        parser.error(f"fixture SQL does not exist: {source_sql}")

    if args.work_dir:
        work = args.work_dir.resolve()
        work.mkdir(parents=True, exist_ok=False)
        temporary = None
    else:
        temporary = tempfile.TemporaryDirectory(prefix="arch-notebook-sql-acceptance-")
        work = Path(temporary.name)

    database = work / "sample.sqlite"
    connection = sqlite3.connect(database)
    try:
        connection.executescript(source_sql.read_text(encoding="utf-8"))
        connection.commit()
    finally:
        connection.close()
    source_hash_before = sha256(database)

    query = """SELECT connection_id, source, source_type, target, target_type, relationship,
                      event_date AS date, confidence, directed, source_reference, notes
               FROM connection_observations WHERE snapshot_id = :snapshot_id
               ORDER BY connection_id"""
    case_path = work / "automation.case.json"
    case_path.write_text(
        json.dumps(
            {
                "format_version": 19,
                "title": "FICTIONAL SQL AUTOMATION ACCEPTANCE",
                "description": "Synthetic acceptance case; no operational data.",
                "entities": [],
                "links": [],
                "automation_jobs": [
                    {
                        "id": "7f2acbca-3263-4cda-b8e8-c3a586e8c4c1",
                        "name": "Synthetic connection capture",
                        "database_path": str(database),
                        "series": "synthetic-periods",
                        "query": query,
                        "parameters": [["snapshot_id", "text:baseline"]],
                        "query_timeout_minutes": 15,
                        "enabled": True,
                        "minimum_connections": 1,
                        "minimum_confidence": None,
                        "maximum_connections": None,
                        "maximum_regressions": None,
                        "maximum_added_connections": None,
                        "maximum_connection_property_changes": None,
                        "maximum_entity_property_changes": None,
                        "maximum_skipped_rows": 0,
                        "require_unchanged_schema": False,
                    }
                ],
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    commands: list[dict] = []

    def run(label: str, *arguments: str) -> dict:
        command = [str(binary), *arguments]
        process = subprocess.run(command, text=True, capture_output=True, check=False)
        record = {
            "label": label,
            "command": command,
            "exit_code": process.returncode,
            "stdout": process.stdout,
            "stderr": process.stderr,
        }
        commands.append(record)
        if process.returncode != 0:
            raise RuntimeError(f"{label} failed with {process.returncode}: {process.stderr}")
        try:
            return json.loads(process.stdout)
        except json.JSONDecodeError as error:
            raise RuntimeError(f"{label} did not return JSON: {error}") from error

    original_case_bytes = case_path.read_bytes()
    preflight = run("query_preflight", "--validate-enabled-jobs", str(case_path), "--json")
    if not preflight.get("ok") or preflight.get("failed") != 0:
        raise RuntimeError(f"query-only preflight failed: {preflight}")
    if case_path.read_bytes() != original_case_bytes:
        raise RuntimeError("query-only preflight changed the case file")

    baseline = run("baseline_batch", "--run-enabled-jobs", str(case_path), "--json")
    current = run(
        "current_override_batch",
        "--run-enabled-jobs",
        str(case_path),
        "--job-param",
        "Synthetic connection capture",
        "snapshot_id=text:current",
        "--json",
    )
    case = json.loads(case_path.read_text(encoding="utf-8"))
    snapshots = case.get("snapshots", [])
    runs = case.get("automation_runs", [])
    jobs = case.get("automation_jobs", [])
    if len(snapshots) != 2 or [len(snapshot["connections"]) for snapshot in snapshots] != [4, 4]:
        raise RuntimeError("expected two four-record snapshots")
    if len(runs) != 2 or any(not run_record.get("success") for run_record in runs):
        raise RuntimeError("expected two successful saved automation runs")
    if any(run_record.get("trigger") != "cli" for run_record in runs):
        raise RuntimeError("expected CLI-triggered automation history")
    if jobs[0].get("parameters") != [["snapshot_id", "text:baseline"]]:
        raise RuntimeError("one-run parameter override changed the saved job")
    if not baseline.get("ok") or not current.get("ok"):
        raise RuntimeError("automation batch returned a non-ok result")

    history = run("run_history", "--list-runs", str(case_path), "--json")
    history_runs = history.get("runs", [])
    if len(history_runs) != 2:
        raise RuntimeError("expected both automation records in run history")
    current_run = next((entry for entry in history_runs if entry.get("snapshot_id") == snapshots[1]["id"]), None)
    if current_run is None:
        raise RuntimeError("current snapshot is missing from run history")
    change_kinds = {
        kind
        for change in current_run.get("delta", {}).get("changes", [])
        for kind in change.get("kinds", [])
    }
    required_changes = {"removed", "confidence_declined", "source_notes_changed", "added"}
    if not required_changes.issubset(change_kinds):
        raise RuntimeError(f"missing expected snapshot deltas: {required_changes - change_kinds}")

    batch_listing = run("batch_history", "--list-batches", str(case_path), "--json")
    if batch_listing.get("total") != 2:
        raise RuntimeError("expected two persisted CLI batch checkpoints")

    comparison = work / "comparison.csv"
    run(
        "comparison_export",
        "--export-sql-comparison",
        str(case_path),
        snapshots[0]["id"],
        snapshots[1]["id"],
        str(comparison),
        "--json",
    )
    with comparison.open(newline="", encoding="utf-8") as source:
        comparison_rows = list(csv.DictReader(source))
    exported_changes = {(row["change"], row["connection_id"]) for row in comparison_rows}
    if not {
        ("REMOVED", "C-104"),
        ("CONFIDENCE DOWN · SOURCE NOTES CHANGED", "C-102"),
        ("ADDED", "C-105"),
    }.issubset(exported_changes):
        raise RuntimeError(f"comparison export is missing expected rows: {exported_changes}")

    source_hash_after = sha256(database)
    if source_hash_before != source_hash_after:
        raise RuntimeError("configured SQLite source changed during read-only automation")

    report = {
        "schema_version": 1,
        "result": "PASS",
        "fixture": source_sql.name,
        "binary": str(binary),
        "work_dir": str(work),
        "database_sha256_before": source_hash_before,
        "database_sha256_after": source_hash_after,
        "source_unchanged": True,
        "query_preflight_saved_case": False,
        "snapshot_counts": [len(snapshot["connections"]) for snapshot in snapshots],
        "snapshot_ids": [snapshot["id"] for snapshot in snapshots],
        "run_ids": [run_record["id"] for run_record in runs],
        "batch_ids": [run_record["batch_id"] for run_record in runs],
        "saved_job_parameters_after_one_run_override": jobs[0]["parameters"],
        "expected_delta_kinds": sorted(required_changes),
        "comparison_rows": len(comparison_rows),
        "commands": [
            {"label": record["label"], "exit_code": record["exit_code"]}
            for record in commands
        ],
    }
    report_path = work / "acceptance-report.json"
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2, sort_keys=True))
    if temporary is not None:
        temporary.cleanup()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
