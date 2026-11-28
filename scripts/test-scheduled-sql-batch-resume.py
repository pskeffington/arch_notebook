#!/usr/bin/env python3
"""Exercise scheduled SQL batch recovery after a process is interrupted."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import signal
import sqlite3
import subprocess
import tempfile
import time
import uuid
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--fixture", type=Path, default=Path("fixtures/chart/correctness.case.json"))
    parser.add_argument("--work-dir", type=Path, help="retain artifacts in this new directory")
    args = parser.parse_args()
    binary = args.binary.resolve()
    fixture = args.fixture.resolve()
    if not binary.is_file() or not fixture.is_file():
        parser.error("binary and case fixture must exist")

    temporary = None
    if args.work_dir:
        work = args.work_dir.resolve()
        work.mkdir(parents=True, exist_ok=False)
    else:
        temporary = tempfile.TemporaryDirectory(prefix="arch-notebook-scheduled-resume-")
        work = Path(temporary.name)

    first_database_path = work / "first.sqlite"
    second_database_path = work / "second.sqlite"
    for database_path in (first_database_path, second_database_path):
        connection = sqlite3.connect(database_path)
        connection.execute(
            "CREATE TABLE records (source TEXT, target TEXT, relationship TEXT)"
        )
        connection.execute("INSERT INTO records VALUES ('Alpha', 'Bravo', 'CONTACT')")
        connection.commit()
        connection.close()

    case = json.loads(fixture.read_text(encoding="utf-8"))
    def make_job(name: str, database_path: Path) -> dict[str, object]:
        return {
            "id": str(uuid.uuid4()),
            "chart_id": None,
            "name": name,
            "database_path": str(database_path),
            "series": name,
            "query": "SELECT source, target, relationship FROM records",
            "parameters": [],
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

    case["automation_jobs"] = [
        make_job("First checkpoint", first_database_path),
        make_job("Interrupted capture", second_database_path),
    ]
    case["automation_runs"] = []
    case["automation_batches"] = []
    case_path = work / "scheduled-resume.case.json"
    case_path.write_text(json.dumps(case, indent=2) + "\n", encoding="utf-8")

    lock = sqlite3.connect(second_database_path, timeout=2)
    lock.execute("BEGIN EXCLUSIVE")
    lock.execute("UPDATE records SET relationship = 'LOCKED' WHERE source = 'Alpha'")

    environment = os.environ.copy()
    environment["ARCH_NOTEBOOK_AUTOMATION_TRIGGER"] = "scheduled"
    command = [
        str(binary),
        "--run-enabled-jobs",
        str(case_path),
        "--progress",
        "--resume-incomplete-scheduled-batch",
        "--json",
    ]
    interrupted_log = work / "interrupted.stderr.log"
    process = subprocess.Popen(
        command,
        env=environment,
        stdout=subprocess.DEVNULL,
        stderr=interrupted_log.open("w", encoding="utf-8"),
        text=True,
    )
    batch_id = None
    deadline = time.monotonic() + 30
    try:
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(
                    f"first scheduled process exited before interruption: {process.returncode}; "
                    f"see {interrupted_log}"
                )
            try:
                saved_case = json.loads(case_path.read_text(encoding="utf-8"))
                batches = saved_case.get("automation_batches", [])
                if batches and batches[-1].get("completed_jobs"):
                    batch_id = batches[-1]["batch_id"]
                    log = interrupted_log.read_text(encoding="utf-8")
                    if "Running job 2 of 2" in log:
                        break
            except (OSError, json.JSONDecodeError):
                pass
            time.sleep(0.025)
        else:
            raise RuntimeError("timed out waiting for job 1 checkpoint and job 2 start")
        os.kill(process.pid, signal.SIGKILL)
        process.wait(timeout=10)
    finally:
        lock.rollback()
        lock.close()
        if process.poll() is None:
            process.kill()
            process.wait(timeout=10)

    interrupted_case = json.loads(case_path.read_text(encoding="utf-8"))
    if len(interrupted_case["automation_runs"]) != 1:
        raise RuntimeError("interruption did not leave exactly the first completed run")
    if interrupted_case["automation_batches"][-1]["batch_id"] != batch_id:
        raise RuntimeError("interrupted batch ID changed before recovery")
    if len(interrupted_case["automation_batches"][-1]["completed_jobs"]) != 1:
        raise RuntimeError("interruption did not preserve exactly one completed checkpoint")

    resumed = subprocess.run(
        command,
        env=environment,
        text=True,
        capture_output=True,
        check=False,
        timeout=60,
    )
    if resumed.returncode != 0:
        raise RuntimeError(
            f"scheduled recovery exited {resumed.returncode}:\n"
            f"stdout:\n{resumed.stdout}\nstderr:\n{resumed.stderr}"
        )
    report = json.loads(resumed.stdout)
    completed_case = json.loads(case_path.read_text(encoding="utf-8"))
    batch = completed_case["automation_batches"][-1]
    if report.get("batch_id") != batch_id or report.get("auto_resumed") is not True:
        raise RuntimeError(f"runner did not report recovery of batch {batch_id}: {report}")
    if f"AUTOMATION BATCH AUTO-RESUME · {batch_id}" not in resumed.stderr:
        raise RuntimeError("runner did not log the automatic resume decision")
    if len(completed_case["automation_runs"]) != 2:
        raise RuntimeError("recovery duplicated or omitted a scheduled SQL run")
    if len(batch["completed_jobs"]) != 2:
        raise RuntimeError("recovery did not complete both checkpoint entries")
    if len(report.get("skipped_jobs", [])) != 1:
        raise RuntimeError("recovery did not skip the already checkpointed job")
    if any(run.get("trigger") != "scheduled" for run in completed_case["automation_runs"]):
        raise RuntimeError("recovered SQL runs lost their scheduled trigger")

    case_digest = hashlib.sha256(case_path.read_bytes()).hexdigest()
    manual_environment = environment.copy()
    manual_environment.pop("ARCH_NOTEBOOK_AUTOMATION_TRIGGER", None)
    manual_attempt = subprocess.run(
        command,
        env=manual_environment,
        text=True,
        capture_output=True,
        check=False,
        timeout=10,
    )
    if manual_attempt.returncode == 0 or "restricted to an unfiltered scheduled runner" not in manual_attempt.stderr:
        raise RuntimeError("automatic resume flag was not rejected outside a scheduled runner")
    if hashlib.sha256(case_path.read_bytes()).hexdigest() != case_digest:
        raise RuntimeError("rejected manual resume attempt changed the case")

    print(
        json.dumps(
            {
                "schema_version": 1,
                "result": "PASS",
                "batch_id": batch_id,
                "process_interruption": "SIGKILL after first job checkpoint",
                "recovered_same_batch": report["batch_id"] == batch_id,
                "auto_resumed": report["auto_resumed"],
                "completed_jobs": len(batch["completed_jobs"]),
                "already_completed_job_skipped": len(report["skipped_jobs"]) == 1,
                "saved_runs": len(completed_case["automation_runs"]),
                "manual_resume_flag_rejected_without_case_change": True,
                "work_dir": str(work),
            },
            indent=2,
        )
    )
    if temporary:
        temporary.cleanup()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
