# PACKET-05: Redact destination paths from packet-export responses

- **Task / owner:** Remove caller-supplied output paths from packet-export success messages; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source baseline:** `bc9e33de6d4a5b4ea11ac5acc15b5b1f79874dc6`.
- **Scope:** `src/main.rs`, `scripts/test-analysis-packet-acceptance.py`, `docs/notebook-workflows.md`, and the vector/community roadmap.
- **Security review:** Independent, read-only security review found that success output echoed the destination path; reviewer follow-up reports **PASS** after correction. The reviewer did not execute tests.

## Change

JSON success output now lists only `finding.md` and `network.svg` in `output_files`. Human-readable output says the files were written to the requested directory without printing its path. The Linux packet acceptance script checks both modes for absence of the destination path. Workflow documentation describes the JSON response change.

## Verification

All commands ran from the repository root against the fictional `fixtures/chart/correctness.case.json` fixture. Each exited 0:

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | **PASS** |
| `git diff --check` | **PASS** |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, 61 passed, 0 failed, 1 ignored |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | **PASS** |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | **PASS** |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | **PASS** |
| `python3 scripts/test-analysis-packet-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-security-privacy-02` | **PASS** |

Release candidate SHA-256: `604e95d768748e8543a91325fdc8917b7aab1dd68770b0a8b0e83ccf58f419fa`.

The packet harness verified JSON and human output omit the supplied destination path, default redaction, explicit disclosure behavior, SVG XML and direction/self-loop markers, unchanged input fixture and case copy, refusal to overwrite an existing destination, duplicate-option rejection, and private Linux output permissions. Fixture SHA-256: `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.

## Limits and status

This is local Linux CLI evidence on fictional data. Small or distinctive graph topology can still identify a network; explicit disclosure flags still opt in named fields. This does not establish native GUI acceptance, external sharing approval, or release readiness. The operator's intermittently running review screen showed stacked relationship labels and entity cards clipped at viewport edges. The screenshot does not show whether the clipping came from pan/zoom, and this patch does not resolve or validate that visual behavior. Native input-capable review after Fit all remains **OPEN**.
