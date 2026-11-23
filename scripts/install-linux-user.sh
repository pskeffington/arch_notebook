#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "This installer is for Linux." >&2
  exit 2
fi

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <extracted-archive-directory>" >&2
  exit 2
fi

case "$(uname -m)" in
  x86_64) archive_arch="x86_64" ;;
  aarch64|arm64) archive_arch="aarch64" ;;
  *) echo "Unsupported Linux architecture: $(uname -m)" >&2; exit 2 ;;
esac

archive_dir="${1%/}"
expected_dir="arch-notebook-linux-$archive_arch"
if [[ "$(basename "$archive_dir")" != "$expected_dir" ]]; then
  echo "Expected the $expected_dir archive directory for this system." >&2
  exit 1
fi

binary_source="$archive_dir/arch-notebook"
desktop_source="$archive_dir/arch-notebook.desktop"
automation_installer_source="$archive_dir/install-linux-automation.sh"
if [[ ! -f "$binary_source" || -L "$binary_source" || ! -x "$binary_source" ]]; then
  echo "Missing executable archive binary: $binary_source" >&2
  exit 1
fi
if [[ ! -f "$desktop_source" || -L "$desktop_source" ]]; then
  echo "Missing desktop entry: $desktop_source" >&2
  exit 1
fi
if [[ ! -f "$automation_installer_source" || -L "$automation_installer_source" || ! -x "$automation_installer_source" ]]; then
  echo "Missing executable archive automation installer: $automation_installer_source" >&2
  exit 1
fi

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

binary_dir="$home_dir/.local/bin"
desktop_dir="$data_home/applications"
binary_path="$binary_dir/arch-notebook"
automation_installer_path="$binary_dir/arch-notebook-install-automation"
desktop_path="$desktop_dir/arch-notebook.desktop"
mkdir -p "$binary_dir" "$desktop_dir"
binary_tmp="$(mktemp "$binary_dir/.arch-notebook.XXXXXX")"
trap 'rm -f "$binary_tmp"' EXIT
install -m 755 "$binary_source" "$binary_tmp"
mv -f "$binary_tmp" "$binary_path"
trap - EXIT

automation_installer_tmp="$(mktemp "$binary_dir/.arch-notebook-install-automation.XXXXXX")"
trap 'rm -f "$automation_installer_tmp"' EXIT
install -m 755 "$automation_installer_source" "$automation_installer_tmp"
mv -f "$automation_installer_tmp" "$automation_installer_path"
trap - EXIT

desktop_exec=${binary_path//\\/\\\\}
desktop_exec=${desktop_exec//\"/\\\"}
desktop_exec=${desktop_exec//%/%%}
desktop_tmp="$(mktemp "$desktop_dir/.arch-notebook.desktop.XXXXXX")"
trap 'rm -f "$desktop_tmp"' EXIT
ARCH_NOTEBOOK_DESKTOP_EXEC="$desktop_exec" awk '
  BEGIN { exec_value = ENVIRON["ARCH_NOTEBOOK_DESKTOP_EXEC"] }
  /^Exec=/ { print "Exec=\"" exec_value "\""; next }
  { print }
' "$desktop_source" > "$desktop_tmp"
chmod 644 "$desktop_tmp"
mv -f "$desktop_tmp" "$desktop_path"
trap - EXIT

echo "Installed executable: $binary_path"
echo "Installed automation installer: $automation_installer_path"
echo "Installed application menu entry: $desktop_path"
echo "Add $binary_dir to PATH to run arch-notebook from a terminal."
