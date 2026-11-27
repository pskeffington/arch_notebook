# PACKET-04: Community-aware analyst finding packet

- **Task / owner:** Extend local findings delivery with graph community context; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source baseline:** `179aff68d32e59e49f411a18f17e90144903712d`; final source commit is recorded in Git after evidence capture.
- **Changed paths:** `src/main.rs`, `scripts/test-analysis-packet-acceptance.py`, `docs/notebook-workflows.md`, this evidence record, its JSON artifact, and `docs/roadmaps/social-network-graphing-roadmap.md`. Other pre-existing dirty paths were preserved and excluded.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `4bc1d5b72d623a32293338b459633c3d38c1726db01edd958b136c50e635e4d1`.
- **Fixture:** `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.

## Delivered behavior

`finding.md` now includes detected community membership, cross-community relationship-record totals, distinct endpoint-pair totals, record direction counts, and a structural proxy table. Members use the SVG's `Entity NNN` pseudonyms by default; existing explicit name and ID disclosure switches control their corresponding fields. The report states the deterministic undirected, unweighted simple-graph assumptions and warns that detected communities are not real-world affiliations. The proxy ranking's last key and displayed count are explicitly all incident records, including within-community records. Opt-in entity labels escape Markdown table pipes.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --all` and `git diff --check` | **PASS** | Both exited 0 on the candidate. |
| Focused community packet regression | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline analysis_packet_reports_detected_communities_and_cross_group_records -- --test-threads=1` exited 0. |
| Full offline Rust suite | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 58 passed, 0 failed. |
| Release build | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0. |
| Linux packet acceptance | **PASS** | `python3 scripts/test-analysis-packet-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-packet-community-20261127-03` exited 0. |
| Independent community-packet review | **PASS** | Read-only review found and the implementation corrected ambiguous proxy-record semantics and Markdown pipe escaping; reviewer then confirmed both fixes with no residual finding. The reviewer did not run tests. |

Acceptance JSON: [PACKET-04 report](artifacts/PACKET-04-community-summary-20261127.json), SHA-256 `57eec9a83c4bdf80b79d90dc93c7bdb2f2e9d8a05cb095adf3d530eeea869d3e`.

Acceptance JSON: [PACKET-04 report](artifacts/PACKET-04-community-summary-20261127.json), SHA-256 `5140f6c6a36bda7107735950e2cee89fc2afd6f49fa135b8ccc14bac2ae949f5`. The acceptance harness confirms case-copy and fixture immutability, default redaction and explicit disclosure, active/selected chart scope, SVG XML validity and all 8 relationship records (5 directed markers, 1 self-loop), duplicate-option rejection, refusal to overwrite an existing destination, and restrictive Linux output permissions. The Rust fixture additionally checks three detected communities, cross-community record/pair/direction counts, proxy ranking semantics, caveats, private names/IDs, and escaping of a pipe-containing disclosed proxy name.

## Limits and remaining gates

This validates local Linux CLI output on fictional data. It does not establish real-world affiliation, Analyst's Notebook compatibility, non-Linux atomic publication, native UI acceptance, external sharing approval, or release readiness. Community analysis is structural and ignores relationship type and date. Native G3 interaction remains **BLOCKED** by unavailable desktop input; this work does not close the broader graphing goal.
