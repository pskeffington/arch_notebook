# Quick open case actions — 2026-11-28

**Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-nested-conditions` / `0291865fd639de1656e94f86792902df978cb0e3`.

Quick open now includes **Open case** and **New case**, searchable by action name and useful terms. **Chart** remains the first result, preserving the prior Enter behavior when the search is empty. Both commands route through the existing case-action guard, so an unsaved edit continues to show the current discard confirmation before loading or replacing a case.

| Gate | Result | Evidence |
| --- | --- | --- |
| Quick command search and unsaved-change route | PASS | `quick_command_search_matches_labels_and_keywords_case_insensitively`, `quick_case_actions_keep_the_unsaved_change_confirmation`, `quick_open_case_commands_reach_unsaved_change_confirmation`, and `quick_open_enter_without_a_query_keeps_chart_as_the_default` passed. Headless UI tests click both case commands and send Enter with an empty search. |
| Full offline Rust workspace suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline`: 81 passed, 0 failed, 3 ignored. |
| Formatting, locked offline check, debug build | PASS | `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. The debug profile was later cleared to free `/tmp` space, then the source was rebuilt by the full workspace test and locked check. |
| Optimized release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0 in 2m58s; binary SHA-256 `8381bd57bdf84017377d92ef62916bde737e05ebe15dbca4b98317ec4b62ed27`. Earlier attempts had been interrupted before a result. |
| Native display and pointer acceptance | NOT RUN | No accessible native display path was available in this session. |

Exact source candidate is recorded in the commit containing this evidence. `git diff --check` exited 0 before commit.
