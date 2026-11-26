# ANX round-trip and DTD rejection acceptance

- **Task / owner:** ANX interoperability hardening / Arch Notebook maintainer
- **Source commit:** `1014ba56e5b02bc16c172b2bfe5492cc89562e39`
- **Scope:** fictional chart round-trip plus malformed-shape and DTD external-entity rejection; no product parser changes in this evidence update.
- **Platform:** Linux x86_64 host; prebuilt release binary at `/tmp/arch-notebook-target/release/arch-notebook`.
- **Fixture:** `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.
- **Command:** `python3 scripts/test-anx-roundtrip-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-anx-xxe-acceptance`
- **Observed exit:** `0`.
- **Result:** **PASS**. ANX export/import passed; 8 entities and 8 relationships were retained, including a repeated relationship ID; unsupported shape import returned expected exit `1`; DTD with a file external entity returned expected exit `1` and the explicit `document types are not supported` error. Both failed imports left their case files unchanged, and the marker contents did not appear in output.
- **ANX artifact SHA-256:** `38f14b200a8a28ca3dc9e065bfc4665677dfbb8042db255690fe5869f631d65f`.
- **IBM validation:** **NOT RUN**. No IBM Analyst's Notebook executable or vendor XSD was available. Internal acceptance does not establish IBM schema or application compatibility.
- **Security rationale:** IBM has published an XML entity injection security bulletin for older Analyst's Notebook releases: <https://www.ibm.com/support/pages/node/738667>. IBM describes ANX as its XML chart interchange format in the [Analyst's Notebook user guide](https://www.ibm.com/docs/en/SSJSV9_9.2.4/pdf/analysts_notebook_pdf.pdf).
- **Changed paths:** `scripts/test-anx-roundtrip-acceptance.py`, `README.md`, and this evidence record.
- **Next action:** run the ANX fixture against IBM's supported schema and application when an authorized product installation is available; separately finish native UI-00 interaction and visual acceptance.
