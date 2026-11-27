# PACKET-03: Current release finding-packet acceptance

- **Task / owner:** Verify social-network graph results delivery against the current Arch Notebook release binary; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source:** `e27ad5e842dd1c6d503f05c51844f12790502fa6` (current packet and graph product code; later commits through `282f9de3a755557c856b16f50145d9f37936e882` are test and evidence changes).
- **Binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.
- **Fixture:** `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Current release packet harness | PASS | `python3 scripts/test-analysis-packet-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-packet-active-goal-282` exited 0. |
| Source and fixture immutability | PASS | Harness confirmed the input fixture and exported case copy are unchanged. |
| Default privacy and opt-in disclosure | PASS | Default packet omits names, relationship labels, IDs, evidence references, and chart title. The opt-in packet discloses selected fields. |
| Scope, SVG, and fail-closed behavior | PASS | Active/selected chart counts match; SVG parses and retains all 8 records, 5 direction markers, 1 self-loop, and legend. Duplicate JSON options and an existing destination fail without creating or overwriting output. |
| Output permissions | PASS | Harness confirms private Unix directory/file permissions. |
| CLI version | PASS | `arch-notebook --version --json` returned version `0.1.0`. |

## Durable artifact

- Acceptance JSON: [current release packet report](artifacts/PACKET-03-current-branch-20261126.json), SHA-256 `58c2b3aa6033b193ac2a7b08a743bd7f97c04102bda0cc6379736cd10694667f`.

## Limits

This validates local Linux packet generation on a fictional fixture. It does not establish native UI interaction, IBM Analyst's Notebook compatibility, non-Linux publication, or permission to publish results externally. The packet is a redacted finding template and SVG overview; software-generated findings are intentionally absent. Review the privacy implications of topology before sharing.
