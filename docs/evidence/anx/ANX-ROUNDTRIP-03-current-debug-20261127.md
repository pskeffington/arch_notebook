# ANX-ROUNDTRIP-03 — current-source internal acceptance

- **Task / owner:** Revalidate supported ANX exchange behavior on the current-source binary; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source:** `ab6e29318fca406d748f6694c89b3e25cd06b347`; subsequent commits only update evidence/documentation.
- **Candidate:** `/tmp/arch-notebook-current-target/debug/arch-notebook`, SHA-256 `74c900167f450e28f391506d043bfb17fe177382afe19d6f8fdd16bcab24e606`.
- **Fixture:** fictional `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.
- **ANX output SHA-256:** `14b30e137b84112ce1f682b0735c03a732bccc862b19794b8daeb1af3b0fd2d3` (export contains a generated timestamp).

## Acceptance

| Check | Result | Evidence |
| --- | --- | --- |
| UTF-16LE ANX export and import | PASS | Both exited 0; 8 entities, 8 relationships, one label. |
| Supported record identity | PASS | Repeated connection ID retained as a distinct record; local SQL source keys were omitted. |
| Unsupported shape and DTD/external entity rejection | PASS | Both negative imports exited 1; the candidate case remained unchanged. |
| IBM XSD/application validation | NOT TESTED | No IBM application or schema was found in the local TRANSCRIBE, user share, `/opt`, or `/usr/local/share` search paths. IBM's support note points to the XSD/examples in an installed Analyst's Notebook tree: [IBM ANX format support note](https://www.ibm.com/support/pages/node/546669). |

This verifies Arch Notebook's documented supported subset and import-safety behavior. It does not establish IBM XSD conformance or IBM application rendering.

## Reproduction

```sh
python3 scripts/test-anx-roundtrip-acceptance.py \
  --binary /tmp/arch-notebook-current-target/debug/arch-notebook \
  --fixture fixtures/chart/correctness.case.json
```

The harness exited 0 and removed its temporary work directory after completion. The full exact-head CI run also exercised ANX round-trip acceptance: [run 46059559135](https://github.com/pskeffington/arch_notebook/actions/runs/46059559135).
