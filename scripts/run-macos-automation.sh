#!/bin/bash
set -u

if [[ $# -lt 3 || $# -gt 4 ]]; then
  echo "Usage: $0 <arch-notebook-binary> <case-file> <log-directory> [chart-export-directory]" >&2
  exit 2
fi

binary="$1"
case_file="$2"
log_dir="$3"
chart_export_dir="${4:-}"
stdout_log="$log_dir/arch-notebook-automation.log"
stderr_log="$log_dir/arch-notebook-automation-error.log"
umask 077

rotate_log() {
  local path="$1"
  local max_bytes=5242880
  local size
  local index

  [[ ! -L "$path" ]] || return 1
  [[ -f "$path" ]] || return 0
  size="$(wc -c < "$path")"
  size="${size//[[:space:]]/}"
  (( size >= max_bytes )) || return 0

  rm -f "$path.5" || return 1
  for ((index = 4; index >= 1; index--)); do
    [[ ! -L "$path.$index" ]] || return 1
    if [[ -f "$path.$index" ]]; then
      mv -f "$path.$index" "$path.$((index + 1))" || return 1
      chmod 600 "$path.$((index + 1))" || return 1
    fi
  done
  mv -f "$path" "$path.1" || return 1
  chmod 600 "$path.1" || return 1
}

mkdir -p "$log_dir" || exit 1
rotate_log "$stdout_log" || exit 1
rotate_log "$stderr_log" || exit 1
touch "$stdout_log" "$stderr_log" || exit 1
chmod 600 "$stdout_log" "$stderr_log" || exit 1
export ARCH_NOTEBOOK_AUTOMATION_TRIGGER=scheduled
if [[ -n "$chart_export_dir" ]]; then
  exec "$binary" --run-enabled-jobs "$case_file" --progress --export-chart-csv-dir "$chart_export_dir" >>"$stdout_log" 2>>"$stderr_log"
fi
exec "$binary" --run-enabled-jobs "$case_file" --progress >>"$stdout_log" 2>>"$stderr_log"
