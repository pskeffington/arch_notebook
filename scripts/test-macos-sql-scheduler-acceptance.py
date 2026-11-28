#!/usr/bin/env python3
"""Exercise macOS SQL LaunchAgent generation with isolated host-command stubs."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import os
import plistlib
import sqlite3
import subprocess
import tempfile
from pathlib import Path


def run(
    command: list[str], *, env: dict[str, str], expected: int = 0
) -> subprocess.CompletedProcess[str]:
    process = subprocess.run(command, env=env, text=True, capture_output=True, check=False)
    if process.returncode != expected:
        raise RuntimeError(
            f"command expected exit {expected}, got {process.returncode}: "
            f"{command!r}\nstdout:\n{process.stdout}\nstderr:\n{process.stderr}"
        )
    return process


def plist_paths(agent_dir: Path) -> dict[str, bytes]:
    return {path.name: path.read_bytes() for path in agent_dir.glob("*.plist")}


def canonical_schedule_id(case_path: Path) -> str:
    return hashlib.sha256(str(case_path.resolve()).encode()).hexdigest()[:12]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--installer", required=True, type=Path)
    parser.add_argument("--runner", required=True, type=Path)
    parser.add_argument("--source-sql", required=True, type=Path)
    parser.add_argument("--sql-acceptance", required=True, type=Path)
    parser.add_argument("--work-dir", type=Path, help="retain artifacts in this new directory")
    args = parser.parse_args()
    binary, installer, runner, source_sql, sql_acceptance = (
        path.resolve()
        for path in (args.binary, args.installer, args.runner, args.source_sql, args.sql_acceptance)
    )
    for label, path in (
        ("binary", binary),
        ("installer", installer),
        ("runner", runner),
        ("SQL fixture", source_sql),
        ("SQL acceptance script", sql_acceptance),
    ):
        if not path.is_file():
            parser.error(f"{label} does not exist: {path}")

    temporary = None
    if args.work_dir:
        work = args.work_dir.resolve()
        work.mkdir(parents=True, exist_ok=False)
    else:
        temporary = tempfile.TemporaryDirectory(prefix="arch-notebook-macos-scheduler-")
        work = Path(temporary.name)

    fixture_work = work / "sql-fixture"
    fixture = run(
        [
            "python3",
            str(sql_acceptance),
            "--binary",
            str(binary),
            "--source-sql",
            str(source_sql),
            "--work-dir",
            str(fixture_work),
        ],
        env=os.environ.copy(),
    )
    if json.loads(fixture.stdout).get("result") != "PASS":
        raise RuntimeError("synthetic SQL fixture acceptance did not pass")
    case_template = fixture_work / "automation.case.json"
    case_payload = json.loads(case_template.read_text(encoding="utf-8"))
    source_database = work / "scheduled-window.sqlite"
    database = sqlite3.connect(source_database)
    try:
        database.executescript(source_sql.read_text(encoding="utf-8"))
        window_start, window_end, before_window = database.execute(
            "SELECT date('now', 'start of month', '-1 month'), "
            "date('now', 'start of month'), "
            "date('now', 'start of month', '-1 month', '-1 day')"
        ).fetchone()
        scheduled_rows = [
            (
                "rolling-window",
                connection_id,
                source,
                "PERSON",
                target,
                "LOCATION",
                "OBSERVED",
                event_date,
                2,
                1,
                "LAUNCHAGENT-ACCEPTANCE",
                "synthetic calendar boundary",
            )
            for connection_id, source, target, event_date in (
                ("BEFORE-WINDOW", "Prior Boundary", "Prior Place", before_window),
                ("WINDOW-START", "Window Start", "Included Place", window_start),
                ("WINDOW-END", "Window End", "Excluded Place", window_end),
            )
        ]
        database.executemany(
            "INSERT INTO connection_observations "
            "(snapshot_id, connection_id, source, source_type, target, target_type, "
            "relationship, event_date, confidence, directed, source_reference, notes) "
            "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            scheduled_rows,
        )
        database.commit()
    finally:
        database.close()
    case_payload["automation_jobs"][0]["database_path"] = str(source_database)
    case_payload["automation_jobs"][0]["series"] = "rolling-utc-month"
    case_payload["automation_jobs"][0]["parameters"] = []
    case_payload["automation_jobs"][0]["query"] = """SELECT connection_id, source, source_type, target,
        target_type, relationship, source AS source_entity_id,
        target AS target_entity_id, event_date AS date, confidence, directed,
        source_reference, notes
        FROM connection_observations
        WHERE event_date >= date('now', 'start of month', '-1 month')
          AND event_date < date('now', 'start of month')
        ORDER BY connection_id"""

    home = work / "home"
    fake_bin = work / "host-bin"
    home.mkdir()
    fake_bin.mkdir()
    agent_dir = home / "Library" / "LaunchAgents"
    launch_log = work / "launchctl.jsonl"
    launch_state = work / "loaded-labels.json"
    app_log = work / "app-commands.jsonl"
    launch_state.write_text("[]\n", encoding="utf-8")

    (fake_bin / "uname").write_text(
        "#!/bin/sh\nprintf 'Darwin\\n'\n", encoding="utf-8"
    )
    (fake_bin / "uname").chmod(0o755)
    (fake_bin / "plutil").write_text(
        "#!/usr/bin/env python3\n"
        "import plistlib, sys\n"
        "if len(sys.argv) != 3 or sys.argv[1] != '-lint': raise SystemExit(2)\n"
        "with open(sys.argv[2], 'rb') as source: plistlib.load(source)\n"
        "print(f'{sys.argv[2]}: OK')\n",
        encoding="utf-8",
    )
    (fake_bin / "plutil").chmod(0o755)
    (fake_bin / "launchctl").write_text(
        "#!/usr/bin/env python3\n"
        "import json, os, pathlib, plistlib, sys\n"
        "args = sys.argv[1:]\n"
        "with open(os.environ['LAUNCHCTL_LOG'], 'a', encoding='utf-8') as out:\n"
        "    out.write(json.dumps(args) + '\\n')\n"
        "state_path = pathlib.Path(os.environ['LAUNCHCTL_STATE'])\n"
        "loaded = set(json.loads(state_path.read_text(encoding='utf-8')))\n"
        "op = args[0] if args else ''\n"
        "if op == 'print':\n"
        "    label = args[1].rsplit('/', 1)[-1]\n"
        "    raise SystemExit(0 if label in loaded else 3)\n"
        "plist_path = pathlib.Path(args[-1]) if args else pathlib.Path()\n"
        "if op == 'bootstrap':\n"
        "    marker = os.environ.get('FAIL_BOOTSTRAP_ONCE')\n"
        "    if marker and not pathlib.Path(marker).exists():\n"
        "        pathlib.Path(marker).write_text('failed once', encoding='utf-8')\n"
        "        raise SystemExit(37)\n"
        "    with plist_path.open('rb') as source: label = plistlib.load(source)['Label']\n"
        "    loaded.add(label)\n"
        "elif op == 'bootout':\n"
        "    if plist_path.exists():\n"
        "        with plist_path.open('rb') as source: label = plistlib.load(source)['Label']\n"
        "    else:\n"
        "        label = args[-2].rsplit('/', 1)[-1]\n"
        "    loaded.discard(label)\n"
        "state_path.write_text(json.dumps(sorted(loaded)) + '\\n', encoding='utf-8')\n"
        "raise SystemExit(0)\n",
        encoding="utf-8",
    )
    (fake_bin / "launchctl").chmod(0o755)

    app_bundle = work / 'App bundle "Quoted" $cash %ready.app'
    binary_dir = app_bundle / "Contents" / "MacOS"
    resources_dir = app_bundle / "Contents" / "Resources"
    binary_dir.mkdir(parents=True)
    resources_dir.mkdir(parents=True)
    app_proxy = binary_dir / "arch-notebook"
    app_proxy.write_text(
        "#!/usr/bin/env python3\n"
        "import json, os, sys\n"
        "with open(os.environ['APP_COMMANDS_LOG'], 'a', encoding='utf-8') as out:\n"
        "    out.write(json.dumps(sys.argv[1:]) + '\\n')\n"
        f"os.execv({str(binary)!r}, [{str(binary)!r}, *sys.argv[1:]])\n",
        encoding="utf-8",
    )
    app_proxy.chmod(0o755)
    bundled_runner = resources_dir / "run-macos-automation.sh"
    bundled_runner.write_bytes(runner.read_bytes())
    bundled_runner.chmod(0o755)

    base_env = os.environ.copy()
    for key in (
        "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE",
        "ARCH_NOTEBOOK_AUTOMATION_WEEKDAY",
        "ARCH_NOTEBOOK_AUTOMATION_MONTH_DAY",
        "ARCH_NOTEBOOK_AUTOMATION_TIME",
        "ARCH_NOTEBOOK_CHART_EXPORT_DIR",
        "ARCH_NOTEBOOK_REQUIRE_USABLE_CONNECTIONS",
        "FAIL_BOOTSTRAP_ONCE",
    ):
        base_env.pop(key, None)
    base_env.update(
        {
            "HOME": str(home),
            "PATH": f"{fake_bin}:{os.environ.get('PATH', '/usr/bin:/bin')}",
            "LAUNCHCTL_LOG": str(launch_log),
            "LAUNCHCTL_STATE": str(launch_state),
            "APP_COMMANDS_LOG": str(app_log),
            "LC_ALL": "C",
        }
    )

    export_dir = work / 'exports "quoted" $cash %done'
    export_dir.mkdir()
    case_specs = [
        ("15-minute", {"ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "15m"}, {"StartInterval": 900}),
        ("hourly", {"ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "hourly"}, {"StartInterval": 3600}),
        (
            "daily",
            {"ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "daily", "ARCH_NOTEBOOK_AUTOMATION_TIME": "03:25"},
            {"StartCalendarInterval": {"Hour": 3, "Minute": 25}},
        ),
        (
            "weekly",
            {
                "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "weekly",
                "ARCH_NOTEBOOK_AUTOMATION_TIME": "15:45",
                "ARCH_NOTEBOOK_AUTOMATION_WEEKDAY": "2",
                "ARCH_NOTEBOOK_REQUIRE_USABLE_CONNECTIONS": "false",
            },
            {"StartCalendarInterval": {"Weekday": 2, "Hour": 15, "Minute": 45}},
        ),
        (
            "monthly",
            {
                "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "monthly",
                "ARCH_NOTEBOOK_AUTOMATION_TIME": "23:59",
                "ARCH_NOTEBOOK_AUTOMATION_MONTH_DAY": "28",
            },
            {"StartCalendarInterval": {"Day": 28, "Hour": 23, "Minute": 59}},
        ),
    ]

    installed: list[dict[str, object]] = []
    for index, (cadence, settings, expected_schedule) in enumerate(case_specs):
        case_path = work / f"Case {index} 'analyst' $literal %.case.json"
        case_path.write_text(json.dumps(case_payload, indent=2) + "\n", encoding="utf-8")
        schedule_env = base_env | settings
        if cadence == "hourly":
            schedule_env["ARCH_NOTEBOOK_CHART_EXPORT_DIR"] = str(export_dir)
        install = run(
            ["bash", str(installer), str(case_path), str(app_bundle)], env=schedule_env
        )
        if "Installed and loaded " not in install.stdout:
            raise RuntimeError(f"LaunchAgent install did not report success for {cadence}: {install.stdout}")

        label = f"com.archnotebook.automation.{canonical_schedule_id(case_path)}"
        plist_path = agent_dir / f"{label}.plist"
        with plist_path.open("rb") as source:
            settings_plist = plistlib.load(source)
        expected_args = [str(bundled_runner), str(app_proxy), str(case_path.resolve()), str(home / "Library" / "Logs" / "ArchNotebook" / canonical_schedule_id(case_path))]
        if cadence == "hourly":
            expected_args.append(str(export_dir.resolve()))
        if settings_plist.get("ProgramArguments") != expected_args:
            raise RuntimeError(f"LaunchAgent arguments changed for {cadence}: {settings_plist.get('ProgramArguments')!r}")
        if settings_plist.get("Label") != label or settings_plist.get("RunAtLoad") is not True:
            raise RuntimeError(f"LaunchAgent identity or RunAtLoad changed for {cadence}")
        if label not in json.loads(launch_state.read_text(encoding="utf-8")):
            raise RuntimeError(f"stub launchctl did not load the {cadence} agent")
        for key, expected in expected_schedule.items():
            if settings_plist.get(key) != expected:
                raise RuntimeError(f"unexpected {key} for {cadence}: {settings_plist.get(key)!r}")
        expected_policy = "false" if cadence == "weekly" else "true"
        if settings_plist.get("EnvironmentVariables", {}).get(
            "ARCH_NOTEBOOK_INSTALL_PREFLIGHT_REQUIRE_USABLE_CONNECTIONS"
        ) != expected_policy:
            raise RuntimeError(f"preflight policy was not saved for {cadence}")

        status = run(["bash", str(installer), "--status", str(case_path)], env=schedule_env)
        expected_status = {
            "15-minute": "every 15 minutes",
            "hourly": "hourly",
            "daily": "daily",
            "weekly": "weekly",
            "monthly": "monthly",
        }[cadence]
        if expected_status not in status.stdout or "loaded" not in status.stdout:
            raise RuntimeError(f"status did not restore {cadence} schedule state: {status.stdout}")
        if cadence == "daily" and "03:25" not in status.stdout:
            raise RuntimeError(f"daily schedule time missing from status: {status.stdout}")
        if cadence == "weekly" and not all(
            value in status.stdout for value in ("Tuesday", "15:45", "requires usable connections: false")
        ):
            raise RuntimeError(f"weekly schedule details missing from status: {status.stdout}")
        if cadence == "monthly" and not all(value in status.stdout for value in ("day: 28", "23:59")):
            raise RuntimeError(f"monthly schedule details missing from status: {status.stdout}")
        if cadence == "hourly" and str(export_dir.resolve()) not in status.stdout:
            raise RuntimeError(f"schedule status lost its export directory: {status.stdout}")
        installed.append({"cadence": cadence, "case": case_path, "label": label, "plist": plist_path})

    initial_files = plist_paths(agent_dir)
    if len(initial_files) != len(case_specs):
        raise RuntimeError(f"expected one case-specific LaunchAgent per case, found {sorted(initial_files)}")
    initial_calls = [json.loads(line) for line in app_log.read_text(encoding="utf-8").splitlines()]
    if len(initial_calls) != len(case_specs) or any(
        call[0] != "--check-enabled-job-queries"
        or "--progress" not in call
        or (("--require-usable" in call) != (index != 3))
        for index, call in enumerate(initial_calls)
    ):
        raise RuntimeError(f"LaunchAgent install used unexpected SQL preflight arguments: {initial_calls!r}")

    invalid_query_case = work / "invalid-query.case.json"
    invalid_payload = json.loads(json.dumps(case_payload))
    invalid_payload["automation_jobs"][0]["query"] = "SELECT * FROM missing_acceptance_table"
    invalid_query_case.write_text(json.dumps(invalid_payload, indent=2) + "\n", encoding="utf-8")
    calls_before_preflight_failure = launch_log.read_bytes()
    failure = run(
        ["bash", str(installer), str(invalid_query_case), str(app_bundle)],
        env=base_env | {"ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "daily"},
        expected=1,
    )
    if "failed preflight" not in failure.stderr.lower():
        raise RuntimeError(f"invalid SQL preflight failure was not actionable: {failure.stderr}")
    if plist_paths(agent_dir) != initial_files or launch_log.read_bytes() != calls_before_preflight_failure:
        raise RuntimeError("failed SQL preflight changed LaunchAgents or called launchctl")

    invalid_day = run(
        ["bash", str(installer), str(installed[0]["case"]), str(app_bundle)],
        env=base_env
        | {
            "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "monthly",
            "ARCH_NOTEBOOK_AUTOMATION_MONTH_DAY": "29",
        },
        expected=2,
    )
    if "between 1 and 28" not in invalid_day.stderr or plist_paths(agent_dir) != initial_files:
        raise RuntimeError("invalid monthly date was not rejected before changing schedules")

    rollback_case = installed[1]
    rollback_plist = rollback_case["plist"]
    prior_plist_bytes = rollback_plist.read_bytes()
    rollback_marker = work / "fail-bootstrap-once"
    rollback = run(
        ["bash", str(installer), str(rollback_case["case"]), str(app_bundle)],
        env=base_env
        | {
            "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "daily",
            "ARCH_NOTEBOOK_AUTOMATION_TIME": "14:22",
            "FAIL_BOOTSTRAP_ONCE": str(rollback_marker),
        },
        expected=1,
    )
    if "Could not load the new LaunchAgent" not in rollback.stderr:
        raise RuntimeError(f"LaunchAgent load failure was not reported: {rollback.stderr}")
    if rollback_plist.read_bytes() != prior_plist_bytes:
        raise RuntimeError("failed schedule replacement did not restore the previous plist")
    if rollback_case["label"] not in json.loads(launch_state.read_text(encoding="utf-8")):
        raise RuntimeError("failed schedule replacement did not reload the previous LaunchAgent")

    failed_first_case = work / "first-install-failure.case.json"
    failed_first_case.write_text(json.dumps(case_payload, indent=2) + "\n", encoding="utf-8")
    failed_first_marker = work / "fail-first-bootstrap-once"
    failed_first = run(
        ["bash", str(installer), str(failed_first_case), str(app_bundle)],
        env=base_env | {"FAIL_BOOTSTRAP_ONCE": str(failed_first_marker)},
        expected=1,
    )
    failed_first_label = f"com.archnotebook.automation.{canonical_schedule_id(failed_first_case)}"
    if "Could not load the new LaunchAgent" not in failed_first.stderr:
        raise RuntimeError(f"initial load failure was not reported: {failed_first.stderr}")
    if (agent_dir / f"{failed_first_label}.plist").exists():
        raise RuntimeError("failed first install left a LaunchAgent plist behind")
    if failed_first_label in json.loads(launch_state.read_text(encoding="utf-8")):
        raise RuntimeError("failed first install left a loaded LaunchAgent")

    remove_case = installed[0]
    run(["bash", str(installer), "--remove", str(remove_case["case"])], env=base_env)
    if Path(remove_case["plist"]).exists():
        raise RuntimeError("case-specific removal left its LaunchAgent plist")
    for entry in installed[1:]:
        if not Path(entry["plist"]).exists():
            raise RuntimeError(f"removing one case removed another schedule: {entry['cadence']}")
    if remove_case["label"] in json.loads(launch_state.read_text(encoding="utf-8")):
        raise RuntimeError("case-specific removal did not boot out the target agent")

    legacy_case = installed[2]
    legacy_plist = agent_dir / "com.archnotebook.automation.plist"
    legacy_payload = plistlib.loads(Path(legacy_case["plist"]).read_bytes())
    legacy_payload["Label"] = "com.archnotebook.automation"
    legacy_plist.write_bytes(plistlib.dumps(legacy_payload, fmt=plistlib.FMT_XML, sort_keys=True))
    Path(legacy_case["plist"]).unlink()
    loaded = set(json.loads(launch_state.read_text(encoding="utf-8")))
    loaded.discard(legacy_case["label"])
    loaded.add("com.archnotebook.automation")
    launch_state.write_text(json.dumps(sorted(loaded)) + "\n", encoding="utf-8")
    legacy_status = run(
        ["bash", str(installer), "--status", str(legacy_case["case"])], env=base_env
    )
    if "daily" not in legacy_status.stdout or "03:25" not in legacy_status.stdout:
        raise RuntimeError(f"case status did not discover a matching legacy schedule: {legacy_status.stdout}")
    run(["bash", str(installer), "--remove", str(legacy_case["case"])], env=base_env)
    if legacy_plist.exists() or "com.archnotebook.automation" in json.loads(
        launch_state.read_text(encoding="utf-8")
    ):
        raise RuntimeError("case-specific removal did not remove the matching legacy schedule")

    scheduled_entry = installed[1]
    scheduled_case_path = Path(scheduled_entry["case"])
    scheduled_log_dir = home / "Library" / "Logs" / "ArchNotebook" / canonical_schedule_id(scheduled_case_path)
    source_database_before = hashlib.sha256(source_database.read_bytes()).hexdigest()
    run(
        [
            str(bundled_runner),
            str(app_proxy),
            str(scheduled_case_path),
            str(scheduled_log_dir),
            str(export_dir),
        ],
        env=base_env,
    )
    scheduled_case_after = json.loads(scheduled_case_path.read_text(encoding="utf-8"))
    scheduled_runs = scheduled_case_after.get("automation_runs", [])
    if not scheduled_runs or not scheduled_runs[-1].get("success") or scheduled_runs[-1].get("trigger") != "scheduled":
        raise RuntimeError("LaunchAgent runner did not save a successful scheduled-trigger run")
    scheduled_snapshots = scheduled_case_after.get("snapshots", [])
    if not scheduled_snapshots or not scheduled_snapshots[-1].get("connections"):
        raise RuntimeError("LaunchAgent runner did not save a connection snapshot")
    scheduled_ids = sorted(
        connection.get("connection_id")
        for connection in scheduled_snapshots[-1]["connections"]
    )
    if scheduled_ids != ["WINDOW-START"]:
        raise RuntimeError(
            "LaunchAgent rolling-month query returned unexpected boundary rows: "
            f"{scheduled_ids} (start={window_start}, end={window_end})"
        )
    scheduled_csvs = list(export_dir.glob("*.csv"))
    if len(scheduled_csvs) != 1:
        raise RuntimeError(f"LaunchAgent runner did not export one chart CSV: {scheduled_csvs}")
    with scheduled_csvs[0].open(newline="", encoding="utf-8") as source:
        scheduled_csv_rows = list(csv.DictReader(source))
    if len(scheduled_csv_rows) != 1:
        raise RuntimeError(f"scheduled chart CSV has unexpected rows: {len(scheduled_csv_rows)}")
    source_database_after = hashlib.sha256(source_database.read_bytes()).hexdigest()
    if source_database_after != source_database_before:
        raise RuntimeError("scheduled LaunchAgent capture modified its SQLite source")
    app_calls = [json.loads(line) for line in app_log.read_text(encoding="utf-8").splitlines()]
    scheduled_app_call = app_calls[-1]
    if scheduled_app_call != [
        "--run-enabled-jobs",
        str(scheduled_case_path.resolve()),
        "--progress",
        "--resume-incomplete-scheduled-batch",
        "--export-chart-csv-dir",
        str(export_dir.resolve()),
    ]:
        raise RuntimeError(f"LaunchAgent runner invoked unexpected application arguments: {scheduled_app_call!r}")

    launch_calls = [json.loads(line) for line in launch_log.read_text(encoding="utf-8").splitlines()]
    report = {
        "schema_version": 1,
        "result": "PASS",
        "cadences": [cadence for cadence, _, _ in case_specs],
        "case_specific_launchagents": len(case_specs),
        "real_arch_notebook_preflight": True,
        "special_character_paths_preserved": True,
        "invalid_preflight_and_schedule_leave_agents_unchanged": True,
        "failed_replacement_restores_prior_plist_and_agent": True,
        "failed_first_install_leaves_no_agent": True,
        "case_removal_and_legacy_schedule_compatibility": True,
        "generated_launchagent_runner": {
            "trigger": scheduled_runs[-1]["trigger"],
            "included_connection_ids": scheduled_ids,
            "excluded_connection_ids": ["BEFORE-WINDOW", "WINDOW-END"],
            "window_start_utc": window_start,
            "window_end_utc_exclusive": window_end,
            "chart_csv_rows": len(scheduled_csv_rows),
            "source_database_unchanged": source_database_after == source_database_before,
        },
        "launchctl_is_temporary_stub": True,
        "real_macos_launchd_untouched": True,
        "launchctl_calls": len(launch_calls),
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
