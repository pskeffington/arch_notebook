# SQL-SEARCH-PACKAGE-01 — portable saved SQL search definitions

- **Task / owner:** Add definition-only saved SQL search package interchange and validate runtime; primary agent.
- **Repository / branch / source commit:** `arch_notebook` / `codex/ui-00-chart-nested-conditions` / source `1ca420391bc14d06e9aa0fd54c7bc2b778c0e053`.
- **Product language:** Rust.
- **Format:** JSON `arch-notebook-sql-search`, schema version 1.
- **Release binary SHA-256:** `5872cb68ebebca61961397b566fab9944ca7f958210e5a1a5b8d8e29fc6cf135`.
- **Installed binary:** `/home/sysadmin/.local/bin/arch-notebook`, same digest.

## Behavior

`--export-sql-search <case> <name-or-id> <output.json> [--json]` writes a portable definition package. `--import-sql-search <case> <input.json> [--name <name>] [--database <path>] [--series <label>] [--json]` adds it to another case. Relative paths rebase from the exporting case directory unless `--database` is provided. Import assigns a new case-local ID. Import never connects to SQLite, runs the query, creates a snapshot, or creates an Automation job. Packages exclude entities, charts, chart IDs, query results, history, and chart-bound seed values; they retain only the seed parameter name. Duplicate names, malformed definitions, seed-value leakage, unsupported versions, and exports that would replace the case/lock/source database are rejected.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Unit regressions | **PASS** | `saved_sql_search_package_round_trips_rebased_path_without_case_data` and `saved_sql_search_package_rejects_bound_seed_values_and_source_overwrite`. |
| Full Rust suite | **PASS** | 99 passed, 0 failed, 3 ignored. |
| Formatting and workspace check | **PASS** | `cargo fmt --all -- --check`, `git diff --check`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline` exited 0. |
| Release build | **PASS** | `TMPDIR=/home/sysadmin/.cache/arch-notebook-build-tmp CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline`, exit 0; 2m55s. |
| Black-box package CLI acceptance | **PASS** | `python3 scripts/test-saved-sql-search-package-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report /tmp/SQL-SEARCH-PACKAGE-01.json`, exit 0. Includes path override, seed omission, duplicate import no-write, malicious bound-seed rejection, source overwrite protection/source unchanged, no execution, unchanged snapshot count. |
| Isolated Linux install acceptance | **PASS** | `scripts/test-linux-user-install-acceptance.py` with the release binary, exit 0. |
| Current-user install | **PASS** | Updated `~/.local/bin/arch-notebook`; digest matches release. Desktop shortcut equals menu entry; SVG/PNG and MIME definition match repository assets. |
| Clinton primary corpus QA | **PASS** | `scripts/test-clinton-corpus-acceptance.py` with the release binary, exit 0; read-only preflight/replay passed, 6,173 relationship rows, zero skipped rows and zero deltas; source inputs unchanged. |
| Native UI / IBM interop | **NOT RUN** | No native pointer review and no i2 Analyze connection/schema validation. |

## Aggregate artifacts

- CLI package acceptance: [SQL-SEARCH-PACKAGE-01-cli.json](artifacts/SQL-SEARCH-PACKAGE-01-cli.json), SHA-256 `79250ba114d4c1b6fbddb514e931e11426f607815ec6144d01f7fb7c7d505353`.
- Isolated installer acceptance: [SQL-SEARCH-PACKAGE-01-install.json](artifacts/SQL-SEARCH-PACKAGE-01-install.json), SHA-256 `9dac28904fe3848306ec8eefa05a12807136e9c1fb237e1c2254cf6369fd9fd3`.
- Clinton corpus acceptance: [SQL-SEARCH-PACKAGE-01-clinton.json](artifacts/SQL-SEARCH-PACKAGE-01-clinton.json), SHA-256 `715f3072f1e7ae15f4abfc8f023e63bdda35a4ec7623052253aae9f66247dd39`.

Reports include aggregate counts and file hashes only, not email names, bodies, subjects, or individual relationship rows.
