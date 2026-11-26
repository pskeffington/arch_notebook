#!/usr/bin/env bash
set -euo pipefail
package_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "$package_root/tools/gui-qa/run_gui_qa.py" --bundle "$package_root" --launch "$@"
