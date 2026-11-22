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

case_file="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
app_bundle="${2:-/Applications/Arch Notebook.app}"
binary="$app_bundle/Contents/MacOS/arch-notebook"
if [[ ! -r "$case_file" || ! -f "$case_file" ]]; then
  echo "Case file is not readable: $case_file" >&2
  exit 1
fi
if [[ ! -x "$binary" ]]; then
  echo "Arch Notebook executable not found: $binary" >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to write the LaunchAgent plist." >&2
  exit 1
fi

label="com.archnotebook.automation"
agent_dir="$HOME/Library/LaunchAgents"
log_dir="$HOME/Library/Logs"
plist_path="$agent_dir/$label.plist"
mkdir -p "$agent_dir" "$log_dir"

python3 - "$plist_path" "$binary" "$case_file" "$log_dir" "$label" <<'PY'
import os
import plistlib
import sys
import tempfile

destination, binary, case_file, log_dir, label = sys.argv[1:]
payload = {
    "Label": label,
    "ProgramArguments": [binary, "--run-enabled-jobs", case_file, "--json"],
    "RunAtLoad": True,
    "StartInterval": 3600,
    "StandardOutPath": os.path.join(log_dir, "arch-notebook-automation.log"),
    "StandardErrorPath": os.path.join(log_dir, "arch-notebook-automation-error.log"),
}
directory = os.path.dirname(destination)
fd, temporary = tempfile.mkstemp(prefix=f".{label}.", suffix=".plist", dir=directory)
try:
    with os.fdopen(fd, "wb") as output:
        plistlib.dump(payload, output, fmt=plistlib.FMT_XML, sort_keys=True)
    os.chmod(temporary, 0o644)
    os.replace(temporary, destination)
except BaseException:
    try:
        os.unlink(temporary)
    except FileNotFoundError:
        pass
    raise
PY

plutil -lint "$plist_path"
gui_domain="gui/$(id -u)"
launchctl bootout "$gui_domain" "$plist_path" >/dev/null 2>&1 || true
launchctl bootstrap "$gui_domain" "$plist_path"
echo "Installed and loaded $plist_path"
echo "The first run starts at login/load; later runs start hourly."
echo "Logs: $log_dir/arch-notebook-automation{,-error}.log"
