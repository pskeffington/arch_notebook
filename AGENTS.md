# Arch Notebook project instructions

- Rust native desktop application; persist cases locally in JSON.
- SQL ingestion reads SQLite databases in read-only mode. It must not modify the source database.
- Treat every SQL result as an immutable connection snapshot; compare snapshots before showing regression claims.
- Keep automation jobs and their bounded run history in the case file; command-line runs must save results atomically and retain failure records.
- Preserve evidence references, imported record identity, and original merged entity records.
- Keep examples fictional and do not add telemetry or network services.
- Build artifacts belong in `/tmp/arch-notebook-target`.
- Basic checks: `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline`.
- Run `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` for implementation passes; the current workflow does not execute tests until the UI-00 CI task is implemented.
- Keep `.github/workflows/build.yml`, `packaging/arch/PKGBUILD`, `packaging/arch/arch-notebook.desktop`, and `scripts/package-macos.sh` aligned with supported Linux/macOS targets and package dependencies.
- Keep `README.md` and `RELEASING.md` consistent with the release workflow. Do not publish a GitHub Release without explicit owner approval.

## Current engineering handoff (2026-11-25)

- Immediate shared-code priority: read [CODEX_NEXT.md](CODEX_NEXT.md), [ROADMAP.md](ROADMAP.md) and [the vector UI acceptance protocol](docs/vector-ui-acceptance.md), then implement UI-00: reproducible fictional fixtures, baseline measurements, meaningful chart regression tests and an executing CI test gate. Continue UI-01 through UI-03 after their prerequisite gates pass; UI-05 records integrated native acceptance. UI-04 is conditional.
- Linux Operations owns shared chart implementation. Foundation means Mac Operations and owns Mac evidence plus parallel R1 source verification. Use one writer for `src/main.rs`, separate feature worktrees/branches and exact-SHA handoffs. Do not concurrently rewrite the monolithic application from both seats.
- Preserve the current egui/eframe 0.31.1 and Glow baseline, branding, normalized entity-position serialization and existing layouts/navigation. Add conservative CPU culling to the existing painter clipping; share tested geometry between rendering and picking. Invalidate cached link indices when graph structure changes; `connection_id` alone is not globally unique.
- Read the development status in README.md before selecting work; baseline 2dadb09 passed all five platform build/package jobs (Actions run 45269810987).
- Retain native host acceptance using fictional fixtures alongside the vector UI queue. Record platform, commit, commands, expected and observed results for capture/comparison, schema drift, chart CSV/TSV round-trip and scheduled execution. UI-00 records the initial available-host smoke/baseline; UI-05 reconciles Linux and Mac results. Keep build evidence separate from interactive acceptance.
- Validate IBM i2 CSV/TSV handoff with an actual receiving installation before claiming interoperability acceptance. Native ANX remains unimplemented; obtain vendor schema and fixtures first.
- Preserve read-only source access, immutable captures, stable IDs and atomic writes. Do not publish a release or imply notarization without explicit approval and evidence.
- Refresh README.md and RELEASING.md when acceptance or release evidence changes.
