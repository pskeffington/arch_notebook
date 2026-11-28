# Proximity-map encoding legend — 2026-11-28

**Task / owner:** Clarify the sampled topology map's visual encodings; Rust UI implementation.

**Repository / branch / baseline:** `pskeffington/arch_notebook` / `codex/ui-00-chart-nested-conditions` / `81217fe8df8c466a4cf68030d1715f3abcfb5771`.

## Change

The sampled contact topology now places a compact wrapped legend between its coverage summary and the map. It explains the gold origin group, blue groups in later hop layers, yellow proxy ring, direction arrow, selected and hovered tie styling, and thicker lines for larger relationship-record counts. This is explanatory UI only; topology, sampling, record selection, and chart data are unchanged.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Focused topology UI regression | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline headless_ui_drives_geodesic_and_detected_community_actions` exited 0; the production headless panel asserts all seven legend explanations. |
| Retained Clinton topology UI regression | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target ARCH_NOTEBOOK_CLINTON_CASE_PATH=/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json cargo test --locked --offline clinton_case_topology_map_renders_at_real_corpus_scale -- --ignored --nocapture` exited 0. The test exercised the real 2,813-entity / 6,173-record case and rendered 48 sampled groups, 47 contact pairs, and 1,315 represented records. |
| Full Rust suite, formatting, diff | PASS | `cargo fmt --all -- --check`, `git diff --check`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0; 94 passed, 0 failed, 3 ignored. |
| Locked offline release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0. SHA-256: `84d34b9581beba8286f5a8a490ca128cbef59f2652ae871b31adcf439c91094f`. |
| Current-user installer and launcher assets | PASS | `scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report /tmp/proximity-map-legend-install-acceptance-final.json` exited 0. Installed binary matches the release; desktop entry, shortcut, SVG/PNG icons, MIME definition, and SQL automation helper match package assets. |
| Installed application | PASS | `/home/sysadmin/.local/bin/arch-notebook --version --json` reported `arch-notebook` 0.1.0; installed binary SHA-256 matches the release. |
| Native screenshot / pointer review | BLOCKED | `wayland-info` failed with `Operation not permitted`; X11 `xwininfo` could not open `:0`. No native appearance or pointer-feel claim is made. |
| IBM Analyst's Notebook visual parity | NOT TESTED | No IBM installation was available or exercised. |

The Clinton regression also reports headless frame-build and tessellation samples, but those are not GPU, compositor, or frame-rate measurements. No native visual acceptance is claimed.
