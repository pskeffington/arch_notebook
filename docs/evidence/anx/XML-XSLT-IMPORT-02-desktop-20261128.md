# Desktop XML/XSLT import adaptation — 2026-11-28

- **Task:** expose XML-to-ANX transformation in the desktop Import menu without blocking chart interaction.
- **Owner:** Codex implementation; single-session review (not independent).
- **Baseline:** `6dcef3f511d067167027ecf7ab65133a326ebd5a`.
- **Scope:** stylesheet path control, asynchronous transform/parse worker, chart-switch guard, shared safe transform implementation for desktop and CLI, regression and acceptance evidence.
- **Changed paths:** `src/main.rs`, `README.md`, `docs/analyst-notebook-parity.md`, `docs/build-acceptance.md`, this evidence record.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --all && git diff --check` | PASS, exit 0 | Final source formatting and whitespace checks passed. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS, exit 0 | 82 passed, 0 failed, 3 ignored. Includes worker source-validation error routing to the captured target chart. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | PASS, exit 0 | Locked offline check passed. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | PASS, exit 0 | Final debug binary SHA-256 `5013c713fdac357dd2bdb691a1b1b62b4ee9edd9ed1a07a426efb8651b6b4cba`. |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS, exit 0 | Final optimized build completed in 2m52s; release binary SHA-256 `4f37a78a91230b8c1aeafc16d0d3aedc18638dd83bb56eaf2172d504cf24c08d`. |
| XML/XSLT acceptance on debug and release | PASS, exit 0 | Both passed identity transform and strict ANX import (8 entities, 8 relationships), rejected UTF-8/UTF-16 source DTDs and stylesheet DTD, and preserved the case after rejected imports. Fixture SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`. |
| Worker error routing regression | PASS, included in Rust suite | Background worker returns the original chart ID and a DTD validation error; test does not require `xsltproc`. |
| Native desktop pointer/keyboard interaction | NOT TESTED | The Import menu was not driven in a native window in this check. |
| IBM schema/application interoperability | NOT TESTED | No IBM XSD or running IBM product was used. |

The desktop **Import…** menu now accepts an XML input path plus a trusted XSLT stylesheet path. It runs transformation and ANX parsing on a background worker, discards the parsed payload if the active chart changes before completion, then applies it through the same staged import path as ANX. The CLI and desktop share DTD checks, source-size bounds, network/write restrictions, and private temporary-file creation. Stylesheets remain trusted executable transformations; the output cap is checked after `xsltproc` exits, so controlling CPU and peak memory still depends on the stylesheet source.

## Next action

Exercise the desktop menu with native input and a real organization-owned stylesheet, then validate its output against the matching IBM schema and open it in the licensed IBM application. SQL scheduling can invoke the CLI after an operator confirms the source and stylesheet; this feature does not configure a SQL job or schedule automatically.
