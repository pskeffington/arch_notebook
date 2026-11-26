#!/usr/bin/env bash
set -euo pipefail

dropin_dir="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/ollama.service.d"
dropin="$dropin_dir/99-arch-notebook-gui-qa-cloud.conf"
usage() { echo "usage: $0 enable|disable|status [--yes]" >&2; exit 2; }
[[ $# -ge 1 && $# -le 2 ]] || usage
action="$1"
yes="${2:-}"
[[ -z "$yes" || "$yes" == "--yes" ]] || usage

assert_no_symlink_components() {
  local path="$1" current="/" component
  local -a components
  [[ "$path" == /* ]] || return 1
  IFS=/ read -r -a components <<< "${path#/}"
  for component in "${components[@]}"; do
    [[ -z "$component" || "$component" == "." ]] && continue
    [[ "$component" != ".." ]] || return 1
    current="${current%/}/$component"
    [[ ! -L "$current" ]] || return 1
  done
}

confirm() {
  if [[ "$yes" == "--yes" ]]; then return 0; fi
  [[ -t 0 ]] || { echo "Refusing non-interactive change; rerun with --yes after reviewing the cloud-data notice." >&2; return 1; }
  read -r -p "Allow Ollama cloud models for this user service? Requests to :cloud models go to Ollama-hosted compute and may use account credits. [y/N] " answer
  [[ "$answer" == "y" || "$answer" == "Y" ]]
}

write_dropin() {
  [[ ! -e "$dropin" && ! -L "$dropin" ]] || return 1
  umask 077
  (set -o noclobber; cat > "$dropin" <<'EOF'
[Service]
UnsetEnvironment=OLLAMA_NO_CLOUD
EOF
  )
}

case "$action" in
  status)
    assert_no_symlink_components "$dropin_dir" || { echo "Refusing a relative or symlinked Ollama drop-in path: $dropin_dir" >&2; exit 1; }
    if [[ -L "$dropin" ]]; then echo "Refusing symlinked drop-in path: $dropin" >&2; exit 1
    elif [[ -f "$dropin" ]]; then echo "QA cloud opt-in drop-in: present"
    else echo "QA cloud opt-in drop-in: absent"; fi
    if systemctl --user is-active --quiet ollama.service; then echo "ollama.service: active"; else echo "ollama.service: inactive or user bus unavailable"; fi
    ;;
  enable)
    confirm || { echo "No change made."; exit 1; }
    assert_no_symlink_components "$dropin_dir" || { echo "Refusing a relative or symlinked Ollama drop-in path: $dropin_dir" >&2; exit 1; }
    mkdir -p "$dropin_dir"
    assert_no_symlink_components "$dropin_dir" && [[ ! -e "$dropin" && ! -L "$dropin" ]] || { echo "Refusing to overwrite or follow a symlinked Ollama drop-in path: $dropin" >&2; exit 1; }
    write_dropin || { echo "Could not create the cloud-mode drop-in safely." >&2; exit 1; }
    if systemctl --user daemon-reload && systemctl --user restart ollama.service; then
      echo "Removed the OLLAMA_NO_CLOUD environment guard for the Ollama user service. An Ollama server.json policy may still disable cloud; sign-in is separate and this script does not access credentials."
    else
      rm -f -- "$dropin"
      if systemctl --user daemon-reload && systemctl --user restart ollama.service; then
        echo "Cloud-mode activation failed; the drop-in was removed and the prior service configuration was restarted. No sign-in or model request was performed." >&2
      else
        echo "Cloud-mode activation failed and rollback could not be verified. The drop-in is removed, but the active service state is unknown; inspect ollama.service before using cloud models." >&2
      fi
      exit 1
    fi
    ;;
  disable)
    confirm || { echo "No change made."; exit 1; }
    assert_no_symlink_components "$dropin_dir" && [[ ! -L "$dropin" ]] || { echo "Refusing symlinked Ollama drop-in path." >&2; exit 1; }
    [[ -f "$dropin" ]] || { echo "QA cloud opt-in drop-in is not present."; exit 0; }
    cmp -s "$dropin" <(printf '[Service]\nUnsetEnvironment=OLLAMA_NO_CLOUD\n') || {
      echo "Drop-in contents do not match this QA helper; refusing to remove it." >&2
      exit 1
    }
    rm -- "$dropin"
    if systemctl --user daemon-reload && systemctl --user restart ollama.service; then
      echo "Removed only the GUI QA cloud opt-in drop-in and restarted Ollama with the unit's original cloud policy."
    else
      if write_dropin && systemctl --user daemon-reload && systemctl --user restart ollama.service; then
        echo "Cloud disable could not be applied; restored the QA opt-in drop-in and restarted Ollama. Cloud remains enabled." >&2
      else
        echo "Cloud disable failed and the service state could not be restored or verified. Inspect the Ollama unit and QA drop-in before using cloud models." >&2
      fi
      exit 1
    fi
    ;;
  *) usage ;;
esac
