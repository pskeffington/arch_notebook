# UI-ENTITY-TYPE-LAYOUT-02: Aspect-aware entity type clusters

- **Result:** PASS for deterministic layout calculations, Rust tests, offline builds, and local launcher synchronization; native visual interaction remains BLOCKED.
- **Task / owner:** Continue analyst-facing chart organization; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `0e54d8d70378a1a9f47f1268f679e381d105e997`.
- **Source commit:** `a09b49c` (`feat: adapt entity clusters to chart canvas`).
- **Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `README.md`.
- **Rust source SHA-256:** `5e1ec2057de7b85d492f6e79f4b8ee3880a75788e28ee788a45686e3705142a3`.
- **Build/install work directory:** `/tmp/arch-notebook-target` and `/tmp/arch-notebook-local-install-20261126b/arch-notebook-linux-x86_64`.

Entity-type layout now distributes type clusters across a grid based on the most recently rendered chart aspect ratio. It retains type/name ordering and pinned entity coordinates, and centers partial rows inside each cluster. Regression tests cover pin preservation, stable sorting, aspect-driven cluster columns/rows, and partial-row centering.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0 | Formatting is clean. |
| `git diff --check` | PASS, exit 0 | No whitespace errors. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS, exit 0 | 20 passed, 0 failed, 0 ignored. Includes the new aspect-grid and partial-row regressions. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | PASS, exit 0 | Offline integration check completed. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS, exit 0 | Optimized Rust build completed in 1m 42s. |
| User install | PASS, exit 0 | `scripts/install-linux-user.sh` installed the candidate from the staged archive directory. |
| Installed binary and launcher | PASS | Release and installed binary SHA-256 both equal `54d57b05cd9ebb66de4b60274fb4062e06a96705f4a68f80eabb7649e249bdb7`. Installed version command returned `{"name":"arch-notebook","schema_version":1,"version":"0.1.0"}`. Both desktop entries passed `desktop-file-validate` and `cmp` confirmed they are identical. |
| Installed graphics | PASS | Repository and installed SVG SHA-256: `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8`; PNG SHA-256: `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`. |
| Native chart visual/interactivity | BLOCKED | `wayland-info` returned `failed to create display: Operation not permitted`; chart arrangement and input were not visually inspected. Existing UI-00 native gate remains open. |

The final build/test chain passed after one corrected test-only type qualification. No IBM application/XSD interoperability claim is made by this chart-layout change.
