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
- Next pass: native host acceptance using fictional fixtures. Record platform, commit, commands, expected and observed results for capture/comparison, schema drift, chart CSV/TSV round-trip and scheduled execution. Keep build evidence separate from interactive acceptance.
- Validate IBM i2 CSV/TSV handoff with an actual receiving installation before claiming interoperability acceptance. Native ANX remains unimplemented; obtain vendor schema and fixtures first.
- Preserve read-only source access, immutable captures, stable IDs and atomic writes. Do not publish a release or imply notarization without explicit approval and evidence.
- Refresh README.md and RELEASING.md when acceptance or release evidence changes.

## Email analysis follow-on — 2026-11-25

The November 25 development record reports a successful end-to-end TRANS → Arch Notebook test using a Clinton email PDF dump, linking actors through correspondence and displaying the resulting graph. Prioritize the [acceptance evidence package](docs/email-analysis-acceptance.md), then GUI layout and vector rendering refinements. Preserve working ingestion and relationship semantics; use fictional fixtures for regression tests. Do not mark exact-revision runtime acceptance passed until artifacts are reviewed.
