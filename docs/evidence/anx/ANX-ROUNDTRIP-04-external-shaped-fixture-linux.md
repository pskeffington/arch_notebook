# ANX-ROUNDTRIP-04 — external-shaped XML import semantics

- **Task / owner:** Improve internal ANX interoperability coverage without claiming vendor conformance; implementation session.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-baseline` / `1370e2172473dffbd1f4923d424bda6909292a25`.
- **Candidate source:** recorded after commit.
- **Changed paths:** `scripts/test-anx-roundtrip-acceptance.py`, this evidence file, `docs/sql-ingestion.md`, `docs/analyst-notebook-parity.md`, `docs/build-acceptance.md`.

## Change

Extended the existing ANX acceptance harness with a small UTF-8 XML input that uses named Strength and LinkType references, an `ArrowOnTail` link, a chart-item date, and a custom relationship attribute. The harness checks the resulting endpoint reversal and directed flag, maps the relationship type and Strong strength to the expected Arch Notebook values, normalizes the date, and preserves the custom attribute. Existing round-trip, unsupported-shape rejection, DTD rejection, and atomicity checks remain in the same harness.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| ANX acceptance harness | PASS | `python3 scripts/test-anx-roundtrip-acceptance.py --binary /tmp/arch-notebook-target/debug/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-anx-interoperability-followup-2` exited 0. Report: `result=PASS`, `external_shape_reverse_arrow=true`, `external_shape_strength_type_date_attribute=true`, unsupported-shape and DTD rejection pass. |
| Production importer | PASS, scoped | The test invokes the actual `--import-anx` CLI against generated temporary XML and checks the saved case. |
| IBM XSD/application conformance | NOT TESTED | The input is a bounded synthetic fixture, not a vendor fixture. No IBM XSD or Analyst's Notebook/Chart Reader was available. |

This acceptance improves our coverage of the supported subset. It does not claim that the fixture validates against IBM's schema or reproduces every IBM export variant.
