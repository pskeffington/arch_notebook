# Quick Open saved import design navigation

- **Task / owner:** Continue Arch Notebook navigation and ingestion usability; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `bee438928a74f14196cca34ed93241926ded201e`.
- **Candidate source commit:** `d6149baf960692f9551b31b89f7976181a8e9e25`.
- **Scope:** Add case-local CSV/TSV import designs with a remembered source file to Quick Open by design name. Dispatch uses the existing background inspector and opens mapping review without importing chart data.
- **Release binary SHA-256:** `5c92cf1023cd194c5f6ba03ec99d7fae75523624dfee9c79e384f285a30cc6fd`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Focused Quick Open interaction | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline quick_open_reinspects_saved_import_design_without_importing_chart_data`: 1 passed. It verifies command visibility, selection, background CSV inspection, mapping review, and unchanged entity/link counts. |
| Full Rust suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`: 104 passed, 0 failed, 3 ignored. |
| Formatting, workspace check, whitespace | PASS | `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, and `git diff --check` exited 0. |
| Optimized release build | PASS | `TMPDIR=/home/sysadmin/.cache/arch-notebook-build-tmp CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline`, exit 0. |
| Isolated installer acceptance | PASS | [Installer aggregate JSON](artifacts/QUICK-OPEN-IMPORT-DESIGN-01-install.json), SHA-256 `bc18e1b82f88319119fcf40149104625d8f8ee6e54533922db5079810435a586`. Binary, launchers, shortcut mode, SVG/PNG icons, SQL helper, and MIME definition passed. |
| Current-user install | PASS | `bash scripts/install-linux-user.sh /tmp/arch-notebook-linux-x86_64`, exit 0. `/home/sysadmin/.local/bin/arch-notebook` matches release SHA-256. |
| Clinton primary corpus | PASS, bounded | `python3 scripts/test-clinton-corpus-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/CLINTON-CORPUS-QA-quick-open-import.json`, exit 0. One query-only preflight passed; 6,173 rows replayed, zero skipped, zero deltas, read-only inputs unchanged. [Aggregate-only JSON](artifacts/QUICK-OPEN-IMPORT-DESIGN-01-clinton-aggregate.json), SHA-256 `959f7dd849c66ef4d58ec1253546cc28641915020e3c9f0addaed9e3d6af2b5d`. |
| Native pointer feel and IBM navigation comparison | NOT RUN | Headless action/inspection coverage does not establish native visual or pointer acceptance; no IBM application was available. |

The Clinton artifact contains no names, relationship labels, IDs, or evidence references. This run does not establish PDF extraction completeness, Qwen analysis, or human content review. The remembered import source remains case-local and excluded from portable design packages.
