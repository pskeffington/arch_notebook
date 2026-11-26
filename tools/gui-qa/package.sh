#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
[[ "$(uname -s)" == Linux && "$(uname -m)" == x86_64 ]] || { echo "GUI QA package currently targets Linux x86_64." >&2; exit 2; }
target_dir="${CARGO_TARGET_DIR:-/tmp/arch-notebook-target}"
artifact_dir="${GUI_QA_ARTIFACT_DIR:-/tmp}"
fixture="$repo_root/fixtures/chart/correctness.case.json"
source_commit="$(git -C "$repo_root" rev-parse HEAD)"
short_commit="${source_commit:0:12}"
package_name="arch-notebook-gui-qa-$short_commit"
archive="$artifact_dir/$package_name.tar.gz"
if [[ -e "$archive" || -L "$archive" || -e "$archive.sha256" || -L "$archive.sha256" ]]; then
  echo "Refusing to overwrite existing GUI QA package or digest: $archive" >&2
  exit 1
fi
staging_root="$(mktemp -d "$artifact_dir/.${package_name}.XXXXXX")"
package_root="$staging_root/$package_name"
archive_linked=0
checksum_linked=0
publish_complete=0

cleanup() {
  if [[ "$publish_complete" != 1 ]]; then
    [[ "$checksum_linked" == 0 ]] || rm -f -- "$archive.sha256"
    [[ "$archive_linked" == 0 ]] || rm -f -- "$archive"
  fi
  rm -rf -- "$staging_root"
}
trap cleanup EXIT

mkdir -p "$package_root/bin" "$package_root/fixtures" "$package_root/tools/gui-qa"

printf 'Formatting check…\n'
(cd "$repo_root" && cargo fmt --all -- --check)
printf 'Offline GUI QA helper checks…\n'
(cd "$repo_root" && for script in tools/gui-qa/package.sh tools/gui-qa/run.sh tools/gui-qa/ollama-cloud-mode.sh; do bash -n "$script"; done)
(cd "$repo_root" && python3 -B tools/gui-qa/test_gui_qa.py)
(cd "$repo_root" && python3 -B tools/gui-qa/qwen_review.py --self-test)
printf 'Offline test suite…\n'
(cd "$repo_root" && CARGO_TARGET_DIR="$target_dir" cargo test --locked --offline)
printf 'Release build…\n'
(cd "$repo_root" && CARGO_TARGET_DIR="$target_dir" cargo build --release --locked --offline)

binary="$target_dir/release/arch-notebook"
test -x "$binary"
version_json="$("$binary" --version --json)"
python3 -c 'import json,sys; d=json.loads(sys.argv[1]); assert d["name"] == "arch-notebook"; assert d["version"]' "$version_json"

install -m 0755 "$binary" "$package_root/bin/arch-notebook"
install -m 0644 "$fixture" "$package_root/fixtures/correctness.case.json"
install -m 0644 "$repo_root/fixtures/chart/manifest.json" "$package_root/fixtures/fixture-manifest.json"
install -m 0755 "$repo_root/tools/gui-qa/run.sh" "$package_root/run.sh"
install -m 0644 "$repo_root/tools/gui-qa/run_gui_qa.py" "$package_root/tools/gui-qa/run_gui_qa.py"
install -m 0644 "$repo_root/tools/gui-qa/qwen_review.py" "$package_root/tools/gui-qa/qwen_review.py"
install -m 0644 "$repo_root/tools/gui-qa/manual-review.template.json" "$package_root/manual-review.template.json"
install -m 0644 "$repo_root/tools/gui-qa/review-checklist.md" "$package_root/review-checklist.md"
install -m 0644 "$repo_root/tools/gui-qa/README.md" "$package_root/README.md"
install -m 0755 "$repo_root/tools/gui-qa/ollama-cloud-mode.sh" "$package_root/ollama-cloud-mode.sh"
install -m 0644 "$repo_root/tools/gui-qa/test_gui_qa.py" "$package_root/tools/gui-qa/test_gui_qa.py"

binary_sha="$(sha256sum "$package_root/bin/arch-notebook" | cut -d' ' -f1)"
fixture_sha="$(sha256sum "$package_root/fixtures/correctness.case.json" | cut -d' ' -f1)"
declared_fixture_sha="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1], encoding="utf-8"))["files"]["correctness.case.json"]["sha256"])' "$package_root/fixtures/fixture-manifest.json")"
[[ "$fixture_sha" == "$declared_fixture_sha" ]] || { echo "Fictional chart fixture does not match its committed manifest." >&2; exit 1; }
source_sha="$(sha256sum "$repo_root/src/main.rs" | cut -d' ' -f1)"
lock_sha="$(sha256sum "$repo_root/Cargo.lock" | cut -d' ' -f1)"
source_status="$(git -C "$repo_root" status --porcelain=v1 -- src assets Cargo.toml Cargo.lock build.rs .cargo)"
source_dirty="false"
[[ -z "$source_status" ]] || source_dirty="true"
python3 - "$package_root/manifest.json" "$package_root" "$source_commit" "$version_json" "$binary_sha" "$fixture_sha" "$source_sha" "$lock_sha" "$source_dirty" <<'PY'
import hashlib
import json
import pathlib
import sys

output, package_root, source_commit, version_json, binary_sha, fixture_sha, source_sha, lock_sha, source_dirty = sys.argv[1:]
package_root = pathlib.Path(package_root)
version = json.loads(version_json)
qa_tool_paths = (
    "run.sh",
    "tools/gui-qa/run_gui_qa.py",
    "tools/gui-qa/qwen_review.py",
    "manual-review.template.json",
    "review-checklist.md",
    "README.md",
    "ollama-cloud-mode.sh",
    "tools/gui-qa/test_gui_qa.py",
)
qa_tools = {}
for relative in qa_tool_paths:
    content = (package_root / relative).read_bytes()
    qa_tools[relative] = hashlib.sha256(content).hexdigest()
manifest = {
    "schema": "arch-notebook-gui-qa-package/v1",
    "target": "linux-x86_64",
    "source_commit": source_commit,
    "source_files": {
        "src/main.rs_sha256": source_sha,
        "Cargo.lock_sha256": lock_sha,
        "app_or_dependency_worktree_dirty": source_dirty == "true",
    },
    "application": version,
    "qa_tools_sha256": qa_tools,
    "binary": {"path": "bin/arch-notebook", "sha256": binary_sha},
    "fixture": {
        "path": "fixtures/correctness.case.json",
        "sha256": fixture_sha,
        "source": "Arch Notebook fictional correctness fixture",
    },
    "run_isolation": {
        "bubblewrap_required": True,
        "network_isolated_by_default": True,
        "xdg_data_config_cache": "per-run temporary directories",
    },
    "qwen_review": {
        "default_model": "qwen3.5:9b",
        "cloud_requires_explicit_allow_flag": True,
        "sends_only_sanitized_qa_report_unless_screenshot_opted_in": True,
    },
}
with open(output, "w", encoding="utf-8") as stream:
    json.dump(manifest, stream, indent=2, sort_keys=True)
    stream.write("\n")
PY

python3 "$package_root/tools/gui-qa/run_gui_qa.py" --preflight --output "$staging_root/preflight-report.json" --bundle "$package_root"
install -m 0644 "$staging_root/preflight-report.json" "$package_root/package-preflight.json"

(cd "$staging_root" && tar --sort=name --mtime='@0' --owner=0 --group=0 --numeric-owner -cf - "$package_name" | gzip -n > "$package_name.tar.gz")
(cd "$staging_root" && sha256sum "$package_name.tar.gz" > "$package_name.tar.gz.sha256")
ln -- "$staging_root/$package_name.tar.gz" "$archive"
archive_linked=1
ln -- "$staging_root/$package_name.tar.gz.sha256" "$archive.sha256"
checksum_linked=1
publish_complete=1
printf 'GUI QA package: %s\n' "$archive"
printf 'SHA-256: %s\n' "$(cut -d' ' -f1 < "$archive.sha256")"
