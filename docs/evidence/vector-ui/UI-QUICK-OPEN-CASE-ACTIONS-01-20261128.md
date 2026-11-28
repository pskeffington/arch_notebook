# Quick open case actions — 2026-11-28

**Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-nested-conditions` / `0291865fd639de1656e94f86792902df978cb0e3`.

Quick open now includes **Open case** and **New case**, searchable by action name and useful terms. Both commands route through the existing case-action guard, so an unsaved edit continues to show the current discard confirmation before loading or replacing a case.

| Gate | Result | Evidence |
| --- | --- | --- |
| Quick command search and unsaved-change route | PASS | `quick_command_search_matches_labels_and_keywords_case_insensitively` and `quick_case_actions_keep_the_unsaved_change_confirmation` passed. |
| Full offline Rust workspace suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline`: 79 passed, 0 failed, 3 ignored. |
| Formatting, locked offline check, debug build | PASS | `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0. |
| Release build for this source | BLOCKED | The combined release build stopped returning output; its command session required interruption (exit 130). A one-job retry (`CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline`) also stopped returning output; no Cargo/rustc process remained, the command session required interruption (exit 130), and the existing artifact timestamp/hash remained unchanged. The release artifact on disk is from the previous source and was not used as evidence for this change. `/tmp` was 91–92% full (1.4–1.5 GiB available). |
| Native display and pointer acceptance | NOT RUN | No accessible native display path was available in this session. |

Exact source candidate is recorded in the commit containing this evidence. `git diff --check` exited 0 before commit.
