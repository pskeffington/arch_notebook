# ANX-ROUNDTRIP-01 Linux acceptance evidence

- **Task / owner:** Improve supported ANX import/export fidelity; Codex implementation and local verification.
- **Baseline:** `20047ddea0b649df4283b570767c2999866fa43b` on `codex/ui-00-chart-baseline`.
- **Scope:** Preserve supported Arch Notebook chart data across its ANX UTF-16LE export/import path, including repeated connection IDs, local normalized positions, labels, and significant whitespace. Add a deterministic acceptance harness to Linux CI.
- **Changed paths:** `src/main.rs`, `scripts/test-anx-roundtrip-acceptance.py`, `.github/workflows/build.yml`, `docs/sql-ingestion.md`, this evidence file.
- **Fixture:** `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.

## Findings

The initial local round trip failed because two distinct records shared a connection ID and the common import validator rejected the duplicate. The exporter also quantizes positions into ANX chart coordinates; the importer then normalized those coordinates, moving entities and labels within Arch Notebook's normalized canvas. The generic import path additionally trimmed text even when its trim option was disabled.

The importer now assigns deterministic record-level source scopes only to repeated ANX connection IDs, preserving each original ID while keeping the records distinct. Arch Notebook ANX attributes now preserve normalized entity and standalone-label positions and the exact relationship label. ANX import disables whitespace trimming and the generic delimited importer honors that option for endpoint names. UUID metadata is excluded from imported custom attributes.

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| ANX export succeeds and declares UTF-16LE with BOM | PASS | `anx_export`, exit 0; output begins `FF FE`, parses as XML, root is `Chart`, and contains `ChartItemCollection`. |
| ANX import preserves supported entities and positions | PASS | `anx_import`, exit 0; all 8 entity UUIDs, names, types, representations, details, attributes, aliases, external IDs, merged records, lock states, coordinates and colors match. |
| ANX import preserves relationship records and values | PASS | 8 of 8 records retained; endpoints, nonempty connection IDs, labels, dates, confidence, direction, notes, source notes, source references, and custom attributes match. Repeated IDs receive two deterministic record scopes. Anonymous IDs receive generated nonempty IDs. |
| Standalone label round trip | PASS | One label retains UUID, text, normalized position, font size, color, legend state, entries and truncation state. |
| Rust tests, formatter, diff check and debug build | PASS | Commands and exit codes below. |
| IBM schema/application compatibility | NOT RUN | No IBM XSD, Analyst's Notebook, or Chart Reader was available. This test does not establish vendor interoperability. |
| Remote CI for this candidate | PENDING | The CI step has been added; this implementation candidate has not yet been pushed/dispatched. |

## Commands and observed exit codes

From `/home/sysadmin/github/arch_notebook`:

```text
cargo fmt --all
exit 0

git diff --check
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline
exit 0 (12 passed, 0 failed)

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
exit 0

python3 scripts/test-anx-roundtrip-acceptance.py --binary /tmp/arch-notebook-target/debug/arch-notebook --fixture fixtures/chart/correctness.case.json
exit 0 (8 entities, 8 relationships, 1 label)
```

The harness printed its JSON report to stdout; its default temporary directory was removed after success, and no run artifact is retained in the repository. The ANX bytes include a generated timestamp, so the test checks XML structure and semantic round-trip fields rather than byte-for-byte export determinism.

## Limits and next action

This evidence covers Arch Notebook's supported internal subset only. It does not validate IBM XSD conformance, IBM application rendering, unsupported ANX chart-item forms, or external transfer fidelity. Run the new Linux CI gate, then validate representative exported/imported files against an IBM-provided schema and application before claiming IBM interoperability. Preserve the separate native UI-00 gate before starting UI-01.
