#!/usr/bin/env python3
"""Exercise Linux SQL schedule installation in temporary HOME/config roots."""

from __future__ import annotations

import argparse
import csv
import json
import os
import shlex
import sqlite3
import subprocess
import tempfile
from pathlib import Path


def run(command: list[str], *, env: dict[str, str], expected: int = 0) -> subprocess.CompletedProcess[str]:
    process = subprocess.run(command, env=env, text=True, capture_output=True, check=False)
    if process.returncode != expected:
        raise RuntimeError(
            f"command expected exit {expected}, got {process.returncode}: "
            f"{command!r}\nstdout:\n{process.stdout}\nstderr:\n{process.stderr}"
        )
    return process


def unit_paths(unit_dir: Path) -> set[str]:
    return {path.name for path in unit_dir.glob("arch-notebook-automation-*")}


def decode_exec_start(service: Path) -> list[str]:
    line = next(
        (line for line in service.read_text(encoding="utf-8").splitlines() if line.startswith("ExecStart=")),
        None,
    )
    if line is None:
        raise RuntimeError(f"service has no ExecStart: {service}")
    return [part.replace("%%", "%").replace("$$", "$") for part in shlex.split(line.partition("=")[2])]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--installer", required=True, type=Path)
    parser.add_argument("--source-sql", required=True, type=Path)
    parser.add_argument("--sql-acceptance", required=True, type=Path)
    parser.add_argument("--work-dir", type=Path, help="retain artifacts in this new directory")
    args = parser.parse_args()
    binary = args.binary.resolve()
    installer = args.installer.resolve()
    source_sql = args.source_sql.resolve()
    sql_acceptance = args.sql_acceptance.resolve()
    for label, path in (("binary", binary), ("installer", installer), ("SQL fixture", source_sql), ("SQL acceptance script", sql_acceptance)):
        if not path.is_file():
            parser.error(f"{label} does not exist: {path}")

    temporary = None
    if args.work_dir:
        work = args.work_dir.resolve()
        work.mkdir(parents=True, exist_ok=False)
    else:
        temporary = tempfile.TemporaryDirectory(prefix="arch-notebook-scheduler-acceptance-")
        work = Path(temporary.name)

    fixture_work = work / "sql-fixture"
    fixture_report = run(
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
    if json.loads(fixture_report.stdout).get("result") != "PASS":
        raise RuntimeError("synthetic SQL fixture acceptance did not pass")
    case_template = fixture_work / "automation.case.json"
    case_payload = json.loads(case_template.read_text(encoding="utf-8"))
    scheduler_database = work / "scheduler.sqlite"
    database = sqlite3.connect(scheduler_database)
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
                "SCHEDULER-ACCEPTANCE",
                "synthetic calendar boundary",
            )
            for connection_id, source, target, event_date in (
                ("BEFORE-WINDOW", "Prior Boundary", "Prior Place", before_window),
                ("WINDOW-START", "Window Start", "Included Place", window_start),
                ("WINDOW-END", "Window End", "Excluded Place", window_end),
            )
        ]
        database.executemany(
            "INSERT INTO connection_observations VALUES "
            "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            scheduled_rows,
        )
        database.commit()
    finally:
        database.close()
    case_payload["automation_jobs"][0]["database_path"] = str(scheduler_database)
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
    config_home = work / "config"
    data_home = work / "data"
    bin_dir = work / "bin"
    home.mkdir()
    config_home.mkdir()
    data_home.mkdir()
    bin_dir.mkdir()
    unit_dir = config_home / "systemd" / "user"
    systemctl_log = work / "systemctl.jsonl"
    app_log = work / "app-commands.jsonl"

    systemctl = bin_dir / "systemctl"
    systemctl.write_text(
        "#!/usr/bin/env python3\n"
        "import json, os, sys\n"
        "with open(os.environ['SYSTEMCTL_LOG'], 'a', encoding='utf-8') as out:\n"
        "    out.write(json.dumps(sys.argv[1:]) + '\\n')\n"
        "args = sys.argv[1:]\n"
        "raise SystemExit(0)\n",
        encoding="utf-8",
    )
    systemctl.chmod(0o755)

    proxy = bin_dir / "arch-notebook-test-proxy"
    proxy.write_text(
        "#!/usr/bin/env python3\n"
        "import json, os, sys\n"
        "with open(os.environ['APP_COMMANDS_LOG'], 'a', encoding='utf-8') as out:\n"
        "    out.write(json.dumps(sys.argv[1:]) + '\\n')\n"
        f"os.execv({str(binary)!r}, [{str(binary)!r}, *sys.argv[1:]])\n",
        encoding="utf-8",
    )
    proxy.chmod(0o755)

    env = os.environ.copy()
    env.update(
        {
            "HOME": str(home),
            "XDG_CONFIG_HOME": str(config_home),
            "XDG_DATA_HOME": str(data_home),
            "PATH": f"{bin_dir}:/usr/bin:/bin",
            "SYSTEMCTL_LOG": str(systemctl_log),
            "APP_COMMANDS_LOG": str(app_log),
            "LC_ALL": "C.UTF-8",
        }
    )

    case_specs = [
        ("hourly", {"ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "hourly"}, "OnCalendar=hourly"),
        ("quarter-hour", {"ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "15m"}, "OnCalendar=*:0/15"),
        (
            "daily",
            {
                "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "daily",
                "ARCH_NOTEBOOK_AUTOMATION_TIME": "03:25",
            },
            "OnCalendar=*-*-* 03:25:00",
        ),
        (
            "weekly",
            {
                "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "weekly",
                "ARCH_NOTEBOOK_AUTOMATION_TIME": "15:45",
                "ARCH_NOTEBOOK_AUTOMATION_WEEKDAY": "2",
                "ARCH_NOTEBOOK_REQUIRE_USABLE_CONNECTIONS": "false",
            },
            "OnCalendar=Tue *-*-* 15:45:00",
        ),
        (
            "monthly",
            {
                "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "monthly",
                "ARCH_NOTEBOOK_AUTOMATION_TIME": "23:59",
                "ARCH_NOTEBOOK_AUTOMATION_MONTH_DAY": "28",
            },
            "OnCalendar=*-*-28 23:59:00",
        ),
    ]
    installed: list[dict[str, str]] = []
    exported_dir = work / 'exports "quoted" $cash %done'
    exported_dir.mkdir()
    for index, (label, settings, expected_calendar) in enumerate(case_specs):
        case_path = work / f"Case {index} 'analyst' $literal %.case.json"
        case_path.write_text(json.dumps(case_payload, indent=2) + "\n", encoding="utf-8")
        schedule_env = env | settings
        if label == "hourly":
            schedule_env = schedule_env | {"ARCH_NOTEBOOK_CHART_EXPORT_DIR": str(exported_dir)}
        before = unit_paths(unit_dir)
        install = run(
            ["bash", str(installer), str(case_path), str(proxy)], env=schedule_env
        )
        if "Enabled " not in install.stdout:
            raise RuntimeError(f"scheduler install did not report success for {label}")
        after = unit_paths(unit_dir)
        added = after - before
        if len(added) != 2:
            raise RuntimeError(f"expected one service and timer for {label}, created {sorted(added)}")
        timer_path = next(unit_dir.glob("*.timer")) if index == 0 else next(
            path for path in unit_dir.glob("*.timer") if path.name not in {entry["timer"] for entry in installed}
        )
        service_path = unit_dir / f"{timer_path.stem}.service"
        timer_text = timer_path.read_text(encoding="utf-8")
        service_text = service_path.read_text(encoding="utf-8")
        if expected_calendar not in timer_text or "Persistent=true" not in timer_text:
            raise RuntimeError(f"unexpected timer contents for {label}: {timer_text}")
        if "Type=oneshot" not in service_text or "TimeoutStartSec=infinity" not in service_text:
            raise RuntimeError(f"service lacks bounded scheduler semantics for {label}")
        argv = decode_exec_start(service_path)
        if argv[:4] != [str(proxy), "--run-enabled-jobs", str(case_path.resolve()), "--progress"]:
            raise RuntimeError(f"service ExecStart arguments changed for {label}: {argv}")
        if label == "hourly":
            if argv[4:6] != ["--export-chart-csv-dir", str(exported_dir.resolve())]:
                raise RuntimeError(f"special-character export path was not preserved: {argv}")
        if f"Environment=ARCH_NOTEBOOK_INSTALL_PREFLIGHT_REQUIRE_USABLE_CONNECTIONS={'false' if label == 'weekly' else 'true'}" not in service_text:
            raise RuntimeError(f"preflight policy was not recorded for {label}")

        status = run(["bash", str(installer), "--status", str(case_path)], env=schedule_env)
        if label == "weekly" and not all(
            value in status.stdout for value in ("weekly", "Tuesday", "15:45", "requires usable connections: false")
        ):
            raise RuntimeError(f"weekly schedule status did not restore its saved policy: {status.stdout}")
        if label == "hourly" and str(exported_dir.resolve()) not in status.stdout:
            raise RuntimeError(f"schedule status lost its export directory: {status.stdout}")
        installed.append(
            {
                "label": label,
                "case": str(case_path),
                "timer": timer_path.name,
                "service": str(service_path),
            }
        )

    app_calls = [json.loads(line) for line in app_log.read_text(encoding="utf-8").splitlines()]
    expected_app_calls = len(case_specs)
    if len(app_calls) != expected_app_calls or any(
        len(call) < 3
        or call[0] != "--check-enabled-job-queries"
        or call[2] != "--progress"
        or (("--require-usable" in call) != (index != 3))
        for index, call in enumerate(app_calls)
    ):
        raise RuntimeError(f"installer SQL preflight did not use the expected CLI gates: {app_calls}")

    scheduled_case_path = Path(installed[0]["case"])
    scheduled_case_before = json.loads(scheduled_case_path.read_text(encoding="utf-8"))
    scheduled_runs_before = len(scheduled_case_before.get("automation_runs", []))
    scheduled_service_env = env | {"ARCH_NOTEBOOK_AUTOMATION_TRIGGER": "scheduled"}
    scheduled_service = run(
        decode_exec_start(Path(installed[0]["service"])), env=scheduled_service_env
    )
    scheduled_case_after = json.loads(scheduled_case_path.read_text(encoding="utf-8"))
    scheduled_runs_after = scheduled_case_after.get("automation_runs", [])
    if len(scheduled_runs_after) != scheduled_runs_before + 1:
        raise RuntimeError("generated service command did not append exactly one automation run")
    if not scheduled_runs_after[-1].get("success") or scheduled_runs_after[-1].get("trigger") != "scheduled":
        raise RuntimeError(
            "generated service command did not save a successful scheduled-trigger run"
        )
    scheduled_snapshots = scheduled_case_after.get("snapshots", [])
    if not scheduled_snapshots or not scheduled_snapshots[-1].get("connections"):
        raise RuntimeError("generated service command did not save a connection snapshot")
    scheduled_connection_ids = sorted(
        connection.get("connection_id")
        for connection in scheduled_snapshots[-1]["connections"]
    )
    if scheduled_connection_ids != ["WINDOW-START"]:
        raise RuntimeError(
            "scheduled UTC month query did not include only the half-open window start: "
            f"{scheduled_connection_ids} (start={window_start}, end={window_end})"
        )
    scheduled_csv_files = list(exported_dir.glob("*.csv"))
    if len(scheduled_csv_files) != 1:
        raise RuntimeError(
            f"scheduled service command did not export exactly one chart CSV: {scheduled_csv_files}"
        )
    with scheduled_csv_files[0].open(newline="", encoding="utf-8") as source:
        scheduled_csv_rows = list(csv.DictReader(source))
    if len(scheduled_csv_rows) != len(scheduled_snapshots[-1]["connections"]):
        raise RuntimeError("scheduled chart CSV row count differs from its saved snapshot")

    invalid_case = work / "invalid-preflight.case.json"
    invalid_payload = json.loads(json.dumps(case_payload))
    invalid_payload["automation_jobs"][0]["query"] = "SELECT * FROM missing_acceptance_table"
    invalid_case.write_text(json.dumps(invalid_payload, indent=2) + "\n", encoding="utf-8")
    installed_before_failure = unit_paths(unit_dir)
    systemctl_before_failure = systemctl_log.read_bytes()
    failed_preflight_env = env | {"ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "daily"}
    failure = run(
        ["bash", str(installer), str(invalid_case), str(proxy)],
        env=failed_preflight_env,
        expected=1,
    )
    if "failed preflight" not in failure.stderr.lower():
        raise RuntimeError(f"invalid query failure was not actionable: {failure.stderr}")
    if unit_paths(unit_dir) != installed_before_failure or systemctl_log.read_bytes() != systemctl_before_failure:
        raise RuntimeError("failed SQL preflight changed installed schedules or called systemctl")

    invalid_schedule_env = env | {
        "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE": "monthly",
        "ARCH_NOTEBOOK_AUTOMATION_MONTH_DAY": "29",
    }
    invalid_schedule = run(
        ["bash", str(installer), installed[0]["case"], str(proxy)],
        env=invalid_schedule_env,
        expected=2,
    )
    if "between 1 and 28" not in invalid_schedule.stderr:
        raise RuntimeError(f"invalid monthly date was not rejected clearly: {invalid_schedule.stderr}")
    if unit_paths(unit_dir) != installed_before_failure:
        raise RuntimeError("invalid schedule changed installed unit files")

    first_case = installed[0]["case"]
    run(["bash", str(installer), "--remove", first_case], env=env)
    remaining = unit_paths(unit_dir)
    removed_timer = installed[0]["timer"]
    if removed_timer in remaining or removed_timer.removesuffix(".timer") + ".service" in remaining:
        raise RuntimeError("removing one case did not remove its service and timer")
    for entry in installed[1:]:
        if entry["timer"] not in remaining or entry["timer"].removesuffix(".timer") + ".service" not in remaining:
            raise RuntimeError(f"removing one case removed another schedule: {entry}")
    removal_log = [json.loads(line) for line in systemctl_log.read_text(encoding="utf-8").splitlines()]
    if not any(call[1:3] == ["disable", "--now"] and call[-1] == removed_timer for call in removal_log):
        raise RuntimeError(f"case-specific schedule removal was not invoked: {removal_log}")

    report = {
        "schema_version": 1,
        "result": "PASS",
        "cadences": [label for label, _, _ in case_specs],
        "case_isolated_unit_pairs": len(installed),
        "special_character_paths_preserved": True,
        "preflight_uses_real_arch_notebook_cli": True,
        "generated_service_invocation": {
            "exit_code": scheduled_service.returncode,
            "trigger": scheduled_runs_after[-1]["trigger"],
            "saved_connections": len(scheduled_snapshots[-1]["connections"]),
            "chart_csv_rows": len(scheduled_csv_rows),
        },
        "dynamic_calendar_window": {
            "window_start_utc": window_start,
            "window_end_utc_exclusive": window_end,
            "included_connection_ids": scheduled_connection_ids,
            "excluded_connection_ids": ["BEFORE-WINDOW", "WINDOW-END"],
        },
        "failed_sql_preflight_installed_nothing": True,
        "invalid_month_day_rejected": True,
        "case_specific_removal_preserved_other_schedules": True,
        "systemctl_is_temporary_stub": True,
        "systemctl_calls": len(removal_log),
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
