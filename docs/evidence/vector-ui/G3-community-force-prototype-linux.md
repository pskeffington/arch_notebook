# G3 community-aware force layout prototype — Linux source acceptance

- **Task/owner:** G3 chart layout prototype / Codex implementation
- **Source baseline:** `320daadb0583617536c4f4909ca5ebc2f8d14c69` on `codex/ui-00-chart-baseline`
- **Scope:** deterministic display coordinates, community grouping force, configurable iteration count, pinned-node preservation. No case schema or graph topology changes.
- **Changed paths:** `src/layout.rs`, `src/main.rs`, `docs/roadmaps/vector-community-exploration-roadmap.md`, this evidence record.
- **Release artifact:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `99a37a9252603542d7e99e3e13ae9f312791ca455adc83555914889d337f174d`.

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| Formatting | **PASS** | `cargo fmt --all -- --check` — exit 0 |
| Whitespace | **PASS** | `git diff --check` — exit 0 |
| Rust tests | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` — 44 passed, 0 failed, exit 0 |
| Offline compile check | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` — exit 0 |
| Optimized Linux build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` — exit 0 |
| Native interaction and visual quality | **BLOCKED** | No input-capable application review in this run. |
| Sparse, hub, disconnected, dense performance comparison | **NOT RUN** | Required G0 comparison and large-chart timing remain open. |
| Restore/undo UX | **NOT RUN** | Existing alternate layout modes remain available; one-action restoration of the pre-force positions is not implemented. |

## Implementation limits

The opt-in chart arrangement uses a fixed seed, up to 120 iterations, a 12×12 grid-centroid repulsion approximation, relationship attraction, and weak attraction toward detected-community centroids. The menu exposes iterations from 10 to 120 (default 50). Pinned entities retain exact coordinates. Tests establish repeatability for fixed ordered inputs/settings, finite bounded output on the fixture, pin preservation, and empty/singleton behavior; they do not establish attractive layouts or acceptable latency at production graph sizes. Community assignments remain exploratory structural descriptions. Spatial proximity is not evidence of social distance, causality, or importance.

**Acceptance:** source prototype **PASS**; G3 overall **IN PROGRESS**; native and workload gates **NOT ACCEPTED**.
