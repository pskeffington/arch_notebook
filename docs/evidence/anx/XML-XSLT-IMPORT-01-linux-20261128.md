# XML/XSLT import adaptation — 2026-11-28

- **Task:** add a scriptable, SQL-pipeline-friendly XML adaptation route into Arch Notebook.
- **Owner:** Codex implementation; single-session review (not independent).
- **Baseline:** `14cc2d84d245e06929d817027249fe6c68bccc0a` on `codex/ui-00-chart-nested-conditions`.
- **Scope:** optional `xsltproc` CLI transformation into the existing strict ANX importer; documentation and a synthetic acceptance harness.
- **Changed paths:** `src/main.rs`, `scripts/test-xml-xslt-import-acceptance.py`, `docs/analyst-notebook-parity.md`, `README.md`, this evidence record, and `docs/build-acceptance.md`.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --all -- --check && git diff --check` | PASS, exit 0 | Final formatted source and patch whitespace checks passed. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS, exit 0 | 81 passed, 0 failed, 3 ignored. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | PASS, exit 0 | Offline locked check passed. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | PASS, exit 0 | Debug CLI built; SHA-256 `c463117b92e774dfd2d7e019db4fd026bee04d473b91aadafa2c479a75a62748`. |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS, exit 0 | Optimized CLI built in 3m27s; SHA-256 `f5ff73d6676c565499a39b879dbb245a0e5813acbe217ec1430550ee60a39196`. |
| `python3 scripts/test-xml-xslt-import-acceptance.py --binary /tmp/arch-notebook-target/debug/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-xslt-acceptance-20261128-v3` | PASS, exit 0 | Transformed/imported the fictional 8-entity/8-relationship chart; UTF-8 and UTF-16 DTD source declarations and stylesheet DTD declaration rejected; destination remained byte-identical after failures. Fixture SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`. |
| IBM schema/application validation | NOT TESTED | No IBM ANX XSD or running IBM application was used. |
| Live SQL automation invoking this CLI | NOT TESTED | The feature adds an XML adaptation/import command; no schedule or SQL job was configured to invoke it. |

## Behavior and limits

The same acceptance harness passed against both the debug and release binaries. Release command: `python3 scripts/test-xml-xslt-import-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-xslt-acceptance-20261128-release`.

Command: `arch-notebook --import-xml-xslt <case.json> <input.xml> <stylesheet.xsl> [--json]`.

The CLI requires `xsltproc` on `PATH`, rejects DTD declarations in both source files (including UTF-16), invokes the transform with network fetches and XSLT writes disabled, limits each source and the captured transformed document to 16 MiB, then uses the existing ANX parser and atomic case save. A failed transform or ANX validation leaves the case unchanged. The stylesheet is executable transformation logic and must be trusted. This test uses an identity stylesheet over Arch-generated ANX, so it verifies the adaptation pipeline and failure boundaries, not compatibility with IBM's schema, IBM-generated XML, or an organization's real feed. `xsltproc` output is captured before the size check; trusted stylesheets remain a prerequisite for controlling CPU and peak memory use.

## Next action

Validate a real, approved XML-to-ANX stylesheet and representative output against the schema installed with the target IBM Analyst's Notebook version, then open the result in IBM's application. Connect this command to a SQL automation job only when an operator has a concrete XML source, trusted stylesheet, and desired schedule.
