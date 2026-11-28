#!/usr/bin/env bash
set -euo pipefail
umask 077

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
bin_dir="${XDG_BIN_HOME:-$HOME/.local/bin}"
applications="$data_home/applications"
icon_dir="$data_home/icons/hicolor/scalable/apps"
desktop_dir="${XDG_DESKTOP_DIR:-$HOME/Desktop}"

install -d "$bin_dir" "$applications" "$icon_dir"
install -m 0755 "$repo_root/tools/run-clinton-qwen-gpu-demo.sh" \
  "$bin_dir/arch-notebook-clinton-qwen-gpu-demo"
install -m 0755 "$repo_root/tools/validate-clinton-qwen-gpu-output.py" \
  "$bin_dir/arch-notebook-validate-clinton-qwen-gpu-output"
install -m 0644 "$repo_root/assets/icons/arch-notebook-qwen-gpu-demo.svg" \
  "$icon_dir/arch-notebook-qwen-gpu-demo.svg"
install -d "$desktop_dir"
entry="$(mktemp)"
trap 'rm -f "$entry"' EXIT
sed "s|^Exec=.*|Exec=$bin_dir/arch-notebook-clinton-qwen-gpu-demo|" \
  "$repo_root/packaging/arch/arch-notebook-qwen-gpu-demo.desktop" >"$entry"
install -m 0644 "$entry" "$applications/arch-notebook-clinton-qwen-gpu-demo.desktop"
install -m 0755 "$entry" "$desktop_dir/Arch Notebook — Qwen GPU Demo.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$applications" >/dev/null 2>&1 || true
fi
printf 'Installed Clinton Qwen GPU demo launcher:\n  %s\n  %s\n  %s\n' \
  "$bin_dir/arch-notebook-clinton-qwen-gpu-demo" \
  "$applications/arch-notebook-clinton-qwen-gpu-demo.desktop" \
  "$icon_dir/arch-notebook-qwen-gpu-demo.svg"
printf 'Desktop shortcut:\n  %s\n' "$desktop_dir/Arch Notebook — Qwen GPU Demo.desktop"
