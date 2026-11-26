# UI-00 deterministic fixture reproduction

- **Task / owner:** Verify the UI-00 fictional chart generator, checked-in correctness fixture, workload counts, and cap; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source commit:** `4fda60ad5d98caa198364ab1c4c40da8af8d1101`.
- **Date / host:** 2026-11-26, Linux x86_64; Python 3 standard library.
- **Generator:** `scripts/generate-chart-fixtures.py`; manifest SHA-256 `a75214f2e20b7b5724e301c643f24276d5bfc9857aed890ba4212baaa9e4fcec`.
- **Small fixture:** `fixtures/chart/correctness.case.json`; SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| Correctness fixture determinism | **PASS** | Generated twice to `/tmp/arch-notebook-ui00-fixtures-20261126-01/correctness-{a,b}.case.json`; `cmp` passed for both generated copies and against the checked-in fixture; `python3 -m json.tool` exited 0. Both generator summaries report the same fixture SHA above. |
| Four 1,000-node workload families | **PASS** | Sparse 3,000 links; hub 3,000; disconnected 2,000; dense 10,000. Each family was generated twice and byte-compared; both runs per family produced identical SHA-256 values. |
| Dense cap at 10,000 nodes | **PASS** | Two deterministic outputs each contained 10,000 entities and exactly 100,000 links, equal to the manifest cap. The files are in the same `/tmp` work directory. |
| Summary and artifact record | **PASS** | 12 JSON summary lines retained at `/tmp/arch-notebook-ui00-fixtures-20261126-01/generator-summary.jsonl`; SHA-256 `55cbb2a1a00fec0d84fa47ba0c466a3b8bada41f1b803ba4b82891826685f247`. |
| Native baseline action sequence and timings | **BLOCKED** | The current Wayland and X11 endpoints reject display access; this generator check supplies no rendering or interaction measurements. See [current-branch UI and SQL acceptance](../sql-automation/SQL-AUTO-03-current-branch-linux.md). |
| Current-release window launch | **BLOCKED** | `timeout 10s env -u WAYLAND_DISPLAY XDG_DATA_HOME=/tmp/arch-notebook-ui00-xdg /tmp/arch-notebook-target/release/arch-notebook fixtures/chart/correctness.case.json` exited 1 with `XNotSupported(XOpenDisplayFailed)`. The release binary SHA-256 is recorded in the linked SQL-AUTO-03 report. |
| PR CI execution for this candidate | **PENDING** | The current feature branch has no implementation PR. The CI workflow contains the test and deterministic-generator steps, but no PR-triggered run for `4fda60a` was available; local results are not reported as CI. |

## Reproducibility details

The exact workload command shape was `python3 scripts/generate-chart-fixtures.py --family <family> --nodes <count> --output <path>`. Correctness used `--family correctness` without `--nodes`; sparse, hub, disconnected and dense each used 1,000 nodes; dense was also run twice at 10,000 nodes. Each paired JSON file was checked with `cmp`. The summary reports the fixed UUID5 seed and counts; generated workloads stayed outside Git.

The initial local count-aggregation assertion failed because the verifier expected family summaries in the wrong order (it placed disconnected before hub). No generator or artifact comparison failed. After correcting the expected order to match the generator's declared sequence, all 12 summaries passed the node/link count assertions and the 100,000-link cap check. The corrected command exited 0.

This closes local fixture reproducibility and count checks only. It does not measure native chart frame time, input response, picking latency, user task success, Mac behavior, IBM visual parity, or release readiness.
