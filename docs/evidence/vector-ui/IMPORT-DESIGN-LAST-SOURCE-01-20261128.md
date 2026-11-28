# Saved import design last-source reinspection

- **Task / owner:** Resume `arch_notebook` Rust product work; implementation session.
- **Repository / branch:** `/tmp/arch-notebook-remote-candidate` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `60bf63751e31beb427e3667ad4e5e7d3a71741aa`.
- **Scope:** A successfully used case-local CSV/TSV import design can recall and reinspect its last source file. Reinspection opens the existing import inspection/mapping flow and does not import chart data. Its source path is excluded from portable design packages.
- **Candidate binary SHA-256:** `6f014b594f85a0c35b9a78f3f822459649e4a05489af431cfcf8f0bf439af081`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Focused persistence, reinspection, package exclusion, replacement, and legacy JSON regression | PASS | `saved_import_design_reinspects_last_source_without_importing_and_keeps_path_local`; included in the full Rust suite. |
| Full Rust suite | PASS | 102 passed, 0 failed, 3 ignored. |
| Formatting, locked offline workspace check, and diff validation | PASS | `cargo fmt --all -- --check`, `git diff --check`, and `TMPDIR=/home/sysadmin/.cache/arch-notebook-build-tmp CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline` exited 0. |
| Optimized release build | PASS | `TMPDIR=/home/sysadmin/.cache/arch-notebook-build-tmp CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline` exited 0. |
| Isolated installer acceptance | PASS | `python3 scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report /tmp/IMPORT-DESIGN-LAST-SOURCE-01-install.json` exited 0. Aggregate report: [installer JSON](artifacts/IMPORT-DESIGN-LAST-SOURCE-01-install.json), SHA-256 `a5a049089aba15164bc9b7a4cd97992fd6f9b3ff87b302d45ffcaedf91139711`. |
| Current-user desktop install | PASS | `bash scripts/install-linux-user.sh /tmp/arch-notebook-linux-x86_64` exited 0. `/home/sysadmin/.local/bin/arch-notebook` matches candidate SHA-256; menu entry, Desktop shortcut, icons, and MIME definition synchronized. |
| Primary Clinton corpus QA | PASS, bounded | `python3 scripts/test-clinton-corpus-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/CLINTON-CORPUS-QA-import-design.json` exited 0. One query-only preflight passed; 6,173 relationship rows replayed with zero skipped rows and zero added/changed/removed deltas; PDF, case, and database inputs unchanged. Aggregate report: [Clinton JSON](artifacts/IMPORT-DESIGN-LAST-SOURCE-01-clinton-aggregate.json), SHA-256 `abc34d9db3ecdaa0955ba4503dd7c986e24c6cc262be1dc007c546d3a3373269`. |
| Native UI and IBM application/format interoperability | NOT RUN | Native pointer/display review unavailable. This does not implement IBM `.ximp`/`.oimp` compatibility. |

The aggregate Clinton report is default-redacted and contains no names, relationship labels, IDs, or evidence references. Its topology summary reports 2,813 entities, 6,173 relationship records, 643 detected communities, and 291 proxy candidates. PDF extraction completeness, Qwen analysis, and human content review are not established by this run.
