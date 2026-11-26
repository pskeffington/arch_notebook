# UI-01 entity card hit geometry evidence

- **Task / owner:** Correct chart selection to use painted entity-card bounds and shared viewport transforms; implementation session.
- **Starting source commit:** `0c5cf7bd59b1a66713e165943962354a1c7f6ae0` on `codex/ui-00-chart-baseline`.
- **Scope:** `src/main.rs`, `docs/notebook-workflows.md`, and this evidence record. Entity cards receive click and context-menu priority over link picking; viewport conversion is shared for chart placement, zoom anchoring, and drag coordinates. Stored normalized positions are unchanged.

## Acceptance results

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting | PASS | `cargo fmt --all` and `cargo fmt --all -- --check` exited 0. |
| Diff whitespace and removal of the 75-point radial card target | PASS | `git diff --check` exited 0; source search found no remaining `distance(pos) < 75` chart hit test. |
| Rust unit suite | PASS | `cargo test --locked --offline` exited 0; 48 passed, 0 failed. New tests cover normalized-coordinate round trips, stationary pointer zoom anchoring, degenerate viewports, visible card edges, and topmost-card precedence. |
| Offline compile check | PASS | `cargo check --locked --offline` exited 0. |
| Strict Clippy | FAIL | `cargo clippy --locked --offline --all-targets -- -D warnings` failed on existing `possible_missing_else` findings at SQL editor and connection-table code outside this change. Retrying with that lint allowed exposed 93 additional warning-as-error findings across existing code. No unrelated lint cleanup was included. |
| Current-candidate native pointer and visual review | BLOCKED | This session has no usable desktop input/capture bridge (previous checks report inaccessible X11 and denied Wayland input). The new interaction was not exercised in the running application; no screenshot acceptance is claimed. |
| IBM Analyst's Notebook parity | NOT TESTED | No IBM application or XSD was used. |

The navigator and timeline click branches remain ahead of chart item picking. In normal chart picking, the last painted entity card containing the pointer wins; only when no entity card or label is hit does the app pick and cycle visible relationship routes.
