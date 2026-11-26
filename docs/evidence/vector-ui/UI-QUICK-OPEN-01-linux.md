# UI-QUICK-OPEN-01: Searchable workspace navigation on Linux

- **Result:** PASS for source, build, unit, and X11 interactive acceptance; native Wayland launch remains blocked in this session.
- **Owner:** Arch Notebook implementation session.
- **Baseline:** `39bcf5ce6896b85f39ed2c5d2f8a6a1132ebaa3e` on `codex/ui-00-chart-baseline`.
- **Scope:** Add visible Quick open and Ctrl/Command+K, searchable workspace and chart actions, and direct routing to SQL Review.
- **Changed paths:** `src/main.rs`, `README.md`, `docs/analyst-notebook-parity.md`, and this evidence record.
- **Data:** Only the fictional checked-in correctness fixture was opened. Its SHA-256 before/after was `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0 | Rust source formatting is clean. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS, exit 0 | 17 tests passed, including case-insensitive label/keyword matching for the palette. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | PASS, exit 0 | Full development profile check completed. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | PASS, exit 0 | Debug application built. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS, exit 0 | Optimized application built in 1m48s. |
| `git diff --check` | PASS, exit 0 | No whitespace errors. |
| Open the visible **Quick open** button, type `SQL`, then press Enter | PASS | Filter showed SQL Review and Automation. Enter opened SQL Review. Screenshot: `/tmp/arch-notebook-quick-open-sql-route.png`, SHA-256 `acd282938e51d21dd3d1eec386a7f062ffbd67486d3c21866971785654ab25ab`. Filter screenshot: `/tmp/arch-notebook-quick-open-filter-sql.png`, SHA-256 `6529401ca0905476d3be8ff15c71285efd0826fe51843dca386c458f5484e504`. |
| Press Ctrl+K from the chart | PASS | Quick open appeared in the running app. Screenshot: `/tmp/arch-notebook-quick-open-shortcut3.png`, SHA-256 `b6c85b7ab394d71464521f5be33864bce8e84ba82e43d3455ae27f54b2f5ad80`. |

The Wayland launch attempt returned `WinitEventLoop(OsError: WaylandError(Connection(NoCompositor)))`. The interactive checks therefore used the same debug binary with `WAYLAND_DISPLAY` unset; the X11 window opened and accepted pointer and keyboard input. The app session was stopped after capture; no fixture changes were saved. Screenshot files remain in `/tmp` and are not committed.

## Remaining work

This pass verifies workspace routing and one SQL Review destination in the running application. It does not complete the full UI-00 navigation sequence, compare the visual experience directly with IBM i2 Analyst's Notebook, validate IBM import behavior, or verify live scheduling and restart recovery. Continue with those analyst workflows and re-run native Wayland acceptance when a compositor is available.
