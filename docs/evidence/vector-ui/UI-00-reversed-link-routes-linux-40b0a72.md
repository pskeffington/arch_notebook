# UI-00 reversed parallel link route regression

- **Task / owner:** Verify chart lane identity for reversed parallel relationship records / Arch Notebook maintainer
- **Source commit:** `40b0a728be450c6ff53bb2c89c22c9ade998884f`
- **Branch:** `codex/ui-00-chart-baseline`
- **Scope:** Correct a fixture regression that compared route-relative signed lane offsets without accounting for reversed endpoints. The regression now compares sampled rendered curves after normalizing path direction. Production route generation was not changed.
- **Fixture:** `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.
- **Result:** **PASS**. All 16 Rust tests pass, including the previously ignored reversed-parallel route check; 0 failed and 0 ignored.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `178554daa2fa799da921c51551e8d46796296a1ca5681237819f8002d27e0058`.

## Commands and results

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `git diff --check` | PASS, exit 0 |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS, 16 passed, 0 failed, 0 ignored |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | PASS, exit 0 |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | PASS, exit 0 |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS, exit 0; 1m 39s |

The UI test no longer treats equal signed offsets as proof that two routes overlap. For each fixture link, it builds the route using the link's actual endpoint order, reverses the sampled points for reversed records, and verifies the normalized paths are not identical. This checks the geometry used by drawing and picking while preserving duplicate records.

## Native acceptance

**BLOCKED.** A current attempt to open the release binary with the fictional fixture returned `WinitEventLoop(Os(OsError { ... WaylandError(Connection(NoCompositor)) }))`, exit 1. `wayland-info` returned `failed to create display: Operation not permitted`; X11 inspection could not open `:0`. `spectacle` captured the desktop, but its image showed an unrelated application, so it is not Arch Notebook visual evidence. No interaction sequence or Arch Notebook screenshot was accepted.

The native UI-00 interaction and visual gate remains open. This source-level test result does not satisfy the PR #3 native baseline prerequisite or establish the visual quality of the chart.
