#!/usr/bin/env bash
set -euo pipefail

legacy_schedule_matches_case() {
  python3 - "$1/arch-notebook-automation.service" "$2" <<'PY'
import sys

service_path, case_path = sys.argv[1:]
try:
    with open(service_path, encoding="utf-8") as source:
        lines = source.readlines()
except OSError:
    raise SystemExit(1)
escaped = case_path.replace("\\", "\\\\").replace('"', '\\"')
escaped = escaped.replace("%", "%%").replace("$", "$$")
target = f'"{escaped}"'
if any(line.startswith("ExecStart=") and line.rstrip().endswith(target) for line in lines):
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
    exit 0
  fi
  cadence="$(awk -F= '/^OnCalendar=/ { print $2; exit }' "$timer_file")"
  if [[ -z "$cadence" ]]; then cadence="unknown cadence"; fi
  if command -v systemctl >/dev/null 2>&1; then
    enabled=0
    active=0
    if systemctl --user is-enabled --quiet "$schedule_name.timer"; then enabled=1; fi
    if systemctl --user is-active --quiet "$schedule_name.timer"; then active=1; fi
    if (( enabled && active )); then
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
  if [[ $# -eq 2 ]]; then
    printf 'Arch Notebook SQL schedule for %s: %s; %s.\n' "$case_path" "$cadence" "$state"
  else
    echo "Legacy Arch Notebook SQL schedule: $cadence; $state."
  fi
  exit 0
fi

if [[ $# -gt 2 ]]; then
  echo "Usage: $0 [case-file] [arch-notebook-binary]" >&2
  exit 2
fi

schedule="${ARCH_NOTEBOOK_AUTOMATION_SCHEDULE:-hourly}"
case "$schedule" in
  hourly|daily|weekly) ;;
  *) echo "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE must be hourly, daily, or weekly." >&2; exit 2 ;;
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

if ! command -v systemctl >/dev/null 2>&1; then
  echo "systemctl is required to enable the user timer." >&2
  exit 1
fi

# One locked preflight checks that enabled jobs exist and their queries work.
if ! "$binary" --check-enabled-job-queries "$case_file" --require-usable; then
  echo "One or more enabled SQL jobs failed preflight or returned no usable connections; fix the job, database, or query before installing the timer." >&2
  exit 1
fi

config_home="${XDG_CONFIG_HOME:-$home_dir/.config}"
case "$config_home" in
  /*) ;;
  *) echo "XDG_CONFIG_HOME must be an absolute path." >&2; exit 1 ;;
esac
unit_dir="$config_home/systemd/user"
mkdir -p "$unit_dir"

python3 - "$unit_dir" "$binary" "$case_file" "$schedule" "$schedule_name" <<'PY'
import os
import sys
import tempfile

unit_dir, binary, case_file, schedule, schedule_name = sys.argv[1:]

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
TimeoutStartSec=1h
ExecStart={command}
""".format(command=" ".join(map(quote_exec_arg, [
    # Scheduler output goes to the journal; keep it to per-job summaries.
    binary, "--run-enabled-jobs", case_file
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
echo "Enabled $schedule Arch Notebook automation for: $case_file"
echo "Check status with: systemctl --user status $schedule_name.timer"
echo "View run output with: journalctl --user -u $schedule_name.service"
