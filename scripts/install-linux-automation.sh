#!/usr/bin/env bash
set -euo pipefail

legacy_schedule_matches_case() {
  python3 - "$1/arch-notebook-automation.service" "$2" <<'PY'
import os
import shlex
import sys

service_path, case_path = sys.argv[1:]
try:
    with open(service_path, encoding="utf-8") as source:
        lines = source.readlines()
except OSError:
    raise SystemExit(1)
for line in lines:
    if not line.startswith("ExecStart="):
        continue
    try:
        arguments = shlex.split(line.partition("=")[2])
    except ValueError:
        continue
    if len(arguments) < 3 or arguments[1] != "--run-enabled-jobs":
        continue
    configured_case = arguments[2].replace("%%", "%").replace("$$", "$")
    if os.path.realpath(configured_case) == os.path.realpath(case_path):
        raise SystemExit(0)
raise SystemExit(1)
PY
}

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "This installer is for Linux." >&2
  exit 2
fi

if [[ "${1:-}" == "--remove" ]]; then
  if [[ $# -gt 2 ]]; then
    echo "Usage: $0 --remove [case-file]" >&2
    exit 2
  fi
  home_dir="${HOME:?HOME must be set}"
  case "$home_dir" in
    /*) ;;
    *) echo "HOME must be an absolute path." >&2; exit 1 ;;
  esac
  config_home="${XDG_CONFIG_HOME:-$home_dir/.config}"
  case "$config_home" in
    /*) ;;
    *) echo "XDG_CONFIG_HOME must be an absolute path." >&2; exit 1 ;;
  esac
  schedule_name="arch-notebook-automation"
  if [[ $# -eq 2 ]]; then
    if ! command -v python3 >/dev/null 2>&1; then
      echo "python3 is required to identify the case schedule." >&2
      exit 1
    fi
    case_path="$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$2")"
    schedule_id="$(python3 - "$case_path" <<'PY'
import hashlib
import sys
print(hashlib.sha256(sys.argv[1].encode()).hexdigest()[:12])
PY
)"
    schedule_name+="-$schedule_id"
    if [[ ! -e "$config_home/systemd/user/$schedule_name.timer" ]] && legacy_schedule_matches_case "$config_home/systemd/user" "$case_path"; then
      schedule_name="arch-notebook-automation"
    fi
  fi
  unit_dir="$config_home/systemd/user"
  if command -v systemctl >/dev/null 2>&1; then
    systemctl --user disable --now "$schedule_name.timer" >/dev/null 2>&1 || true
  fi
  rm -f "$unit_dir/$schedule_name.service" "$unit_dir/$schedule_name.timer"
  if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload >/dev/null 2>&1 || true
  fi
  if [[ $# -eq 2 ]]; then
    echo "Removed the Arch Notebook SQL automation schedule for: $case_path"
  else
    echo "Removed the legacy Arch Notebook SQL automation schedule for this user."
  fi
  exit 0
fi

if [[ "${1:-}" == "--status" ]]; then
  if [[ $# -gt 2 ]]; then
    echo "Usage: $0 --status [case-file]" >&2
    exit 2
  fi
  home_dir="${HOME:?HOME must be set}"
  case "$home_dir" in
    /*) ;;
    *) echo "HOME must be an absolute path." >&2; exit 1 ;;
  esac
  config_home="${XDG_CONFIG_HOME:-$home_dir/.config}"
  case "$config_home" in
    /*) ;;
    *) echo "XDG_CONFIG_HOME must be an absolute path." >&2; exit 1 ;;
  esac
  schedule_name="arch-notebook-automation"
  if [[ $# -eq 2 ]]; then
    if ! command -v python3 >/dev/null 2>&1; then
      echo "python3 is required to identify the case schedule." >&2
      exit 1
    fi
    case_path="$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$2")"
    schedule_id="$(python3 - "$case_path" <<'PY'
import hashlib
import sys
print(hashlib.sha256(sys.argv[1].encode()).hexdigest()[:12])
PY
)"
    schedule_name+="-$schedule_id"
    if [[ ! -r "$config_home/systemd/user/$schedule_name.timer" ]] && legacy_schedule_matches_case "$config_home/systemd/user" "$case_path"; then
      schedule_name="arch-notebook-automation"
    fi
  fi
  timer_file="$config_home/systemd/user/$schedule_name.timer"
  if [[ ! -r "$timer_file" ]]; then
    if [[ $# -eq 2 ]]; then
      echo "No Arch Notebook SQL automation schedule is installed for: $case_path"
    else
      echo "No legacy Arch Notebook SQL automation schedule is installed for this user."
    fi
    echo "Chart CSV export directory: disabled"
    exit 0
  fi
  cadence="$(awk -F= '/^OnCalendar=/ { print $2; exit }' "$timer_file")"
  schedule_time=""
  schedule_weekday=""
  schedule_day=""
  if [[ "$cadence" == "*:0/15" ]]; then
    cadence="every 15 minutes"
  elif [[ "$cadence" =~ ^\*-\*-\*[[:space:]]+(([01][0-9]|2[0-3]):[0-5][0-9]):00$ ]]; then
    cadence="daily"
    schedule_time="${BASH_REMATCH[1]}"
  elif [[ "$cadence" =~ ^([[:alpha:]]{3})[[:space:]]+\*-\*-\*[[:space:]]+(([01][0-9]|2[0-3]):[0-5][0-9]):00$ ]]; then
    cadence="weekly"
    schedule_time="${BASH_REMATCH[2]}"
    case "${BASH_REMATCH[1]}" in
      Sun) schedule_weekday="Sunday" ;;
      Mon) schedule_weekday="Monday" ;;
      Tue) schedule_weekday="Tuesday" ;;
      Wed) schedule_weekday="Wednesday" ;;
      Thu) schedule_weekday="Thursday" ;;
      Fri) schedule_weekday="Friday" ;;
      Sat) schedule_weekday="Saturday" ;;
    esac
  elif [[ "$cadence" =~ ^\*-\*-([0-2][0-9])[[:space:]]+(([01][0-9]|2[0-3]):[0-5][0-9]):00$ ]]; then
    cadence="monthly"
    schedule_day="$((10#${BASH_REMATCH[1]}))"
    schedule_time="${BASH_REMATCH[2]}"
  elif [[ "$cadence" == "monthly" ]]; then
    schedule_time="00:00"
    schedule_day="1"
  elif [[ "$cadence" == "weekly" ]]; then
    schedule_time="00:00"
    schedule_weekday="Monday"
  elif [[ "$cadence" == "daily" ]]; then
    schedule_time="00:00"
  fi
  if [[ -z "$cadence" ]]; then cadence="unknown cadence"; fi
  chart_export_dir="$(python3 - "$config_home/systemd/user/$schedule_name.service" <<'PY'
import shlex
import sys

try:
    with open(sys.argv[1], encoding="utf-8") as source:
        line = next((line for line in source if line.startswith("ExecStart=")), "")
    arguments = shlex.split(line.partition("=")[2])
except (OSError, ValueError):
    arguments = []
try:
    export_dir_index = arguments.index("--export-chart-csv-dir")
    print(arguments[export_dir_index + 1].replace("%%", "%").replace("$$", "$"))
except (ValueError, IndexError):
    print("disabled")
PY
)"
require_usable_connections="$(awk -F= '/^Environment=ARCH_NOTEBOOK_INSTALL_PREFLIGHT_REQUIRE_USABLE_CONNECTIONS=/ { print $NF; exit }' "$config_home/systemd/user/$schedule_name.service")"
case "$require_usable_connections" in
  true|false) ;;
  *) require_usable_connections="true" ;;
esac
  if command -v systemctl >/dev/null 2>&1; then
    enabled=0
    active=0
    manager_available=0
    if systemctl --user show-environment >/dev/null 2>&1; then manager_available=1; fi
    if systemctl --user is-enabled --quiet "$schedule_name.timer"; then enabled=1; fi
    if systemctl --user is-active --quiet "$schedule_name.timer"; then active=1; fi
    timer_properties="$(systemctl --user show \
      --property=LastTriggerUSec \
      --property=NextElapseUSecRealtime \
      "$schedule_name.timer" 2>/dev/null || true)"
    last_trigger="$(awk -F= '$1 == "LastTriggerUSec" { print substr($0, index($0, "=") + 1); exit }' <<< "$timer_properties")"
    next_trigger="$(awk -F= '$1 == "NextElapseUSecRealtime" { print substr($0, index($0, "=") + 1); exit }' <<< "$timer_properties")"
    if (( ! manager_available )); then
      state="configured; systemd user manager status unavailable"
    elif (( enabled && active )); then
      state="active and enabled"
    elif (( active )); then
      state="active but not enabled"
    elif (( enabled )); then
      state="enabled but not active"
    else
      state="configured but not active or enabled"
    fi
  else
    state="configured; systemctl is unavailable"
  fi
  if command -v systemctl >/dev/null 2>&1; then
    if [[ -z "$timer_properties" ]]; then
      echo "Timer dispatch timing: unavailable"
    else
      case "${last_trigger,,}" in
        ""|n/a|0) echo "Last timer dispatch: not recorded" ;;
        *) printf 'Last timer dispatch: %s\n' "$last_trigger" ;;
      esac
      case "${next_trigger,,}" in
        ""|n/a|0) echo "Next timer dispatch: not scheduled" ;;
        *) printf 'Next timer dispatch: %s\n' "$next_trigger" ;;
      esac
    fi
  else
    echo "Timer dispatch timing: unavailable"
  fi
  if [[ $# -eq 2 ]]; then
    printf 'Arch Notebook SQL schedule for %s: %s; %s.\n' "$case_path" "$cadence" "$state"
    if [[ -n "$schedule_time" ]]; then printf 'Schedule time: %s\n' "$schedule_time"; fi
    if [[ -n "$schedule_weekday" ]]; then printf 'Schedule weekday: %s\n' "$schedule_weekday"; fi
    if [[ -n "$schedule_day" ]]; then printf 'Schedule day: %s\n' "$schedule_day"; fi
  else
    echo "Legacy Arch Notebook SQL schedule: $cadence; $state."
    if [[ -n "$schedule_time" ]]; then printf 'Schedule time: %s\n' "$schedule_time"; fi
    if [[ -n "$schedule_weekday" ]]; then printf 'Schedule weekday: %s\n' "$schedule_weekday"; fi
    if [[ -n "$schedule_day" ]]; then printf 'Schedule day: %s\n' "$schedule_day"; fi
  fi
  printf 'Chart CSV export directory: %s\n' "$chart_export_dir"
  printf 'Schedule install preflight requires usable connections: %s\n' "$require_usable_connections"
  exit 0
fi

if [[ $# -gt 2 ]]; then
  echo "Usage: $0 [case-file] [arch-notebook-binary]" >&2
  exit 2
fi

schedule="${ARCH_NOTEBOOK_AUTOMATION_SCHEDULE:-hourly}"
schedule_label="$schedule"
schedule_time="${ARCH_NOTEBOOK_AUTOMATION_TIME:-00:00}"
schedule_weekday="${ARCH_NOTEBOOK_AUTOMATION_WEEKDAY:-1}"
schedule_day="${ARCH_NOTEBOOK_AUTOMATION_MONTH_DAY:-1}"
case "$schedule" in
  15m)
    schedule='*:0/15'
    schedule_label="every 15 minutes"
    ;;
  daily|weekly|monthly)
    if [[ ! "$schedule_time" =~ ^([01][0-9]|2[0-3]):[0-5][0-9]$ ]]; then
      echo "ARCH_NOTEBOOK_AUTOMATION_TIME must use 24-hour HH:MM format." >&2
      exit 2
    fi
    case "$schedule" in
      daily) schedule="*-*-* ${schedule_time}:00" ;;
      weekly)
        if [[ ! "$schedule_weekday" =~ ^[0-6]$ ]]; then
          echo "ARCH_NOTEBOOK_AUTOMATION_WEEKDAY must be 0 (Sunday) through 6 (Saturday)." >&2
          exit 2
        fi
        case "$schedule_weekday" in
          0) weekday_name="Sun" ; weekday_label="Sunday" ;;
          1) weekday_name="Mon" ; weekday_label="Monday" ;;
          2) weekday_name="Tue" ; weekday_label="Tuesday" ;;
          3) weekday_name="Wed" ; weekday_label="Wednesday" ;;
          4) weekday_name="Thu" ; weekday_label="Thursday" ;;
          5) weekday_name="Fri" ; weekday_label="Friday" ;;
          6) weekday_name="Sat" ; weekday_label="Saturday" ;;
        esac
        schedule="${weekday_name} *-*-* ${schedule_time}:00"
        schedule_label="weekly on ${weekday_label} at ${schedule_time}"
        ;;
      monthly)
        if [[ ! "$schedule_day" =~ ^([1-9]|1[0-9]|2[0-8])$ ]]; then
          echo "ARCH_NOTEBOOK_AUTOMATION_MONTH_DAY must be between 1 and 28." >&2
          exit 2
        fi
        printf -v schedule_day_padded '%02d' "$schedule_day"
        schedule="*-*-${schedule_day_padded} ${schedule_time}:00"
        schedule_label="monthly on day ${schedule_day} at ${schedule_time}"
        ;;
    esac
    ;;
  hourly) ;;
  *) echo "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE must be 15m, hourly, daily, weekly, or monthly." >&2; exit 2 ;;
esac

home_dir="${HOME:?HOME must be set}"
case "$home_dir" in
  /*) ;;
  *) echo "HOME must be an absolute path." >&2; exit 1 ;;
esac

data_home="${XDG_DATA_HOME:-$home_dir/.local/share}"
case "$data_home" in
  /*) ;;
  *) echo "XDG_DATA_HOME must be an absolute path." >&2; exit 1 ;;
esac

case_file="${1:-$data_home/arch-notebook/harbor-light.case.json}"
if [[ ! -f "$case_file" || ! -r "$case_file" ]]; then
  echo "Case file is not readable: $case_file" >&2
  echo "Launch Arch Notebook once to create the default case, or pass an existing case file." >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to write the systemd user units." >&2
  exit 1
fi
case_file="$(python3 - "$case_file" <<'PY'
import os
import sys
print(os.path.realpath(sys.argv[1]))
PY
)"
schedule_id="$(python3 - "$case_file" <<'PY'
import hashlib
import sys
print(hashlib.sha256(sys.argv[1].encode()).hexdigest()[:12])
PY
)"
schedule_name="arch-notebook-automation-$schedule_id"
if legacy_schedule_matches_case "${XDG_CONFIG_HOME:-$home_dir/.config}/systemd/user" "$case_file"; then
  schedule_name="arch-notebook-automation"
fi

if [[ $# -ge 2 ]]; then
  binary="$2"
else
  binary="$(command -v arch-notebook || true)"
fi
if [[ -z "$binary" || ! -x "$binary" ]]; then
  echo "Arch Notebook executable was not found. Pass its path as the second argument." >&2
  exit 1
fi
binary="$(realpath "$binary")"

chart_export_dir="${ARCH_NOTEBOOK_CHART_EXPORT_DIR:-}"
if [[ -n "$chart_export_dir" ]]; then
  case "$chart_export_dir" in
    /*) ;;
    *) echo "ARCH_NOTEBOOK_CHART_EXPORT_DIR must be an absolute path." >&2; exit 2 ;;
  esac
  if [[ ! -d "$chart_export_dir" ]]; then
    echo "ARCH_NOTEBOOK_CHART_EXPORT_DIR must name an existing directory." >&2
    exit 2
  fi
  chart_export_dir="$(realpath "$chart_export_dir")"
fi

if ! command -v systemctl >/dev/null 2>&1; then
  echo "systemctl is required to enable the user timer." >&2
  exit 1
fi

# One locked preflight checks that enabled jobs exist and their queries work.
require_usable_connections="${ARCH_NOTEBOOK_REQUIRE_USABLE_CONNECTIONS:-true}"
case "$require_usable_connections" in
  true|false) ;;
  *) echo "ARCH_NOTEBOOK_REQUIRE_USABLE_CONNECTIONS must be true or false." >&2; exit 2 ;;
esac
preflight_args=(--check-enabled-job-queries "$case_file" --progress)
if [[ "$require_usable_connections" == "true" ]]; then
  preflight_args+=(--require-usable)
fi
if ! "$binary" "${preflight_args[@]}"; then
  if [[ "$require_usable_connections" == "true" ]]; then
    echo "One or more enabled SQL jobs failed preflight or returned no usable connections; fix the job, database, or query before installing the timer." >&2
  else
    echo "One or more enabled SQL jobs failed query preflight; fix the job, database, or query before installing the timer." >&2
  fi
  exit 1
fi

config_home="${XDG_CONFIG_HOME:-$home_dir/.config}"
case "$config_home" in
  /*) ;;
  *) echo "XDG_CONFIG_HOME must be an absolute path." >&2; exit 1 ;;
esac
unit_dir="$config_home/systemd/user"
mkdir -p "$unit_dir"

python3 - "$unit_dir" "$binary" "$case_file" "$schedule" "$schedule_name" "$chart_export_dir" "$require_usable_connections" <<'PY'
import os
import sys
import tempfile

unit_dir, binary, case_file, schedule, schedule_name, chart_export_dir, require_usable_connections = sys.argv[1:]

def quote_exec_arg(value: str) -> str:
    if any(ord(character) < 32 or ord(character) == 127 for character in value):
        raise SystemExit("Paths containing control characters cannot be used in a systemd unit.")
    # systemd parses quoted arguments itself; escape quotes, specifiers and variables.
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    escaped = escaped.replace("%", "%%").replace("$", "$$")
    return f'"{escaped}"'

service = """[Unit]
Description=Run Arch Notebook SQL automation jobs

[Service]
Type=oneshot
TimeoutStartSec=infinity
Environment=ARCH_NOTEBOOK_AUTOMATION_TRIGGER=scheduled
Environment=ARCH_NOTEBOOK_INSTALL_PREFLIGHT_REQUIRE_USABLE_CONNECTIONS={require_usable_connections}
ExecStart={command}
""".format(require_usable_connections=require_usable_connections, command=" ".join(map(quote_exec_arg, [
    # Scheduler output goes to the journal; report job boundaries and long-query heartbeats.
    binary, "--run-enabled-jobs", case_file, "--progress", "--resume-incomplete-scheduled-batch",
    *( ["--export-chart-csv-dir", chart_export_dir] if chart_export_dir else [] ),
])))

timer = """[Unit]
Description=Schedule Arch Notebook SQL automation

[Timer]
OnCalendar={schedule}
Persistent=true
RandomizedDelaySec=5m
Unit={schedule_name}.service

[Install]
WantedBy=timers.target
""".format(schedule=schedule, schedule_name=schedule_name)

def replace_atomically(destination: str, contents: str) -> None:
    fd, temporary = tempfile.mkstemp(
        prefix=".arch-notebook-automation.", dir=unit_dir, text=True
    )
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as output:
            output.write(contents)
        os.chmod(temporary, 0o644)
        os.replace(temporary, destination)
    except BaseException:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass
        raise

replace_atomically(
    os.path.join(unit_dir, f"{schedule_name}.service"), service
)
replace_atomically(
    os.path.join(unit_dir, f"{schedule_name}.timer"), timer
)
PY

systemctl --user daemon-reload
systemctl --user enable --now "$schedule_name.timer"
systemctl --user restart "$schedule_name.timer"
echo "Enabled $schedule_label Arch Notebook automation for: $case_file"
echo "Schedule install preflight requires usable connections: $require_usable_connections"
echo "Check status with: systemctl --user status $schedule_name.timer"
echo "View run output with: journalctl --user -u $schedule_name.service"
