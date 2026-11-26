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
                      CASE source
                        WHEN 'Mara Venn' THEN 'PERSON-001'
                        WHEN 'Elias Ward' THEN 'PERSON-002'
                      END AS source_entity_id,
                      CASE target
                        WHEN 'North Quay Storage' THEN 'LOCATION-001'
                        WHEN 'Oriole Freight' THEN 'ORGANIZATION-001'
                        WHEN 'Eastbridge Depot' THEN 'LOCATION-002'
                      END AS target_entity_id,
                      event_date AS date, confidence, directed, source_reference, notes,
                      'SYNTHETIC REGISTRY' AS "source_attribute:registry",
                      'SYNTHETIC REGISTRY' AS "target_attribute:registry",
                      'SQL ACCEPTANCE FIXTURE' AS "connection_attribute:capture_method"
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

    def run(label: str, *arguments: str, expected_exit: int = 0) -> dict:
        command = [str(binary), *arguments]
        process = subprocess.run(command, text=True, capture_output=True, check=False)
        record = {
            "label": label,
            "command": command,
            "exit_code": process.returncode,
            "expected_exit_code": expected_exit,
            "stdout": process.stdout,
            "stderr": process.stderr,
        }
        commands.append(record)
        if process.returncode != expected_exit:
            raise RuntimeError(
                f"{label} exited {process.returncode}; expected {expected_exit}: {process.stderr}"
            )
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

    schema_editor = sqlite3.connect(database)
    try:
        schema_editor.execute(
            "ALTER TABLE connection_observations ADD COLUMN acceptance_probe TEXT"
        )
        schema_editor.commit()
    finally:
        schema_editor.close()
    schema_hash_before_preflight = sha256(database)
    case_hash_before_schema_preflight = sha256(case_path)
    case_before_schema_preflight = case_path.read_bytes()
    schema_preflight = run(
        "schema_drift_preflight",
        "--check-enabled-job-queries",
        str(case_path),
        "--require-unchanged-schema",
        "--json",
        expected_exit=1,
    )
    schema_hash_after_preflight = sha256(database)
    if schema_hash_before_preflight != schema_hash_after_preflight:
        raise RuntimeError("read-only schema-drift preflight changed the SQLite source")
    if case_path.read_bytes() != case_before_schema_preflight:
        raise RuntimeError("failed schema-drift preflight changed the case file")
    case_hash_after_schema_preflight = sha256(case_path)
    schema_outcomes = schema_preflight.get("outcomes", [])
    if schema_preflight.get("ok") or schema_preflight.get("failed") != 1:
        raise RuntimeError(f"schema-drift preflight did not fail as expected: {schema_preflight}")
    if schema_preflight.get("saved") is not False:
        raise RuntimeError("failed schema-drift preflight unexpectedly saved the case")
    if len(schema_outcomes) != 1:
        raise RuntimeError(f"expected one schema-drift preflight outcome: {schema_outcomes}")
    schema_outcome = schema_outcomes[0]
    if schema_outcome.get("status") != "FAIL":
        raise RuntimeError(f"schema-drift preflight outcome did not report FAIL: {schema_outcome}")
    if schema_outcome.get("schema_stability") != {
        "baseline_available": True,
        "changed": True,
        "passed": False,
        "required": True,
    }:
        raise RuntimeError(
            f"schema-drift preflight returned unexpected stability details: "
            f"{schema_outcome.get('schema_stability')}"
        )
    if "SCHEMA CHANGED SINCE BASELINE" not in schema_outcome.get("message", "").upper():
        raise RuntimeError(f"schema-drift preflight omitted the drift message: {schema_outcome}")

    failure_case_path = work / "failure.case.json"
    failure_case = json.loads(case_path.read_text(encoding="utf-8"))
    failure_case["automation_jobs"][0]["maximum_regressions"] = 0
    failure_case_path.write_text(
        json.dumps(failure_case, indent=2) + "\n", encoding="utf-8"
    )
    source_hash_before_failure_capture = sha256(database)
    failed_capture = run(
        "failed_capture_record",
        "--run-job",
        str(failure_case_path),
        "Synthetic connection capture",
        "--json",
        expected_exit=1,
    )
    source_hash_after_failure_capture = sha256(database)
    if source_hash_before_failure_capture != source_hash_after_failure_capture:
        raise RuntimeError("failed SQL capture changed the read-only SQLite source")
    failure_outcomes = failed_capture.get("outcomes", [])
    if failed_capture.get("ok") or failed_capture.get("saved") is not True:
        raise RuntimeError(f"failed SQL capture was not saved as expected: {failed_capture}")
    if failed_capture.get("failed") != 1 or len(failure_outcomes) != 1:
        raise RuntimeError(f"failed SQL capture returned unexpected outcomes: {failed_capture}")
    failed_outcome = failure_outcomes[0]
    if failed_outcome.get("status") != "FAIL" or not failed_outcome.get("snapshot_id"):
        raise RuntimeError(f"failed capture did not retain its snapshot and failure: {failed_outcome}")
    if "REGRESSION LIMIT EXCEEDED" not in failed_outcome.get("message", "").upper():
        raise RuntimeError(f"failed capture omitted the expected regression reason: {failed_outcome}")
    failed_case = json.loads(failure_case_path.read_text(encoding="utf-8"))
    failed_runs = failed_case.get("automation_runs", [])
    if len(failed_runs) != 3 or failed_runs[-1].get("success") is not False:
        raise RuntimeError("atomic case save did not retain the failing automation run")
    if failed_runs[-1].get("snapshot_id") != failed_outcome["snapshot_id"]:
        raise RuntimeError("failing run history does not reference its saved capture")
    failed_history = run("failure_run_history", "--list-runs", str(failure_case_path), "--json")
    failed_history_runs = failed_history.get("runs", [])
    if len(failed_history_runs) != 3 or not any(
        record.get("run_id") == failed_runs[-1].get("id")
        and record.get("status") == "FAIL"
        for record in failed_history_runs
    ):
        raise RuntimeError("run-history CLI did not retain the failed capture record")

    chart_roundtrips = []
    canonical_roundtrip_rows: dict[str, list[tuple[str, ...]]] = {}
    source_hash_before_chart_exchange = sha256(database)
    source_case_hash_before_chart_export = sha256(case_path)
    for extension, delimiter, format_name in (("csv", ",", "CSV"), ("tsv", "\t", "TSV")):
        chart_file = work / f"chart-import.{extension}"
        exported_chart = run(
            f"chart_{extension}_export",
            "--export-sql-chart-csv",
            str(case_path),
            snapshots[1]["id"],
            str(chart_file),
            "--json",
        )
        if (
            not exported_chart.get("ok")
            or exported_chart.get("connections") != 4
            or exported_chart.get("format") != format_name
            or exported_chart.get("snapshot_id") != snapshots[1]["id"]
            or exported_chart.get("entity_property_columns") != 1
            or exported_chart.get("relationship_property_columns") != 1
        ):
            raise RuntimeError(f"SQL chart {format_name} export returned unexpected metadata")
        with chart_file.open(newline="", encoding="utf-8") as source:
            chart_rows = list(csv.DictReader(source, delimiter=delimiter))
        if len(chart_rows) != 4:
            raise RuntimeError(f"SQL chart {format_name} export did not contain four records")
        required_headers = {
            "connection_id",
            "source_external_ids_json",
            "target_external_ids_json",
            "source_attribute:registry",
            "target_attribute:registry",
            "connection_attribute:capture_method",
            "ingest_source",
        }
        if not required_headers.issubset(chart_rows[0]):
            raise RuntimeError(f"SQL chart {format_name} export omitted expected identity/property columns")
        if any(row["record_type"] != "connection" for row in chart_rows):
            raise RuntimeError(f"SQL chart {format_name} export contains unexpected record types")
        if any(
            row["source_attribute:registry"] != "SYNTHETIC REGISTRY"
            or row["target_attribute:registry"] != "SYNTHETIC REGISTRY"
            or row["connection_attribute:capture_method"] != "SQL ACCEPTANCE FIXTURE"
            for row in chart_rows
        ):
            raise RuntimeError(f"SQL chart {format_name} export changed SQL properties")
        canonical_roundtrip_rows[format_name] = sorted(
            tuple(row[key] for key in sorted(chart_rows[0])) for row in chart_rows
        )

        design_name = f"SQL snapshot {format_name} handoff"
        design_path = work / f"chart-import.{extension}.design.json"
        design_result = run(
            f"chart_{extension}_design_create",
            "--create-import-design",
            str(chart_file),
            str(design_path),
            "--name",
            design_name,
            "--json",
        )
        if not design_result.get("ok"):
            raise RuntimeError(f"could not create the {format_name} import design: {design_result}")

        roundtrip_case_path = work / f"chart-roundtrip.{extension}.case.json"
        roundtrip_case = json.loads(case_path.read_text(encoding="utf-8"))
        for collection in (
            "entities",
            "links",
            "snapshots",
            "automation_jobs",
            "automation_runs",
            "automation_batches",
        ):
            roundtrip_case[collection] = []
        roundtrip_case_path.write_text(
            json.dumps(roundtrip_case, indent=2) + "\n", encoding="utf-8"
        )
        design_import = run(
            f"chart_{extension}_design_import",
            "--import-import-design",
            str(roundtrip_case_path),
            str(design_path),
            "--name",
            design_name,
            "--json",
        )
        if not design_import.get("ok") or design_import.get("saved") is not True:
            raise RuntimeError(f"could not save the {format_name} import design")
        chart_import = run(
            f"chart_{extension}_data_import",
            "--import-delimited",
            str(roundtrip_case_path),
            str(chart_file),
            "--design",
            design_name,
            "--json",
        )
        if (
            not chart_import.get("ok")
            or chart_import.get("created") != 4
            or chart_import.get("updated") != 0
            or chart_import.get("skipped") != 0
        ):
            raise RuntimeError(f"SQL chart {format_name} import returned unexpected counts")

        imported_case = json.loads(roundtrip_case_path.read_text(encoding="utf-8"))
        imported_entities = {entity["id"]: entity for entity in imported_case["entities"]}
        imported_links = {
            link["connection_id"]: link for link in imported_case["links"]
        }
        if len(imported_case["entities"]) != 5 or len(imported_links) != 4:
            raise RuntimeError(f"SQL chart {format_name} import lost or duplicated graph records")
        for row in chart_rows:
            link = imported_links.get(row["connection_id"])
            if link is None:
                raise RuntimeError(f"SQL chart {format_name} import lost {row['connection_id']}")
            source_entity = imported_entities[link["from"]]
            target_entity = imported_entities[link["to"]]
            if (
                source_entity["name"] != row["source"]
                or source_entity["kind"] != row["source_type"]
                or target_entity["name"] != row["target"]
                or target_entity["kind"] != row["target_type"]
                or link["label"] != row["relationship"]
                or link["date"] != row["date"]
                or link["confidence"] != int(row["confidence"])
                or link["directed"] != (row["directed"].strip().lower() == "true")
                or link["source_notes"] != row["source_notes"]
                or link["source_reference"] != row["source_reference"]
                or link.get("ingest_source") != row["ingest_source"]
            ):
                raise RuntimeError(f"SQL chart {format_name} import changed {row['connection_id']}")
            source_ids = {tuple(pair) for pair in json.loads(row["source_external_ids_json"])}
            target_ids = {tuple(pair) for pair in json.loads(row["target_external_ids_json"])}
            if not source_ids or not target_ids:
                raise RuntimeError(f"SQL chart {format_name} export omitted stable endpoint identity")
            if not source_ids.issubset({tuple(pair) for pair in source_entity["external_ids"]}):
                raise RuntimeError(f"SQL chart {format_name} import changed source identity")
            if not target_ids.issubset({tuple(pair) for pair in target_entity["external_ids"]}):
                raise RuntimeError(f"SQL chart {format_name} import changed target identity")
            if dict(source_entity["attributes"]).get("registry") != row["source_attribute:registry"]:
                raise RuntimeError(f"SQL chart {format_name} import changed source properties")
            if dict(target_entity["attributes"]).get("registry") != row["target_attribute:registry"]:
                raise RuntimeError(f"SQL chart {format_name} import changed target properties")
            if dict(link["attributes"]).get("capture_method") != row["connection_attribute:capture_method"]:
                raise RuntimeError(f"SQL chart {format_name} import changed relationship properties")
        chart_roundtrips.append(
            {
                "format": format_name,
                "exported_connections": exported_chart["connections"],
                "created_connections": chart_import["created"],
                "entities": len(imported_entities),
                "relationships": len(imported_links),
                "skipped": chart_import["skipped"],
                "export_sha256": sha256(chart_file),
                "design_sha256": sha256(design_path),
                "case_sha256": sha256(roundtrip_case_path),
            }
        )
    if canonical_roundtrip_rows["CSV"] != canonical_roundtrip_rows["TSV"]:
        raise RuntimeError("SQL chart CSV and TSV exports contain different record values")
    source_hash_after_chart_exchange = sha256(database)
    if source_hash_before_chart_exchange != source_hash_after_chart_exchange:
        raise RuntimeError("chart CSV/TSV exchange changed the SQLite source")
    if sha256(case_path) != source_case_hash_before_chart_export:
        raise RuntimeError("chart CSV/TSV export changed the source case")

    # Exercise schema adaptation end to end against the disposable source. The
    # application must diagnose the stale query and preserve the failed run;
    # query repair remains an explicit analyst action.
    schema_editor = sqlite3.connect(database)
    try:
        schema_editor.execute(
            "ALTER TABLE connection_observations RENAME COLUMN event_date TO observed_date"
        )
        schema_editor.commit()
    finally:
        schema_editor.close()
    adaptation_case_path = work / "schema-adaptation.case.json"
    adaptation_case_path.write_bytes(failure_case_path.read_bytes())
    adaptation_database_hash = sha256(database)
    inspected_schema = run(
        "adapted_schema_inspection", "--inspect-sqlite", str(database), "--json"
    )
    inspected_columns = {
        column["name"]
        for item in inspected_schema.get("objects", [])
        if item.get("name") == "connection_observations"
        for column in item.get("columns", [])
    }
    if "observed_date" not in inspected_columns or "event_date" in inspected_columns:
        raise RuntimeError(f"SQLite inspection did not expose the changed column: {inspected_schema}")

    case_before_adaptation_failure = sha256(adaptation_case_path)
    adaptation_failure = run(
        "schema_adaptation_failure",
        "--run-job",
        str(adaptation_case_path),
        "Synthetic connection capture",
        "--json",
        expected_exit=1,
    )
    if sha256(database) != adaptation_database_hash:
        raise RuntimeError("schema-adaptation failure changed the read-only SQLite source")
    adaptation_outcomes = adaptation_failure.get("outcomes", [])
    if (
        adaptation_failure.get("ok")
        or adaptation_failure.get("saved") is not True
        or len(adaptation_outcomes) != 1
    ):
        raise RuntimeError(f"schema-adaptation failure was not retained: {adaptation_failure}")
    adaptation_outcome = adaptation_outcomes[0]
    if adaptation_outcome.get("status") != "FAIL":
        raise RuntimeError(f"schema-adaptation outcome did not report FAIL: {adaptation_outcome}")
    adaptation = adaptation_outcome.get("schema_adaptation") or {}
    if adaptation.get("kind") != "missing_column" or adaptation.get("reference") != "event_date":
        raise RuntimeError(f"schema-adaptation diagnosis was not precise: {adaptation}")
    diagnosis = adaptation_outcome.get("schema_diagnosis") or {}
    if any(
        diagnosis.get(key) != expected
        for key, expected in {
            "baseline_available": True,
            "schema_changed": True,
            "query_changed": False,
            "parameters_changed": False,
        }.items()
    ):
        raise RuntimeError(f"schema-adaptation comparison was unexpected: {diagnosis}")
    adapted_failed_case = json.loads(adaptation_case_path.read_text(encoding="utf-8"))
    adapted_failed_runs = adapted_failed_case.get("automation_runs", [])
    if (
        len(adapted_failed_runs) != 4
        or adapted_failed_runs[-1].get("success") is not False
        or "no such column: event_date" not in adapted_failed_runs[-1].get("message", "")
        or adapted_failed_runs[-1].get("schema_diagnosis", {}).get("schema_changed") is not True
    ):
        raise RuntimeError("schema-adaptation failure record was not saved to the case")
    case_after_adaptation_failure = sha256(adaptation_case_path)
    if case_after_adaptation_failure == case_before_adaptation_failure:
        raise RuntimeError("schema-adaptation failure did not update the case history")
    adaptation_history = run(
        "schema_adaptation_history",
        "--list-runs",
        str(adaptation_case_path),
        "--schema-adaptation-needed",
        "--json",
    )
    if adaptation_history.get("total") != 1 or len(adaptation_history.get("runs", [])) != 1:
        raise RuntimeError(f"schema-adaptation failure was not filterable in history: {adaptation_history}")

    repaired_case = json.loads(adaptation_case_path.read_text(encoding="utf-8"))
    repaired_job = repaired_case["automation_jobs"][0]
    stale_expression = "event_date AS date"
    if repaired_job["query"].count(stale_expression) != 1:
        raise RuntimeError("expected one stale event_date projection for reviewed repair")
    repaired_job["query"] = repaired_job["query"].replace(
        stale_expression, "observed_date AS date"
    )
    adaptation_case_path.write_text(
        json.dumps(repaired_case, indent=2) + "\n", encoding="utf-8"
    )
    case_before_query_review = sha256(adaptation_case_path)
    query_review = run(
        "analyst_reviewed_query_validation",
        "--validate-job",
        str(adaptation_case_path),
        "Synthetic connection capture",
        "--query-only",
        "--json",
    )
    if not query_review.get("ok") or query_review.get("saved") is not False:
        raise RuntimeError(f"reviewed query did not validate without saving: {query_review}")
    if sha256(adaptation_case_path) != case_before_query_review:
        raise RuntimeError("read-only reviewed-query validation changed the case file")
    reviewed_capture = run(
        "analyst_reviewed_query_capture",
        "--run-job",
        str(adaptation_case_path),
        "Synthetic connection capture",
        "--json",
    )
    if not reviewed_capture.get("ok") or reviewed_capture.get("saved") is not True:
        raise RuntimeError(f"reviewed query capture did not pass: {reviewed_capture}")
    if sha256(database) != adaptation_database_hash:
        raise RuntimeError("reviewed query capture changed the read-only SQLite source")
    repaired_saved_case = json.loads(adaptation_case_path.read_text(encoding="utf-8"))
    repaired_runs = repaired_saved_case.get("automation_runs", [])
    repaired_snapshots = repaired_saved_case.get("snapshots", [])
    if (
        len(repaired_runs) != 5
        or repaired_runs[-1].get("success") is not True
        or len(repaired_snapshots) != 4
        or len(repaired_snapshots[-1].get("connections", [])) != 4
    ):
        raise RuntimeError("reviewed query did not retain its successful four-record capture")
    adaptation_history_after_repair = run(
        "schema_adaptation_history_after_repair",
        "--list-runs",
        str(adaptation_case_path),
        "--schema-adaptation-needed",
        "--json",
    )
    if adaptation_history_after_repair.get("total") != 1:
        raise RuntimeError("successful repair removed the historical schema-adaptation failure")

    report = {
        "schema_version": 1,
        "result": "PASS",
        "fixture": source_sql.name,
        "binary": str(binary),
        "work_dir": str(work),
        "database_sha256_before_capture": source_hash_before,
        "database_sha256_after_captures": source_hash_after,
        "source_unchanged_during_captures": True,
        "database_sha256_before_schema_preflight": schema_hash_before_preflight,
        "database_sha256_after_schema_preflight": schema_hash_after_preflight,
        "source_unchanged_during_schema_preflight": True,
        "case_sha256_before_schema_preflight": case_hash_before_schema_preflight,
        "case_sha256_after_schema_preflight": case_hash_after_schema_preflight,
        "case_unchanged_during_schema_preflight": True,
        "query_preflight_saved_case": False,
        "schema_preflight_saved_case": schema_preflight.get("saved"),
        "schema_preflight_status": schema_outcome.get("status"),
        "schema_preflight_changed": schema_outcome["schema_stability"]["changed"],
        "database_sha256_before_failure_capture": source_hash_before_failure_capture,
        "database_sha256_after_failure_capture": source_hash_after_failure_capture,
        "source_unchanged_during_failure_capture": True,
        "failure_capture_saved_case": failed_capture.get("saved"),
        "failure_capture_status": failed_outcome.get("status"),
        "failure_capture_run_id": failed_runs[-1].get("id"),
        "failure_capture_snapshot_id": failed_outcome.get("snapshot_id"),
        "failure_run_history_retained": True,
        "chart_csv_tsv_roundtrips": chart_roundtrips,
        "schema_adaptation": {
            "inspected_observed_date": "observed_date" in inspected_columns,
            "failed_run_saved": True,
            "diagnosis": diagnosis,
            "reference": adaptation.get("reference"),
            "history_retained_after_repair": True,
            "reviewed_query_validated_without_case_write": True,
            "reviewed_capture_connections": len(repaired_snapshots[-1]["connections"]),
            "database_sha256": adaptation_database_hash,
            "case_sha256_before_failure": case_before_adaptation_failure,
            "case_sha256_after_failure": case_after_adaptation_failure,
            "case_sha256_after_repair": sha256(adaptation_case_path),
        },
        "csv_and_tsv_values_match": True,
        "source_unchanged_during_chart_exchange": True,
        "case_unchanged_during_chart_export": True,
        "snapshot_counts": [len(snapshot["connections"]) for snapshot in snapshots],
        "snapshot_ids": [snapshot["id"] for snapshot in snapshots],
        "run_ids": [run_record["id"] for run_record in runs],
        "batch_ids": [run_record["batch_id"] for run_record in runs],
        "saved_job_parameters_after_one_run_override": jobs[0]["parameters"],
        "expected_delta_kinds": sorted(required_changes),
        "comparison_rows": len(comparison_rows),
        "commands": [
            {
                "label": record["label"],
                "exit_code": record["exit_code"],
                "expected_exit_code": record["expected_exit_code"],
            }
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
