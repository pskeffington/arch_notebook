# SQL saved-search automation draft guard

- **Task / owner:** Continue the Rust Analyst's Notebook workflow; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `eab5da052a2fae8570d44540776c0bcca9513983`.
- **Scope:** Track which saved SQL definition is loaded into SQL Review separately from the catalog selection; gate automation preparation until the selected search is loaded; allow unseeded searches to prepare drafts while preserving typed parameters.
- **Release binary SHA-256:** `209f105166ee3ef47425aa5349c43118a1d07991f9f741fecf5cf23144b6f0a8`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Focused regressions | PASS | `cargo test --locked --offline automation_draft`: 2 passed. Tests verify mismatch does not mutate the active query/parameters, unseeded loaded search preserves typed parameters and analyst SQL edits, stale preview is cleared, and no job/snapshot is created; chart-seeded handoff still strips its bound value and retains its warning. `cargo test --locked --offline sql_review_saved_search_ui_loads_query_without_running_or_capturing_it`: 1 passed; headless UI confirms both preparation controls are available after their definitions are loaded. |
| Full Rust suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`: 103 passed, 0 failed, 3 ignored. |
| Formatting, compiler check, whitespace | PASS | `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, and `git diff --check` exited 0. |
| Optimized release build | PASS | `TMPDIR=/home/sysadmin/.cache/arch-notebook-build-tmp CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline`, exit 0. |
| Isolated installer acceptance | PASS | [Installer aggregate JSON](artifacts/SQL-SEARCH-AUTOMATION-DRAFT-02-install.json), SHA-256 `e2ea53a22ccadb832e58564814af3ced8c06b79c8cee95e106aa9580cefb822f`. Binary, launchers, shortcut mode, SVG/PNG, helper and MIME package checks passed. |
| Current-user install | PASS | `bash scripts/install-linux-user.sh /tmp/arch-notebook-linux-x86_64`, exit 0. Installed `/home/sysadmin/.local/bin/arch-notebook` SHA-256 matches the release. |
| Clinton primary corpus QA | PASS, bounded | `python3 scripts/test-clinton-corpus-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/CLINTON-CORPUS-QA-search-draft-02.json`, exit 0. One query-only preflight passed; 6,173 rows replayed with zero skipped rows or capture deltas; source inputs unchanged. [Aggregate-only JSON](artifacts/CLINTON-CORPUS-QA-search-draft-02.json), SHA-256 `6c56a155c0b3eaaa89abd1bee5a79045234ee53567f35da4aacbbfd545693643`. |
| Native UI, live timer delivery, IBM interoperability | NOT RUN | Headless UI regression does not establish pointer interaction. User-systemd access and IBM schema/application validation are unavailable. |

The Clinton report is default-redacted; it contains no names, relationship labels, IDs, or evidence references. The run does not establish PDF extraction completeness, Qwen analysis, or human content review.
