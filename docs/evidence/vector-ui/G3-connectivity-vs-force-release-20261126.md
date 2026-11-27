# G3: G0 Connectivity versus Community Force (release profile)

- **Task / owner:** SNG-DELIVERY-002 G3 baseline comparison; implementation session.
- **Repository / branch / source:** `/home/sysadmin/github/arch_notebook`, `codex/ui-00-chart-baseline`, `e86814675eaee96ed6f20caff94d6d5b79b9a2c5`.
- **Follow-up test source:** a headless Arrange/Restore interaction regression was added afterward in `src/main.rs`; it does not change the release binary (`#[cfg(test)]`).
- **Release binary:** `/tmp/arch-notebook-release-verify/release/arch-notebook`; SHA-256 `d4bfcdc0e8f7fb3c374e92b4ebca2e21b2920e3211550a2b9536b6ac88d35676`.
- **Host:** Linux x86_64; `rustc 1.100.0 (0ef1bc386 2026-11-09)`.
- **Raw results:** [12-case paired benchmark JSON](artifacts/G3-connectivity-vs-force-release-20261126.json), SHA-256 `b3bf1dbfb7981f4b2dd3a8939a8240ecd506aab996f98fabe1777e9e2e86572f`.

## Method

The benchmark runs the production G0 `connectivity_layout` and production community force layout on identical deterministic fictional fixtures, including the same node positions and pins. Each path receives one warmup and 25 timed samples; their execution order alternates per sample. Community detection is a separate single sample and is not included in either repeated layout timing. The benchmark excludes painting, text shaping, GPU, compositor, presentation, and input latency.

These are coordinate-routine timings, not full user-action latency. G0 degree counts and force-layout community assignments are prepared before the repeated coordinate timers; community detection has a separate one-off measurement, while G0 degree-count preparation is not timed here.

## Release results

Times are milliseconds. The ratio is force-layout median divided by G0 median. The algorithms do different work and produce different arrangements, so this is a measured workload comparison, not an equal-quality speedup claim.

| Fixture | Nodes | Records | Communities | G0 median | Force median | Force p95 | Median ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Sparse | 1,000 | 3,000 | 61 | 0.067 | 14.768 | 18.497 | 220× |
| Sparse | 5,000 | 15,000 | 310 | 0.292 | 54.497 | 55.610 | 187× |
| Sparse | 10,000 | 30,000 | 635 | 0.637 | 116.439 | 120.522 | 183× |
| Hub | 1,000 | 3,000 | 393 | 0.051 | 13.711 | 74.314 | 269× |
| Hub | 5,000 | 15,000 | 2,087 | 0.291 | 76.646 | 77.707 | 263× |
| Hub | 10,000 | 30,000 | 4,259 | 0.647 | 164.282 | 167.344 | 254× |
| Disconnected | 1,000 | 2,000 | 237 | 0.058 | 12.466 | 48.362 | 216× |
| Disconnected | 5,000 | 10,000 | 1,192 | 0.290 | 65.755 | 66.888 | 227× |
| Disconnected | 10,000 | 20,000 | 2,372 | 0.656 | 140.199 | 143.293 | 214× |
| Dense | 1,000 | 10,000 | 444 | 0.050 | 16.527 | 16.657 | 332× |
| Dense | 5,000 | 50,000 | 2,230 | 0.292 | 88.849 | 89.699 | 304× |
| Dense | 10,000 | 100,000 | 4,462 | 0.645 | 185.122 | 197.190 | 287× |

## Commands and gate results

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | **PASS**, exit 0 |
| `git diff --check` | **PASS**, exit 0 |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, 54 passed, 0 failed |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | **PASS**, exit 0 |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-release-verify cargo build --release --locked --offline -j 1` | **PASS**, exit 0 |
| `python3 scripts/benchmark-community-layout.py --binary /tmp/arch-notebook-release-verify/release/arch-notebook --output /tmp/arch-notebook-g3-g0-vs-force-f03db52.json --iterations 20 --samples 25` | **PASS**, 12 cases, exit 0 |

## Limits and next work

- The G0/G3 CPU coordinate comparison gate **PASS**es on sparse, hub, disconnected, and dense fixtures at 1,000, 5,000, and 10,000 entities.
- The force layout takes 13–185 ms at the recorded medians; it remains an explicit, off-UI-thread arrangement action. No universal performance target or equal-layout-quality claim is made.
- Headless `App::update` UI acceptance clicks View → Arrange, verifies the worker completes and a pinned entity stays fixed, then clicks Restore and verifies all original coordinates and pin flags. This simulated event path passes as part of the 54-test suite; it is not native desktop acceptance.
- Independent review of the new headless regression is **NOT VERIFIED**; this session performed an implementation-owner self-review and found no separate reviewer handoff in the repository.
- Native arrange/restore interaction, accessibility, and visual quality remain **NOT RUN** because this session has no usable display. This benchmark does not close G3 native acceptance or G4 renderer performance.
- Spatial arrangement is exploratory display geometry and does not encode social distance, causality, or importance.
