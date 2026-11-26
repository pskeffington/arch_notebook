# UI-INGEST-NAV-01: Geodesic navigation and responsive ANX parsing

- **Result:** PASS for graph-layer calculations, Rust tests/build, ANX round-trip/security, current SQL scheduler harnesses, and local launcher synchronization; native GUI interaction and IBM compatibility remain unverified.
- **Task / owner:** Continue Arch Notebook navigation and ingestion parity; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `dff7acfb5a5ffa9c79b692e0e32841821c4f64c8`.
- **Source commit:** `b637d965ae5ed4aeea9ec4300860523dfb95b71b`.
- **Changed paths:** `src/main.rs`, `README.md`, `docs/analyst-notebook-parity.md`, `docs/notebook-workflows.md`, and `docs/research/social-network-graphing-and-delivery-review.md`.
- **Rust source SHA-256:** `ee3eb6c804d4fabfd2962e168e0050de5d55355bc43f4b1013ab6f8067a7bf86`.
- **Release and installed binary SHA-256:** `6c801cf7f307c56a7c4060e48fbeb19f89ba54f9a9f4d19fdf4ff966dd542ae9`.
- **Build target directory:** `/tmp/arch-notebook-target`.

The Analysis sidebar now measures geodesic hop layers from a selected chart entity. Its definition is displayed beside the result: links count as one unweighted, undirected step; disconnected entities and self-links do not add a hop layer, and parallel records do not duplicate a neighbor. Layers sort deterministically by entity name and ID.

Desktop ANX XML parsing now runs on a named background worker. Import controls remain disabled while parsing, and the parsed result is discarded if the active chart changed. Automation/CLI context remains synchronous; the round-trip harness caught and helped correct an early async/CLI regression before this final acceptance.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0 | Source formatting clean. |
| `git diff --check` | PASS, exit 0 | No whitespace errors. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS, exit 0 | 22 passed, 0 failed, 0 ignored. Covers geodesic direction/parallel/self-loop/disconnected semantics and ANX worker error routing. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | PASS, exit 0 | Offline integration check completed. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS, exit 0 | Optimized release build completed. |
| ANX round-trip/security harness | PASS, exit 0 | Export/import retained 8 entities, 8 relationships, and 1 chart label; unsupported shape and DTD external-entity inputs were rejected without modifying destination cases. Fixture SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`; exported ANX SHA-256 `4b5d1d917bbf2c91a173c24346b2619d47836c041408b80b6349c71657265191`. IBM XSD/application validation: NOT PERFORMED. |
| Linux SQL scheduler harness on this release | PASS, exit 0 | Generated service saved one scheduled capture and a one-row CSV; only `WINDOW-START` was included. `systemctl` was stubbed. Report SHA-256 `007e0dc9174382057cd0053225cebd1b41709b83991b2a49de89a2075b2bed57`. |
| macOS runner harness on this release | PASS, exit 0 | Runner simulation selected the same row and confirmed SQLite source unchanged. Temporary `launchctl` stub on Linux; no real macOS dispatch. Report SHA-256 `72b852ef05129189c320819f46cbb2c7def53b39c4f77d3ead47f145c18e69ad`; source database SHA-256 `7c62cccec9459451a0c00da3cbb35265be7bf36f7ddb81c6bf18527c0d1a3071`. |
| Local Linux install and launcher graphics | PASS, scoped | Installed executable hash equals release binary; version command succeeded. Both desktop entries passed `desktop-file-validate` and compare identical. Repository/installed SVG hash `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8`; PNG hash `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`. |
| Remote workflow for source commit | BLOCKED / unverified | GitHub Actions API returned `error connecting to api.github.com`; no remote workflow conclusion is asserted for `b637d96`. |
| Native geodesic/ANX UI interaction | BLOCKED | Wayland access remains denied; no rendered menu, interaction, responsiveness, or visual acceptance is claimed. |

All data used by the automated acceptance harnesses is synthetic. The scheduler tests do not establish live `systemd` or macOS `launchd` dispatch. IBM describes ANX as a standardized XML chart representation and points to the schema/examples inside an installed Analyst's Notebook tree; no compatible product installation was available for this run ([IBM format guidance](https://www.ibm.com/support/pages/node/546669), [IBM import formats](https://docs.i2group.com/anb/10.0.2/import_formats.html)).
