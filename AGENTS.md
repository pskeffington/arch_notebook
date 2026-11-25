# Arch Notebook project instructions

- Rust native desktop application; persist cases locally in JSON.
- SQL ingestion reads SQLite databases in read-only mode. It must not modify the source database.
- Treat every SQL result as an immutable connection snapshot; compare snapshots before showing regression claims.
- Keep automation jobs and their bounded run history in the case file; command-line runs must save results atomically and retain failure records.
- Preserve evidence references, imported record identity, and original merged entity records.
- Keep examples fictional and do not add telemetry or network services.
- Build artifacts belong in `/tmp/arch-notebook-target`.
- Basic checks: `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline`.
- Keep `.github/workflows/build.yml`, `packaging/arch/PKGBUILD`, `packaging/arch/arch-notebook.desktop`, and `scripts/package-macos.sh` aligned with supported Linux/macOS targets and package dependencies.
- Keep `README.md` and `RELEASING.md` consistent with the release workflow. Do not publish a GitHub Release without explicit owner approval.

## Current engineering handoff (2026-11-25)

- Read the development status in README.md before selecting work; baseline 2dadb09 passed all five platform build/package jobs (Actions run 45269810987).
- UI-00 source work is recorded on `codex/ui-00-chart-baseline`; complete its native Linux baseline when a usable display is available before advancing to UI-01. Keep build evidence separate from interactive acceptance.
- Native host acceptance with fictional fixtures must cover SQL capture/comparison, schema drift, chart CSV/TSV round-trip, scheduled execution, and chart navigation. Validate CSV/TSV and supported ANX XML against IBM's schema/application before claiming interoperability acceptance.
- Preserve read-only source access, immutable captures, stable IDs and atomic writes. Do not publish a release or imply notarization without explicit approval and evidence.
- Refresh README.md and RELEASING.md when acceptance or release evidence changes.
