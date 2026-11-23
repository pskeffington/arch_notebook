#!/bin/bash
set -u

if [[ $# -ne 3 ]]; then
  echo "Usage: $0 <arch-notebook-binary> <case-file> <log-directory>" >&2
  exit 2
fi

binary="$1"
case_file="$2"
log_dir="$3"
stdout_log="$log_dir/arch-notebook-automation.log"
stderr_log="$log_dir/arch-notebook-automation-error.log"

rotate_log() {
  local path="$1"
  local max_bytes=5242880
  local size
  local index

  [[ -f "$path" ]] || return 0
  size="$(wc -c < "$path")"
  size="${size//[[:space:]]/}"
  (( size >= max_bytes )) || return 0

  rm -f "$path.5"
  for ((index = 4; index >= 1; index--)); do
    [[ ! -f "$path.$index" ]] || mv -f "$path.$index" "$path.$((index + 1))"
  done
  mv -f "$path" "$path.1"
}

mkdir -p "$log_dir" || exit 1
rotate_log "$stdout_log" || exit 1
rotate_log "$stderr_log" || exit 1
exec "$binary" --run-enabled-jobs "$case_file" >>"$stdout_log" 2>>"$stderr_log"
