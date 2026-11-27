# G4-EGUI-TESSELLATION-01 · headless UI and tessellation CPU characterization

- **Task / owner:** SNG-DELIVERY-002 G4 rendering evidence; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source commit:** `152ff5f` (`Measure headless graph UI tessellation`), based on `5db68c4`.
- **Host:** Linux x86_64; `rustc 1.100.0 (0ef1bc386 2026-11-09)`.
- **Benchmark test binary SHA-256:** `4bd351c15d2711fedd5404b2533b49fb403a9043203ad0f03fda57a4af1180cc`.
- **Raw results:** [paired 16-case JSON](artifacts/G4-egui-tessellation-paired-20261126.json), SHA-256 `3edee7b330af0b2cc3f2c5d8d2ebe63050bd3fff816e45d0ee678084bba226d5`.

## Method

The opt-in ignored test drives the production `App::update` through an egui headless context at 1440×900 points, then separately times `Context::tessellate` on that frame's actual clipped shapes. Each case has five warmups and 30 timed samples. The runner builds the release-profile test binary offline, generates fictional sparse, hub, disconnected, and dense charts at 100 and 1,000 entities, and runs paired neutral-pan and 2,000-point horizontal-pan samples. Pan order alternates by fixture to reduce order effects. Fixture hashes, test-binary hash, frame shape counts, tessellated primitive counts, mesh vertices and indices, and median/p95 distributions are retained in the JSON report.

The frame-build timer encloses `Context::run`, including application update, egui layout, and shape generation. Tessellation is a separate CPU timer. Neither timer includes GPU execution, compositor work, buffer presentation, input-to-display latency, or FPS.

## Results

| Fixture | Entities / records | Shapes, neutral → panned | Mesh vertices, neutral → panned | Tessellation median, neutral → panned | Median change |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sparse | 100 / 300 | 5,511 → 1,411 | 112,214 → 12,218 | 2.653 → 0.127 ms | −95.2% |
| Sparse | 1,000 / 3,000 | 53,211 → 12,211 | 1,033,678 → 32,038 | 13.908 → 1.108 ms | −92.0% |
| Hub | 100 / 300 | 5,511 → 1,411 | 112,182 → 12,186 | 3.089 → 0.291 ms | −90.6% |
| Hub | 1,000 / 3,000 | 53,211 → 12,211 | 1,033,650 → 32,010 | 14.153 → 1.258 ms | −91.1% |
| Disconnected | 100 / 200 | 4,038 → 1,310 | 92,082 → 11,606 | 2.358 → 0.206 ms | −91.3% |
| Disconnected | 1,000 / 2,000 | 38,538 → 11,210 | 832,938 → 26,018 | 10.718 → 0.671 ms | −93.7% |
| Dense | 100 / 1,000 | 15,775 → 2,111 | 252,358 → 16,410 | 3.879 → 0.304 ms | −92.2% |
| Dense | 1,000 / 10,000 | 155,875 → 19,211 | 2,438,006 → 74,030 | 29.979 → 1.746 ms | −94.2% |

For these pairs, shifting the chart fully outside the viewport reduces emitted geometry and CPU tessellation work. This is workload characterization, not an isolated culling A/B: the neutral and panned frames expose different visible graph content, and the report does not establish a universal speedup or equal-content renderer improvement. The existing culling regression separately verifies that the production draw path skips offscreen routes and labels.

## Commands and gate results

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | **PASS**, exit 0 |
| `git diff --check` | **PASS**, exit 0 |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, 60 passed, 0 failed, 1 ignored benchmark |
| `python3 scripts/benchmark-chart-egui-tessellation.py --output docs/evidence/vector-ui/artifacts/G4-egui-tessellation-paired-20261126.json --iterations 30 --sizes 100 1000 --compare-pan 2000 0` | **PASS**, 16 paired fixture/view cases, exit 0 |
| Native draw/presentation performance | **NOT MEASURED**; host capture/input access is unavailable or inconsistent. |

G4 now has measured CPU frame-build and tessellation characterization plus geometry culling evidence. Native visual review, GPU/compositor work, and presentation performance remain open. This does not close G4's native rendering gate or G3's Arrange/Restore interaction and layout-quality gate.

## Reproduction

```sh
python3 scripts/benchmark-chart-egui-tessellation.py \
  --output /tmp/g4-egui-tessellation-paired.json \
  --iterations 30 --sizes 100 1000 --compare-pan 2000 0
```

This command writes JSON containing the release test-binary SHA-256 and all 16 raw cases. To compare outputs, use the same host, Rust toolchain, source and profile. Synthetic fixtures are fictional and are generated in a temporary directory.
