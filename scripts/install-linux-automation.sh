#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "This installer is for Linux." >&2
  exit 2
fi

if [[ "${1:-}" == "--remove" ]]; then
  if [[ $# -ne 1 ]]; then
    echo "Usage: $0 --remove" >&2
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
  unit_dir="$config_home/systemd/user"
  if command -v systemctl >/dev/null 2>&1; then
    systemctl --user disable --now arch-notebook-automation.timer >/dev/null 2>&1 || true
  fi
  rm -f "$unit_dir/arch-notebook-automation.service" "$unit_dir/arch-notebook-automation.timer"
  if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload >/dev/null 2>&1 || true
  fi
  echo "Removed Arch Notebook SQL automation schedule for this user."
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
case_file="$(cd "$(dirname "$case_file")" && pwd)/$(basename "$case_file")"

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

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to write the systemd user units." >&2
  exit 1
fi
if ! command -v systemctl >/dev/null 2>&1; then
  echo "systemctl is required to enable the user timer." >&2
  exit 1
fi

# One locked preflight checks that enabled jobs exist and their queries work.
if ! "$binary" --check-enabled-job-queries "$case_file"; then
  echo "One or more enabled SQL jobs failed preflight; fix the job, database, or query before installing the timer." >&2
  exit 1
fi

config_home="${XDG_CONFIG_HOME:-$home_dir/.config}"
case "$config_home" in
  /*) ;;
  *) echo "XDG_CONFIG_HOME must be an absolute path." >&2; exit 1 ;;
esac
unit_dir="$config_home/systemd/user"
mkdir -p "$unit_dir"

python3 - "$unit_dir" "$binary" "$case_file" "$schedule" <<'PY'
import os
import sys
import tempfile

unit_dir, binary, case_file, schedule = sys.argv[1:]

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
Unit=arch-notebook-automation.service

[Install]
WantedBy=timers.target
""".format(schedule=schedule)

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
    os.path.join(unit_dir, "arch-notebook-automation.service"), service
)
replace_atomically(
    os.path.join(unit_dir, "arch-notebook-automation.timer"), timer
)
PY

systemctl --user daemon-reload
systemctl --user enable --now arch-notebook-automation.timer
systemctl --user restart arch-notebook-automation.timer
echo "Enabled $schedule Arch Notebook automation for: $case_file"
echo "Check status with: systemctl --user status arch-notebook-automation.timer"
echo "View run output with: journalctl --user -u arch-notebook-automation.service"
