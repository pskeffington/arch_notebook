#!/usr/bin/env bash
set -euo pipefail

legacy_schedule_matches_case() {
  python3 - "$1/com.archnotebook.automation.plist" "$2" <<'PY'
import os
import plistlib
import sys

plist_path, case_path = sys.argv[1:]
try:
    with open(plist_path, "rb") as source:
        settings = plistlib.load(source)
except (OSError, plistlib.InvalidFileException):
    raise SystemExit(1)
configured_paths = settings.get("ProgramArguments", [])
raise SystemExit(
    0
    if any(os.path.realpath(value) == os.path.realpath(case_path) for value in configured_paths)
    else 1
)
PY
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This installer is for macOS." >&2
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
  label="com.archnotebook.automation"
  if [[ $# -eq 2 ]]; then
    case_path="$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$2")"
    schedule_id="$(python3 - "$case_path" <<'PY'
import hashlib
import sys
print(hashlib.sha256(sys.argv[1].encode()).hexdigest()[:12])
PY
)"
    label+=".$schedule_id"
    if [[ ! -e "$home_dir/Library/LaunchAgents/$label.plist" ]] && legacy_schedule_matches_case "$home_dir/Library/LaunchAgents" "$case_path"; then
      label="com.archnotebook.automation"
    fi
  fi
  plist_path="$home_dir/Library/LaunchAgents/$label.plist"
  launchctl bootout "gui/$(id -u)" "$plist_path" >/dev/null 2>&1 || true
  rm -f "$plist_path"
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
  label="com.archnotebook.automation"
  if [[ $# -eq 2 ]]; then
    case_path="$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$2")"
    schedule_id="$(python3 - "$case_path" <<'PY'
import hashlib
import sys
print(hashlib.sha256(sys.argv[1].encode()).hexdigest()[:12])
PY
)"
    label+=".$schedule_id"
    if [[ ! -r "$home_dir/Library/LaunchAgents/$label.plist" ]] && legacy_schedule_matches_case "$home_dir/Library/LaunchAgents" "$case_path"; then
      label="com.archnotebook.automation"
    fi
  fi
  plist_path="$home_dir/Library/LaunchAgents/$label.plist"
  if [[ ! -r "$plist_path" ]]; then
    if [[ $# -eq 2 ]]; then
      echo "No Arch Notebook SQL automation schedule is installed for: $case_path"
    else
      echo "No legacy Arch Notebook SQL automation schedule is installed for this user."
    fi
    echo "Chart CSV export directory: disabled"
    exit 0
  fi
  if [[ $# -eq 2 ]]; then
    case_path="$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$2")"
  else
    case_path="legacy schedule"
  fi
  python3 - "$plist_path" "$label" "gui/$(id -u)" "$case_path" <<'PY'
import plistlib
import subprocess
import sys

plist_path, label, domain, case_path = sys.argv[1:]
with open(plist_path, "rb") as source:
    settings = plistlib.load(source)
seconds = settings.get("StartInterval")
calendar = settings.get("StartCalendarInterval")
schedule_day = None
schedule_weekday = None
if isinstance(calendar, dict) and isinstance(calendar.get("Day"), int):
    cadence = "monthly"
    schedule_day = calendar["Day"]
    schedule_time = f"{calendar.get('Hour', 0):02d}:{calendar.get('Minute', 0):02d}"
elif isinstance(calendar, dict) and isinstance(calendar.get("Weekday"), int):
    cadence = "weekly"
    schedule_weekday = calendar["Weekday"]
    schedule_time = f"{calendar.get('Hour', 0):02d}:{calendar.get('Minute', 0):02d}"
elif isinstance(calendar, dict) and "Hour" in calendar and "Minute" in calendar:
    cadence = "daily"
    schedule_time = f"{calendar['Hour']:02d}:{calendar['Minute']:02d}"
else:
    cadence = {900: "every 15 minutes", 3600: "hourly", 86400: "daily", 604800: "weekly"}.get(seconds, f"every {seconds} seconds")
    schedule_time = "00:00" if seconds in (86400, 604800) else None
arguments = settings.get("ProgramArguments", [])
chart_export_dir = arguments[4] if len(arguments) >= 5 else "disabled"
require_usable_connections = settings.get("EnvironmentVariables", {}).get(
    "ARCH_NOTEBOOK_INSTALL_PREFLIGHT_REQUIRE_USABLE_CONNECTIONS", "true"
)
if require_usable_connections not in ("true", "false"):
    require_usable_connections = "true"
loaded = subprocess.run(
    ["launchctl", "print", f"{domain}/{label}"],
    stdout=subprocess.DEVNULL,
    stderr=subprocess.DEVNULL,
    check=False,
).returncode == 0
state = "loaded" if loaded else "configured but not loaded in the user LaunchAgent domain"
print(f"Arch Notebook SQL schedule for {case_path}: {cadence}; {state}.")
if schedule_time is not None:
    print(f"Schedule time: {schedule_time}")
if schedule_weekday is not None and 0 <= schedule_weekday <= 6:
    print(f"Schedule weekday: {['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'][schedule_weekday]}")
if schedule_day is not None:
    print(f"Schedule day: {schedule_day}")
print(f"Chart CSV export directory: {chart_export_dir}")
print(f"Schedule install preflight requires usable connections: {require_usable_connections}")
PY
  exit $?
fi

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <case-file> [app-bundle]" >&2
  exit 2
fi

schedule="${ARCH_NOTEBOOK_AUTOMATION_SCHEDULE:-hourly}"
schedule_weekday="${ARCH_NOTEBOOK_AUTOMATION_WEEKDAY:-1}"
schedule_day="${ARCH_NOTEBOOK_AUTOMATION_MONTH_DAY:-1}"
case "$schedule" in
  daily|weekly|monthly)
    schedule_time="${ARCH_NOTEBOOK_AUTOMATION_TIME:-00:00}"
    if [[ ! "$schedule_time" =~ ^([01][0-9]|2[0-3]):[0-5][0-9]$ ]]; then
      echo "ARCH_NOTEBOOK_AUTOMATION_TIME must use 24-hour HH:MM format." >&2
      exit 2
    fi
    if [[ "$schedule" == "monthly" && ! "$schedule_day" =~ ^([1-9]|1[0-9]|2[0-8])$ ]]; then
      echo "ARCH_NOTEBOOK_AUTOMATION_MONTH_DAY must be between 1 and 28." >&2
      exit 2
    fi
    if [[ "$schedule" == "weekly" ]]; then
      if [[ ! "$schedule_weekday" =~ ^[0-6]$ ]]; then
        echo "ARCH_NOTEBOOK_AUTOMATION_WEEKDAY must be 0 (Sunday) through 6 (Saturday)." >&2
        exit 2
      fi
      case "$schedule_weekday" in
        0) weekday_label="Sunday" ;;
        1) weekday_label="Monday" ;;
        2) weekday_label="Tuesday" ;;
        3) weekday_label="Wednesday" ;;
        4) weekday_label="Thursday" ;;
        5) weekday_label="Friday" ;;
        6) weekday_label="Saturday" ;;
      esac
    fi
    ;;
  15m|hourly) schedule_time="00:00" ;;
  *) echo "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE must be 15m, hourly, daily, weekly, or monthly." >&2; exit 2 ;;
esac

case_file="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
app_bundle="${2:-/Applications/Arch Notebook.app}"
if [[ ! -d "$app_bundle" ]]; then
  echo "App bundle not found: $app_bundle" >&2
  exit 1
fi
app_bundle="$(cd "$app_bundle" && pwd -P)"
binary="$app_bundle/Contents/MacOS/arch-notebook"
runner="$app_bundle/Contents/Resources/run-macos-automation.sh"
if [[ ! -r "$case_file" || ! -f "$case_file" ]]; then
  echo "Case file is not readable: $case_file" >&2
  exit 1
fi
if [[ ! -x "$binary" ]]; then
  echo "Arch Notebook executable not found: $binary" >&2
  exit 1
fi
if [[ ! -x "$runner" ]]; then
  echo "Arch Notebook automation runner not found: $runner" >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to write the LaunchAgent plist." >&2
  exit 1
fi
case_file="$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$case_file")"
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
  chart_export_dir="$(cd "$chart_export_dir" && pwd -P)"
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
    echo "One or more enabled SQL jobs failed preflight or returned no usable connections; fix the job, database, or query before installing the LaunchAgent." >&2
  else
    echo "One or more enabled SQL jobs failed query preflight; fix the job, database, or query before installing the LaunchAgent." >&2
  fi
  exit 1
fi

schedule_id="$(python3 - "$case_file" <<'PY'
import hashlib
import sys
print(hashlib.sha256(sys.argv[1].encode()).hexdigest()[:12])
PY
)"
label="com.archnotebook.automation.$schedule_id"
if legacy_schedule_matches_case "$HOME/Library/LaunchAgents" "$case_file"; then
  label="com.archnotebook.automation"
fi
agent_dir="$HOME/Library/LaunchAgents"
log_dir="$HOME/Library/Logs/ArchNotebook/$schedule_id"
plist_path="$agent_dir/$label.plist"
mkdir -p "$agent_dir" "$log_dir"

candidate_plist="$(python3 - "$plist_path" "$runner" "$binary" "$case_file" "$log_dir" "$label" "$schedule" "$chart_export_dir" "$schedule_time" "$schedule_weekday" "$schedule_day" "$require_usable_connections" <<'PY'
import os
import plistlib
import sys
import tempfile

destination, runner, binary, case_file, log_dir, label, schedule, chart_export_dir, schedule_time, schedule_weekday, schedule_day, require_usable_connections = sys.argv[1:]
payload = {
    "Label": label,
    "ProgramArguments": [runner, binary, case_file, log_dir] + ([chart_export_dir] if chart_export_dir else []),
    "RunAtLoad": True,
    "StandardOutPath": "/dev/null",
    "StandardErrorPath": "/dev/null",
    "EnvironmentVariables": {
        "ARCH_NOTEBOOK_INSTALL_PREFLIGHT_REQUIRE_USABLE_CONNECTIONS": require_usable_connections,
    },
}
if schedule == "monthly":
    payload["StartCalendarInterval"] = {
        "Day": int(schedule_day),
        "Hour": int(schedule_time[:2]),
        "Minute": int(schedule_time[3:]),
    }
elif schedule == "weekly":
    payload["StartCalendarInterval"] = {
        "Weekday": int(schedule_weekday),
        "Hour": int(schedule_time[:2]),
        "Minute": int(schedule_time[3:]),
    }
elif schedule == "daily":
    payload["StartCalendarInterval"] = {
        "Hour": int(schedule_time[:2]),
        "Minute": int(schedule_time[3:]),
    }
else:
    payload["StartInterval"] = {"15m": 900, "hourly": 3600, "daily": 86400, "weekly": 604800}[schedule]
directory = os.path.dirname(destination)
fd, temporary = tempfile.mkstemp(prefix=f".{label}.", suffix=".plist", dir=directory)
try:
    with os.fdopen(fd, "wb") as output:
        plistlib.dump(payload, output, fmt=plistlib.FMT_XML, sort_keys=True)
    os.chmod(temporary, 0o644)
    print(temporary)
except BaseException:
    try:
        os.unlink(temporary)
    except FileNotFoundError:
        pass
    raise
PY
 )"
backup_plist=""
cleanup_candidate() {
  [[ -z "$candidate_plist" || ! -e "$candidate_plist" ]] || rm -f "$candidate_plist"
  [[ -z "$backup_plist" || ! -e "$backup_plist" ]] || rm -f "$backup_plist"
}
trap cleanup_candidate EXIT
if [[ -e "$plist_path" ]]; then
  backup_plist="$(mktemp "$agent_dir/.${label}.backup.XXXXXX")"
  cp -p "$plist_path" "$backup_plist"
fi

plutil -lint "$candidate_plist"
gui_domain="gui/$(id -u)"
launchctl bootout "$gui_domain" "$plist_path" >/dev/null 2>&1 || true
if ! mv -f "$candidate_plist" "$plist_path"; then
  echo "Could not install the new LaunchAgent plist; restoring the previous schedule." >&2
  if [[ -n "$backup_plist" ]]; then
    if ! launchctl bootstrap "$gui_domain" "$plist_path"; then
      echo "The previous LaunchAgent plist remains in place but could not be loaded." >&2
    fi
  fi
  exit 1
fi
candidate_plist=""
if ! launchctl bootstrap "$gui_domain" "$plist_path"; then
  echo "Could not load the new LaunchAgent; restoring the previous schedule." >&2
  rm -f "$plist_path"
  if [[ -n "$backup_plist" ]]; then
    mv -f "$backup_plist" "$plist_path"
    backup_plist=""
    if ! launchctl bootstrap "$gui_domain" "$plist_path"; then
      echo "The previous LaunchAgent plist was restored but could not be loaded." >&2
    fi
  fi
  exit 1
fi
if [[ -n "$backup_plist" ]]; then
  rm -f "$backup_plist"
  backup_plist=""
fi
echo "Installed and loaded $plist_path"
echo "Schedule install preflight requires usable connections: $require_usable_connections"
if [[ "$schedule" == "weekly" ]]; then
  echo "The first run starts at login/load; later runs start weekly on $weekday_label at $schedule_time."
else
  echo "The first run starts at login/load; later runs start $schedule."
fi
echo "Logs: $log_dir/arch-notebook-automation{,-error}.log"
