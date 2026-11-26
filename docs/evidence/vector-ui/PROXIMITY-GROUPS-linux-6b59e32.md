# Origin-centered proximity groups evidence

- **Task / owner:** Add distance-based contact grouping and proxy ranking to chart analysis; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source baseline:** `941373e3b8fa537c7ee922c14ab0435e488a3009`.
- **Candidate source commit:** `6b59e32524c1ee487394ca1235cef4716d8937ae`.
- **Changed paths:** `src/proximity.rs`, `src/main.rs`, `docs/notebook-workflows.md`, `docs/analyst-notebook-parity.md`, and this report.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `bd80d18f428655b095484d2f84ddd29ce5c4d369b6169e77d6040766b11d6309`.

## Behavior

The selected chart entity is the origin. BFS treats relationships as undirected and unweighted for shortest-hop assignment. Entities in the same hop layer that contact one another are combined into a connected contact group. The Analysis pane summarizes group-to-group relationship records, distinct unordered endpoint pairs and direction counts. Each aggregate retains exact link indices; selecting a contact pair exposes individual records and can focus them on the chart. Proxy candidates rank entities that directly contact at least two other proximity groups, with distinct-group, distinct-counterpart and cross-group-record counts. This score describes chart structure, not intent or influence. Unreachable entities are counted separately. Analysis is transient and does not modify persisted entities or links.

The chart remains an entity/relationship chart. This pass adds a group-level Analysis-pane roll-up and navigation back to the original group members and records; it does not replace the chart canvas with synthetic group nodes.

## Acceptance results

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Rust tests | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0; 50 passed, 0 failed. New cases cover missing origins, hop assignment, same-layer connected groups, parallel record preservation, direction totals, distinct endpoint pairs and proxy ranking. |
| Locked offline compile | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Optimized release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0 in 2m40s. Artifact SHA-256 is recorded above. |
| GitHub Actions workflow for candidate | PASS | Manual workflow dispatch run [45909562127](https://github.com/pskeffington/arch_notebook/actions/runs/45909562127) targets exact candidate `6b59e32524c1ee487394ca1235cef4716d8937ae`; Linux tests/fixture determinism, Linux x86_64, Linux ARM64, macOS Intel, macOS Apple Silicon, and Arch Linux package jobs all completed successfully. Release-asset attachment was skipped because the event was not a published release. |
| Native chart interaction and visual review | BLOCKED | Launch of the exact release binary with the fictional correctness fixture exited 1: Winit `WaylandError(Connection(NoCompositor))`. X11 is also unavailable. No pointer action or screenshot review is claimed. |
| IBM application parity | NOT TESTED | No IBM Analyst's Notebook application or XSD was used. |
| Independent review | NOT PERFORMED | This report records implementation-owner checks only. |

## Commands and observed outcomes

```text
cargo fmt --all -- --check
exit 0

git diff --check
exit 0

CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline
exit 0 (50 passed, 0 failed)

CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline
exit 0

CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline
exit 0 (optimized profile finished in 2m40s)

env DISPLAY= WAYLAND_DISPLAY=wayland-1 XDG_RUNTIME_DIR=/run/user/1000 \
  HOME=/tmp/arch-notebook-ui00-current/home \
  XDG_DATA_HOME=/tmp/arch-notebook-ui00-current/data \
  XDG_CONFIG_HOME=/tmp/arch-notebook-ui00-current/config \
  ARCH_NOTEBOOK_CHART_DIAGNOSTICS=/tmp/arch-notebook-ui00-current/chart-diagnostics.jsonl \
  /tmp/arch-notebook-target/release/arch-notebook --open-case \
  "$PWD/fixtures/chart/correctness.case.json"
exit 1 (Wayland connection returned NoCompositor)
```

The fixture is the fictional checked-in correctness case, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`. The launch failed before rendering; no chart interaction, Qwen review, IBM compatibility, or native acceptance is claimed.

## Handoff

The source, unit, offline compile and optimized-build gates pass. The remote workflow and native visual gates are pending. The next native review should run **Measure hop layers**, inspect grouped contacts and proxies, verify that exact parallel records open the correct chart relationship, and compare the member-level chart to the group summary. Keep the unresolved UI-00 native baseline gate visible in the vector UI roadmap.
