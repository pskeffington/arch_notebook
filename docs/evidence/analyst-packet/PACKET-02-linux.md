# PACKET-02: Linux packet fixes and acceptance

- **Task/owner:** Fix and verify shortest-path handling of missing endpoints and finding-packet SVG/scope defects; implementation by `/root/shortest_path_missing_endpoints_fix`, integration by `/root`.
- **Source baseline:** `91ebd699b49b8438e168a126e47004db87b908c5` on `codex/ui-00-chart-baseline`; evidence applies to the scoped working diff described below.
- **Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `docs/roadmaps/social-network-graphing-roadmap.md`, and `docs/roadmaps/vector-community-exploration-roadmap.md`; this evidence note and its acceptance report.
- **Scope:** ignore relationships with missing chart endpoints during shortest-path traversal; route reversed/high-multiplicity packet links in unique lanes; disclose that standalone chart labels are omitted; plan detected-community and vector optimization gates.

## Verification results

| Check | Command | Result |
| --- | --- | --- |
| Formatting | `cargo fmt --all -- --check` | **PASS**, exit 0 |
| Diff whitespace | `git diff --check` | **PASS**, exit 0 |
| Full offline unit suite | `CARGO_TARGET_DIR=/tmp/arch-notebook-target-integration cargo test --locked --offline -- --test-threads=1` | **PASS**, exit 0; 36 passed, 0 failed |
| Offline check | `CARGO_TARGET_DIR=/tmp/arch-notebook-target-integration cargo check --locked --offline` | **PASS**, exit 0 |
| Offline debug build | `CARGO_TARGET_DIR=/tmp/arch-notebook-target-integration cargo build --locked --offline` | **PASS**, exit 0 |
| Offline release build | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | **PASS**, exit 0 |
| Packet acceptance | `python3 scripts/test-analysis-packet-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-packet-acceptance-20261126-current` | **PASS**, exit 0 |
| Independent static review | Review of current scoped diff by `method_card_independent_review` | **PASS**, no residual findings; reviewer did not rerun tests |

The Rust suite includes `shortest_path_ignores_relationships_with_missing_endpoints`, which rejects a phantom path through absent IDs and retains a valid direct path. It also includes `analysis_packet_routes_reversed_and_high_multiplicity_records_distinctly`, which exercises 13 mixed-direction records over reversed endpoints and verifies 13 unique lane geometries.

The acceptance harness confirmed the source fixture was unchanged, redaction defaults and disclosure flags, selected-chart counts, duplicate-input rejection, refusal to overwrite an existing output, valid SVG with direction/self-loop markers, and private Unix output permissions.

## Durable artifacts

- Acceptance JSON: [PACKET-02 acceptance report](artifacts/PACKET-02-acceptance-report.json), SHA-256 `d38491df9293670aef49d38250babf33725fd59476ce643bf31e4782aff7782d`.
- Release binary used by harness: `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `261e6cf830bf06f6a6efc9f58791d4836b71859e4d9b5228f64e711b6672646d`.
- Fixture: `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.

## Review status and limits

- The earlier independent packet security review passed at `991c476`; this change did not receive a separate security review. The corrections affect SVG geometry and disclosure of omitted standalone labels; no privacy option or publication mechanism was changed.
- Native chart/card interaction remains **BLOCKED** because this session cannot access an input-capable desktop display. Source/build/CLI acceptance does not establish native GUI visual or interaction behavior.
- Packet directory publication remains Linux-only; non-Linux publication is explicitly unsupported.
- SNG-DELIVERY-002 community detection, vector layout optimization, and rendering optimization are planned but not implemented. See [vector and community roadmap](../../roadmaps/vector-community-exploration-roadmap.md).
- No release or deployment approval is implied.
