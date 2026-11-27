# G4-CULLING-AB-01 · matched route-culling CPU comparison

- **Task / owner:** SNG-DELIVERY-002 G4 rendering evidence; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source commit:** `c1ba6f72e643e2cc0123eab40600634db24685f5`.
- **Host / toolchain:** Linux x86_64; `rustc 1.100.0 (0ef1bc386 2026-11-09)`.
- **Release test binary SHA-256:** `650522285626ba9f702ddd2faa90ac0a9f7f5779db64c0d3cf57fb6ca6acd85e`.
- **Raw results:** [16-case matched culling A/B JSON](artifacts/G4-culling-ab-20261126.json), SHA-256 `d55ac7fa6c0d67b8bb5eb5346542b346b62e0a7fe1f5f10566946a65ec64342d`.

## Method

For every fictional fixture and pan setting, the benchmark runs the production `App::update` and egui tessellator twice with separate app/context state. The only difference is whether the production relationship-route culling branch is active. With culling disabled, the test-only branch skips the viewport-bounds helper completely and submits offscreen route shapes for egui to clip. The override exists only under `cfg(test)`; the normal application build uses the production culling branch.

The suite covers sparse, hub, disconnected, and dense fixtures with 100 and 1,000 entities, at neutral pan and a 2,000-point horizontal pan. Each variant receives five warmups and 30 timed samples. Culling variants alternate order within every sample pair; the initial order alternates by fixture/view. The viewport is 1440×900 points and zoom is 1.0. The frame-build timer includes `Context::run`, application update, UI layout, and shape generation. CPU tessellation is timed separately.

## Results

| Fixture | Entities / records | Panned shapes omitted by culling | Frame-build median reduction | CPU tessellation median reduction |
| --- | ---: | ---: | ---: | ---: |
| Sparse | 100 / 300 | 4,100 | 32.7% | 40.4% |
| Sparse | 1,000 / 3,000 | 41,000 | 36.2% | 57.3% |
| Hub | 100 / 300 | 4,100 | 36.8% | 47.7% |
| Hub | 1,000 / 3,000 | 41,000 | 35.6% | 60.3% |
| Disconnected | 100 / 200 | 2,728 | 20.9% | 29.8% |
| Disconnected | 1,000 / 2,000 | 27,328 | 31.3% | 51.8% |
| Dense | 100 / 1,000 | 13,664 | 45.1% | 63.6% |
| Dense | 1,000 / 10,000 | 136,664 | 42.2% | 77.3% |

At neutral pan, culling on and off emitted identical shape counts in all eight fixtures. Frame-build medians with culling enabled were 0.3–6.8% slower, consistent with paying the bounds-check cost when all routes remain visible. The panned cases isolate the benefit: culling omits offscreen route draw shapes, reducing frame-build medians by 20.9–45.1% and egui CPU tessellation medians by 29.8–77.3% on this host.

The emitted mesh vertex counts were unchanged between culling variants because egui clips the fully offscreen shapes during tessellation. The result demonstrates CPU-side reduction in UI shape submission and tessellation traversal; it does not show fewer vertices in the final mesh, GPU execution improvement, compositor cost, buffer presentation, input-to-display latency, FPS, or native visual quality. These are host-specific synthetic workload results, not universal performance targets.

## Commands and gate results

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` and `git diff --check` | **PASS**, exit 0 |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, 60 passed, 0 failed, 1 ignored benchmark |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | **PASS**, exit 0 |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | **PASS**, exit 0 |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | **PASS**, exit 0 |
| `python3 scripts/benchmark-chart-egui-tessellation.py --output docs/evidence/vector-ui/artifacts/G4-culling-ab-20261126.json --iterations 30 --sizes 100 1000 --compare-pan 2000 0` | **PASS**, 16 fixture/view cases, 32 measured variants, exit 0 |
| Independent implementation review | **PASS** for test-only bypass isolation and matched sampling design; the reviewer did not run tests or builds. |
| Native GPU/presentation and visual/input review | **NOT MEASURED**; X11 access is unavailable in this session. |

This closes the controlled CPU-side route-culling comparison for G4. Native renderer, presentation, and visual/input acceptance remain open, as does G3 native Arrange/Restore and layout-quality review.
