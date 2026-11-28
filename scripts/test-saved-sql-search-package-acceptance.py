#!/usr/bin/env python3
"""Exercise definition-only saved SQL search package CLI interchange."""

from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "fixtures/chart/correctness.case.json"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(binary: Path, *args: str, success: bool = True) -> tuple[subprocess.CompletedProcess[str], dict]:
    result = subprocess.run(
        [str(binary), *args, "--json"], text=True, capture_output=True, check=False
    )
    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(
            f"CLI did not return JSON (exit {result.returncode}): {result.stderr.strip()}"
        ) from error
    if success and result.returncode != 0:
        raise RuntimeError(f"CLI failed (exit {result.returncode}): {payload}")
    if not success and result.returncode == 0:
        raise RuntimeError(f"CLI unexpectedly succeeded: {payload}")
    return result, payload


def write_case(path: Path, searches: list[dict] | None = None) -> None:
    case = json.loads(FIXTURE.read_text(encoding="utf-8"))
    case["saved_sql_searches"] = searches or []
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(case), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    if not binary.is_file() or not binary.stat().st_mode & 0o111:
        raise SystemExit(f"not an executable binary: {binary}")

    with tempfile.TemporaryDirectory(prefix="arch-notebook-sql-search-package-") as temp:
        root = Path(temp)
        source_dir = root / "source"
        destination_dir = root / "destination"
        source_case = source_dir / "investigation.case.json"
        destination_case = destination_dir / "investigation.case.json"
        source_db = source_dir / "data/source.sqlite"
        source_db.parent.mkdir(parents=True)
        with sqlite3.connect(source_db) as db:
            db.execute("CREATE TABLE contacts (source TEXT, target TEXT, owner_id TEXT, state TEXT)")
        search = {
            "id": "00000000-0000-0000-0000-000000008101",
            "name": "Seeded contacts",
            "database_path": "data/source.sqlite",
            "series": "contact review",
            "query": "SELECT source, target FROM contacts WHERE owner_id = :entity_id AND state = :state",
            "parameters": [["state", "text:active"]],
            "seed_parameter_name": "entity_id",
        }
        write_case(source_case, [search])
        write_case(destination_case)
        package_path = root / "packages/seeded-contacts.json"
        package_path.parent.mkdir(parents=True)

        _result, exported = run(
            binary, "--export-sql-search", str(source_case), search["name"], str(package_path)
        )
        package = json.loads(package_path.read_text(encoding="utf-8"))
        assert exported["ok"] and exported["search"] == search["name"]
        assert package["format"] == "arch-notebook-sql-search"
        assert package["schema_version"] == 1
        assert package["search"]["seed_parameter_name"] == "entity_id"
        assert package["search"]["parameters"] == [["state", "text:active"]]
        assert "chart-specific-old-id" not in package_path.read_text(encoding="utf-8")

        destination_db = destination_dir / "data/destination.sqlite"
        destination_db.parent.mkdir(parents=True)
        with sqlite3.connect(destination_db) as db:
            db.execute("CREATE TABLE contacts (source TEXT, target TEXT, owner_id TEXT, state TEXT)")
        _result, imported = run(
            binary,
            "--import-sql-search",
            str(destination_case),
            str(package_path),
            "--name",
            "Imported contacts",
            "--database",
            str(destination_db),
            "--series",
            "destination review",
        )
        imported_case = json.loads(destination_case.read_text(encoding="utf-8"))
        imported_search = imported_case["saved_sql_searches"][-1]
        assert imported["saved"] and imported["executed"] is False
        assert imported_search["name"] == "Imported contacts"
        assert imported_search["database_path"] == str(destination_db)
        assert imported_search["series"] == "destination review"
        assert imported_search["parameters"] == [["state", "text:active"]]
        assert imported_search["seed_parameter_name"] == "entity_id"
        assert imported_search["id"] != search["id"]
        assert len(imported_case["snapshots"]) == len(json.loads(FIXTURE.read_text())["snapshots"])

        destination_hash = sha256(destination_case)
        _result, duplicate = run(
            binary,
            "--import-sql-search",
            str(destination_case),
            str(package_path),
            "--name",
            "imported CONTACTS",
            success=False,
        )
        assert duplicate["ok"] is False
        assert sha256(destination_case) == destination_hash

        source_hash = sha256(source_db)
        _result, protected = run(
            binary,
            "--export-sql-search",
            str(source_case),
            search["name"],
            str(source_db),
            success=False,
        )
        assert protected["ok"] is False
        assert sha256(source_db) == source_hash

        malicious = json.loads(package_path.read_text(encoding="utf-8"))
        malicious["search"]["parameters"].append(["entity_id", "text:chart-specific-old-id"])
        malicious_path = root / "packages/bound-seed.json"
        malicious_path.write_text(json.dumps(malicious), encoding="utf-8")
        malicious_destination = root / "malicious.case.json"
        write_case(malicious_destination)
        malicious_hash = sha256(malicious_destination)
        _result, rejected = run(
            binary,
            "--import-sql-search",
            str(malicious_destination),
            str(malicious_path),
            success=False,
        )
        assert rejected["ok"] is False
        assert sha256(malicious_destination) == malicious_hash

        report = {
            "suite": "arch_notebook_saved_sql_search_package_acceptance",
            "result": "PASS",
            "binary_sha256": sha256(binary),
            "round_trip": "PASS",
            "seed_default_values_exported": False,
            "chart_seed_rebound_by_consumer": True,
            "duplicate_import_case_unchanged": True,
            "malicious_bound_seed_rejected": True,
            "sqlite_source_overwrite_rejected_and_source_unchanged": True,
            "query_executed_during_import": False,
            "snapshot_count_unchanged_during_import": True,
        }
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
