# UI-ANALYSIS-PACKET-OUTPUT-PRIVACY-01 · packet CLI output paths

- **Task / owner:** Keep successful packet-export summaries free of caller-supplied output paths; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `bc9e33de6d4a5b4ea11ac5acc15b5b1f79874dc6`.
- **Product source:** `ad737f9abc10e62f1c1eb4bc3d1a617104eb18f2`.
- **Verification source:** `03fc79fcf33d` includes a separate viewport-fit test refinement after the packet implementation commit.
- **Changed paths:** `src/main.rs`, `scripts/test-analysis-packet-acceptance.py`, `docs/notebook-workflows.md`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `604e95d768748e8543a91325fdc8917b7aab1dd68770b0a8b0e83ccf58f419fa`.
- **Acceptance report:** [packet output privacy JSON](artifacts/UI-ANALYSIS-PACKET-OUTPUT-PRIVACY-01-linux.json), SHA-256 `128b846bf9ed4a85d20a9e54487f2ced8c29a2de214687f413fdc53d0f56bc0b`.

## Change

Successful `--export-analysis-packet ... --json` output now reports the stable output filenames instead of echoing the caller's destination path. The human-readable success message likewise says files were written to the requested directory without printing it. The acceptance harness asserts that the supplied output path is absent from stdout while preserving the existing disclosure flags, file list, entity/record counts, and source hash. The operator already knows the supplied destination argument.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Rust formatting and patch validation | **PASS** | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Full offline Rust suite | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 61 passed, 0 failed, 1 opt-in benchmark ignored. |
| Locked release build | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline --release` exited 0. |
| Packet acceptance harness | **PASS** | `python3 scripts/test-analysis-packet-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-packet-redacted-output-20261127` exited 0. Its assertions confirm the destination path is absent from both JSON and human-readable stdout, default redaction, selected-chart packet scope, fixture immutability, SVG XML/direction/self-loop markers, no-overwrite and failure handling, and private output permissions. |
| Python harness syntax | **PASS** | `python3 -m py_compile scripts/test-analysis-packet-acceptance.py` exited 0. |
| Exact-head GitHub Actions | **PASS** | [Run 45970460460](https://github.com/pskeffington/arch_notebook/actions/runs/45970460460) completed successfully for `a10536543445f15be083239a7bc2c2572e3e5074`. Linux tests/fixture determinism, Linux x86_64/ARM64, macOS Intel/Apple Silicon, and Arch package jobs passed; release attachment was skipped. |
| Cross-platform publication behavior | **NOT TESTED** | The packet acceptance harness's atomic publication path is Linux-only. No macOS or Windows CLI runtime claim is made here. |

This change limits success-output path exposure. It does not change packet contents, disclosure controls, or file-publication semantics.
