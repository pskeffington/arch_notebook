# Arch Notebook project instructions

- Rust native desktop application; persist cases locally in JSON.
- SQL ingestion reads SQLite databases in read-only mode. It must not modify the source database.
- Treat every SQL result as an immutable connection snapshot; compare snapshots before showing regression claims.
- Preserve evidence references, imported record identity, and original merged entity records.
- Keep examples fictional and do not add telemetry or network services.
- Build artifacts belong in `/tmp/arch-notebook-target`.
- Basic checks: `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline`.
