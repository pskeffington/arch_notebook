# COMMUNITY-PROXY-01 · chart-wide bridge entities

- **Task / owner:** Add unique-neighbor and cross-community proxy ranking to detected-community analysis; implementation session.
- **Source branch / baseline:** `codex/ui-00-chart-baseline`, source baseline `15b9e403a7b064e1a4235dc9e2f79ca5b5013928`.
- **Scope:** Rust analysis and Analysis-pane UI, analyst parity documentation, and deterministic regressions.

## Result

| Gate | Result | Evidence |
| --- | --- | --- |
| Unique-neighbor counts | PASS | Community regression confirms repeated boundary records do not inflate the distinct-neighbor count: the tested bridge has 4 distinct neighboring entities. |
| Community bridge ranking | PASS | The same regression confirms the bridge touches 2 detected communities and records 6 relationship rows; the UI shows these three counts separately. |
| Headless UI visibility | PASS | The headless event test opens **Detect communities**, scrolls to **CROSS-COMMUNITY PROXIES**, and finds the expected bridge entity `A3`. |
| Focused regression | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline headless_ui_drives_geodesic_and_detected_community_actions -- --nocapture` exited 0; 1 passed. |
| Full Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0; 55 passed, 0 failed. |
| Compile check | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Formatting / patch whitespace | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native visual review | NOT RUN | This session did not validate the ranking in an interactive desktop window. |

## Method

For each valid, non-self relationship record, both endpoint entities receive the other endpoint as a distinct neighbor and the counterpart's detected community as a contact. An entity is listed when its unique direct neighbors span more than one detected community. Rows sort by distinct community contacts, distinct neighboring entities, relationship-record count, then entity UUID. Direction and duplicate records do not change the unique-neighbor or community-contact counts; relationship records remain counted separately. The community partition remains the existing deterministic greedy-modularity heuristic and is an exploratory structural grouping, not a claim about real-world affiliation or influence.

## Changed paths

- `src/main.rs`
- `README.md`
- `docs/analyst-notebook-parity.md`
- `docs/evidence/vector-ui/COMMUNITY-PROXY-01-linux.md`

Native UI review and independent code review remain open.
