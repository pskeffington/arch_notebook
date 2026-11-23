#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This installer is for macOS." >&2
  exit 2
fi

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <case-file> [app-bundle]" >&2
  exit 2
fi

schedule="${ARCH_NOTEBOOK_AUTOMATION_SCHEDULE:-hourly}"
case "$schedule" in
  hourly|daily|weekly) ;;
  *) echo "ARCH_NOTEBOOK_AUTOMATION_SCHEDULE must be hourly, daily, or weekly." >&2; exit 2 ;;
esac

case_file="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
app_bundle="${2:-/Applications/Arch Notebook.app}"
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
if ! job_list_json=$("$binary" --list-jobs "$case_file" --enabled-only --json); then
  echo "Arch Notebook could not read the case file: $case_file" >&2
  exit 1
fi
if ! python3 -c 'import json, sys; raise SystemExit(0 if json.load(sys.stdin).get("enabled", 0) > 0 else 1)' <<<"$job_list_json"; then
  echo "Enable at least one automation job in the case before installing its LaunchAgent." >&2
  exit 1
fi
if ! "$binary" --check-enabled-job-queries "$case_file"; then
  echo "One or more enabled SQL jobs failed preflight; fix the job, database, or query before installing the LaunchAgent." >&2
  exit 1
fi

label="com.archnotebook.automation"
agent_dir="$HOME/Library/LaunchAgents"
log_dir="$HOME/Library/Logs"
plist_path="$agent_dir/$label.plist"
mkdir -p "$agent_dir" "$log_dir"

candidate_plist="$(python3 - "$plist_path" "$runner" "$binary" "$case_file" "$log_dir" "$label" "$schedule" <<'PY'
import os
import plistlib
import sys
import tempfile

destination, runner, binary, case_file, log_dir, label, schedule = sys.argv[1:]
interval_seconds = {"hourly": 3600, "daily": 86400, "weekly": 604800}[schedule]
payload = {
    "Label": label,
    "ProgramArguments": [runner, binary, case_file, log_dir],
    "RunAtLoad": True,
    "StartInterval": interval_seconds,
    "StandardOutPath": "/dev/null",
    "StandardErrorPath": "/dev/null",
}
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
echo "The first run starts at login/load; later runs start $schedule."
echo "Logs: $log_dir/arch-notebook-automation{,-error}.log"
