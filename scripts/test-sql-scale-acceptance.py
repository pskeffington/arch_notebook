#!/usr/bin/env python3
"""Exercise SQL automation on a fictional joined email-header workload."""

from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
import subprocess
import tempfile
from pathlib import Path


ROW_COUNT = 7_121
CONTACT_COUNT = 1_500


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--work-dir", type=Path, help="retain artifacts in a new directory")
    args = parser.parse_args()
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error(f"Arch Notebook binary does not exist: {binary}")

    temporary = None
    if args.work_dir:
        work = args.work_dir.resolve()
        work.mkdir(parents=True, exist_ok=False)
    else:
        temporary = tempfile.TemporaryDirectory(prefix="arch-notebook-sql-scale-")
        work = Path(temporary.name)

    database = work / "fictional-email-headers.sqlite"
    connection = sqlite3.connect(database)
    try:
        connection.executescript(
            """
            PRAGMA journal_mode = DELETE;
            CREATE TABLE contacts (
                tenant_id TEXT NOT NULL,
                person_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                department TEXT,
                PRIMARY KEY (tenant_id, person_id)
            );
            CREATE TABLE messages (
                message_id INTEGER PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                sent_at TEXT NOT NULL,
                thread_token TEXT,
                confidence INTEGER NOT NULL DEFAULT 2,
                FOREIGN KEY (tenant_id, sender_id)
                    REFERENCES contacts (tenant_id, person_id)
            );
            CREATE TABLE recipients (
                message_id INTEGER NOT NULL,
                recipient_ordinal INTEGER NOT NULL,
                tenant_id TEXT NOT NULL,
                person_id TEXT NOT NULL,
                PRIMARY KEY (message_id, recipient_ordinal),
                FOREIGN KEY (message_id) REFERENCES messages (message_id),
                FOREIGN KEY (tenant_id, person_id)
                    REFERENCES contacts (tenant_id, person_id)
            );
            CREATE INDEX recipients_person_idx ON recipients (tenant_id, person_id);
            """
        )
        connection.executemany(
            "INSERT INTO contacts VALUES (?, ?, ?, ?)",
            (
                (
                    f"TENANT-{index % 3:02d}",
                    f"PERSON-{index:05d}",
                    f"Fictional Contact {index:05d}",
                    None if index % 5 == 0 else f"UNIT-{index % 11:02d}",
                )
                for index in range(CONTACT_COUNT)
            ),
        )
        message_count = ROW_COUNT // 2 + 1
        connection.executemany(
                "INSERT INTO messages VALUES (?, ?, ?, ?, ?, ?)",
            (
                (
                    message_id,
                    f"TENANT-{(message_id * 17) % 3:02d}",
                    f"PERSON-{(message_id * 17) % CONTACT_COUNT:05d}",
                    f"2025-01-{(message_id % 28) + 1:02d}T12:00:00Z",
                    None if message_id % 7 == 0 else f"THREAD-{message_id % 83:03d}",
                    2,
                )
                for message_id in range(1, message_count + 1)
            ),
        )
        recipients = []
        for message_id in range(1, message_count + 1):
            recipient_count = 1 if message_id == 1 else 2
            sender_index = (message_id * 17) % CONTACT_COUNT
            sender_tenant = f"TENANT-{(message_id * 17) % 3:02d}"
            for ordinal in range(recipient_count):
                person_index = (sender_index + ordinal + 1) % CONTACT_COUNT
                recipients.append(
                    (
                        message_id,
                        ordinal,
                        f"TENANT-{person_index % 3:02d}",
                        f"PERSON-{person_index:05d}",
                    )
                )
        connection.executemany("INSERT INTO recipients VALUES (?, ?, ?, ?)", recipients)
        connection.commit()
        actual_rows = connection.execute("SELECT count(*) FROM recipients").fetchone()[0]
        if actual_rows != ROW_COUNT:
            raise RuntimeError(f"fixture has {actual_rows} rows; expected {ROW_COUNT}")
    finally:
        connection.close()

    query = """SELECT
        printf('MSG-%06d:RCPT-%02d', m.message_id, r.recipient_ordinal) AS connection_id,
        sender.display_name AS source,
        'EMAIL ACCOUNT' AS source_type,
        recipient.display_name AS target,
        'EMAIL ACCOUNT' AS target_type,
        'SENT TO' AS relationship,
        m.sent_at AS date,
        m.confidence AS confidence,
        1 AS directed,
        CASE WHEN m.message_id % 13 = 0 THEN NULL
             ELSE printf('HEADER-REF-%06d', m.message_id) END AS source_reference,
        '' AS notes,
        sender.tenant_id || ':' || sender.person_id AS source_entity_id,
        recipient.tenant_id || ':' || recipient.person_id AS target_entity_id,
        sender.department AS "source_attribute:department",
        recipient.department AS "target_attribute:department",
        m.thread_token AS "connection_attribute:thread_token"
    FROM messages AS m
    JOIN contacts AS sender
      ON sender.tenant_id = m.tenant_id AND sender.person_id = m.sender_id
    JOIN recipients AS r ON r.message_id = m.message_id
    JOIN contacts AS recipient
      ON recipient.tenant_id = r.tenant_id AND recipient.person_id = r.person_id
    ORDER BY m.message_id, r.recipient_ordinal"""

    case_path = work / "scale.case.json"
    case_path.write_text(
        json.dumps(
            {
                "format_version": 19,
                "title": "FICTIONAL EMAIL HEADER SCALE ACCEPTANCE",
                "description": "Synthetic header-only relational fixture; no message bodies or real people.",
                "entities": [],
                "links": [],
                "automation_jobs": [
                    {
                        "id": "5b2e6f51-3c22-4a52-9a76-21a9f98a7261",
                        "name": "Synthetic joined header capture",
                        "database_path": str(database),
                        "series": "fictional-email-header-review",
                        "query": query,
                        "parameters": [],
                        "query_timeout_minutes": 15,
                        "enabled": True,
                        "minimum_connections": ROW_COUNT,
                        "minimum_confidence": None,
                        "maximum_connections": ROW_COUNT,
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

    source_hash_before = sha256(database)
    case_before_preflight = case_path.read_bytes()

    def run(label: str, *arguments: str) -> dict:
        process = subprocess.run(
            [str(binary), *arguments], text=True, capture_output=True, check=False
        )
        if process.returncode != 0:
            raise RuntimeError(
                f"{label} exited {process.returncode}: {process.stderr or process.stdout}"
            )
        try:
            return json.loads(process.stdout)
        except json.JSONDecodeError as error:
            raise RuntimeError(f"{label} did not return JSON: {error}") from error

    preflight = run("query preflight", "--validate-enabled-jobs", str(case_path), "--json")
    if not preflight.get("ok") or preflight.get("failed") != 0:
        raise RuntimeError(f"query preflight failed: {preflight}")
    if case_path.read_bytes() != case_before_preflight:
        raise RuntimeError("read-only query preflight changed the case file")
    if sha256(database) != source_hash_before:
        raise RuntimeError("query preflight changed the SQLite source")

    baseline_capture = run(
        "baseline automation capture", "--run-enabled-jobs", str(case_path), "--json"
    )
    source_hash_after_baseline = sha256(database)
    if not baseline_capture.get("ok") or baseline_capture.get("passed") != 1:
        raise RuntimeError(f"large joined baseline capture did not pass: {baseline_capture}")
    if source_hash_after_baseline != source_hash_before:
        raise RuntimeError("baseline SQL capture changed the read-only source")

    # The harness deliberately changes one disposable source row between snapshots.
    connection = sqlite3.connect(database)
    try:
        connection.execute("UPDATE messages SET confidence = 1 WHERE message_id = 101")
        connection.commit()
    finally:
        connection.close()
    source_hash_before_current = sha256(database)
    current_capture = run(
        "current automation capture", "--run-enabled-jobs", str(case_path), "--json"
    )
    source_hash_after_current = sha256(database)
    if not current_capture.get("ok") or current_capture.get("passed") != 1:
        raise RuntimeError(f"large joined current capture did not pass: {current_capture}")
    if source_hash_after_current != source_hash_before_current:
        raise RuntimeError("current SQL capture changed the read-only source")

    case = json.loads(case_path.read_text(encoding="utf-8"))
    if len(case.get("snapshots", [])) != 2 or len(case.get("automation_runs", [])) != 2:
        raise RuntimeError("two captures did not atomically persist both snapshots and runs")
    snapshots = case["snapshots"]
    if any(len(snapshot.get("connections", [])) != ROW_COUNT for snapshot in snapshots):
        raise RuntimeError("a large SQL snapshot lost relationship records")
    history = run("run history", "--list-runs", str(case_path), "--json")
    history_runs = history.get("runs", [])
    if len(history_runs) != 2:
        raise RuntimeError("run history did not retain both large-capture runs")
    latest_run = history_runs[0]
    change_kinds = {
        kind
        for change in latest_run.get("delta", {}).get("changes", [])
        for kind in change.get("kinds", [])
    }
    if "confidence_declined" not in change_kinds:
        raise RuntimeError("large snapshot comparison missed the deliberate confidence decline")
    snapshot = snapshots[-1]
    links = case.get("links", [])
    if len(snapshot.get("connections", [])) != ROW_COUNT or len(links) != ROW_COUNT:
        raise RuntimeError("large SQL capture lost or collapsed relationship records")
    connection_ids = [link.get("connection_id") for link in links]
    if len(set(connection_ids)) != ROW_COUNT:
        raise RuntimeError("composite message/recipient identities were not retained uniquely")
    if not any(identifier.endswith(":RCPT-01") for identifier in connection_ids):
        raise RuntimeError("composite recipient identity is missing from the captured links")
    endpoint_pairs = [(link["from"], link["to"]) for link in links]
    if len(set(endpoint_pairs)) >= ROW_COUNT:
        raise RuntimeError("fixture did not exercise repeated endpoint pairs")
    if not 1_000 <= len(case["entities"]) <= CONTACT_COUNT:
        raise RuntimeError(f"unexpected endpoint identity count: {len(case['entities'])}")
    if snapshot.get("skipped_rows") != 0:
        raise RuntimeError(f"large joined capture skipped rows: {snapshot.get('skipped_rows')}")
    if source_hash_after_baseline != source_hash_before or source_hash_after_current != source_hash_before_current:
        raise RuntimeError("an application capture changed the SQLite source")
    null_spellings = {"null", "none"}
    entity_sql_values = [
        (name, value)
        for entity in case["entities"]
        for _, name, value in entity.get("sql_attribute_values", [])
    ]
    link_sql_values = [
        (name, value)
        for link in links
        for name, value in link.get("attributes", [])
    ]
    if any(value.strip().lower() in null_spellings for _, value in entity_sql_values + link_sql_values):
        raise RuntimeError("SQL NULL attribute was serialized as a string marker")
    department_values = sum(name == "department" for name, _ in entity_sql_values)
    thread_values = sum(name == "thread_token" for name, _ in link_sql_values)
    if department_values >= len(case["entities"]):
        raise RuntimeError("NULL contact departments were not omitted from SQL entity properties")
    if thread_values >= ROW_COUNT:
        raise RuntimeError("NULL thread aliases were not omitted from SQL connection properties")

    report = {
        "schema_version": 1,
        "result": "PASS",
        "workload": "fictional joined email headers; no bodies",
        "source_rows": ROW_COUNT,
        "captured_relationship_records": len(links),
        "snapshots": len(snapshots),
        "distinct_composite_record_ids": len(set(connection_ids)),
        "endpoint_entities": len(case["entities"]),
        "repeated_endpoint_pairs": ROW_COUNT - len(set(endpoint_pairs)),
        "skipped_rows": snapshot.get("skipped_rows"),
        "compared_snapshot_confidence_decline": True,
        "sqlite_source_unchanged_during_each_capture": True,
        "query_only_preflight": "PASS",
        "case_persisted_atomically": True,
        "nullable_attributes_handled": True,
        "entities_with_non_null_department": department_values,
        "connections_with_non_null_thread": thread_values,
        "source_sha256_before_harness_mutation": source_hash_before,
        "source_sha256_after_harness_mutation": source_hash_before_current,
        "baseline_capture_duration_ms": baseline_capture["outcomes"][0]["duration_ms"],
        "current_capture_duration_ms": current_capture["outcomes"][0]["duration_ms"],
        "snapshot_ids": [snapshot["id"] for snapshot in snapshots],
        "work_dir": str(work),
    }
    print(json.dumps(report, indent=2, sort_keys=True))
    if temporary is not None:
        temporary.cleanup()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
