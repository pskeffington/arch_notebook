# G2-DIRECTION-LEGEND-01 · community interaction count legend

- **Task / owner:** SNG-DELIVERY-002 G2 count interpretation follow-up; implementation session.
- **Baseline:** `e3ec3868268d1716dfd7e00e2dbd46b630369a65` on `codex/ui-00-chart-baseline`.
- **Candidate source:** `7ca85ec77754b57ddab07fb6bf5f0d4b4a97d766`.
- **Scope:** Clarify the meaning of direction symbols and endpoint-pair counts in detected-community interaction rows.

## Change

The compact row continues to show relationship records, distinct endpoint pairs, and `→ / ← / ↔` record counts. Its hover explanation now states that arrows follow the displayed community numbering, `↔` denotes undirected records, record totals retain repeats, and distinct unordered endpoint pairs collapse reversed or parallel records. No analysis or saved chart data changes.

## Verification

| Check | Result |
| --- | --- |
| Offline Rust suite: `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, 61 passed, 0 failed, 1 ignored; exit 0 |
| `cargo fmt --all -- --check`; `git diff --check` | **PASS**, exit 0 |
| Offline release build: `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | **PASS**, exit 0; 2m49s |
| Current-user install | **PASS**; installed launcher hash matches candidate binary `da469fb1eaf19675a25cf84581dbd001fc6dd56d524b3cc3cf77bfe92fbbca7b`; `--version --json`, desktop entry validation, and byte comparison passed |
| Independent source review | **PASS**; see review handoff below |
| Native candidate launch | **BLOCKED**; `WaylandError(Connection(NoCompositor))`, exit 1 |
| Native visual/input review | **NOT TESTED**; no window appeared |

The deterministic sparse-30 fixture remains `/tmp/arch-notebook-review-sparse30.case.json` (30 fictional entities, 90 relationship records; SHA-256 `bf88b5df8c2c8e259e41a52486d8f09a12f6a6665d3a60892c802d8c06fa872d`). The native G2 action gate remains open for a host session with a compositor and input access.

## Reviewer handoff

- The reviewer inspected only the scoped tooltip change in `src/main.rs`, without editing files or running tests.
- **Gate status:** count and direction semantics remain covered by the existing interaction fixture; tooltip source/build/install checks PASS; native visual/input review NOT TESTED; no release claim.
