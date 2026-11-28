# SQL-SEARCH-PACKAGE-02 — desktop saved SQL search package workflow

- **Task / owner:** Add saved-search package import/export to SQL Review; primary agent.
- **Repository / branch / source commit:** `arch_notebook` / `codex/ui-00-chart-nested-conditions` / baseline `bdd6c55128b4af30d015ffa952e5c59f2e19f833`; final source commit recorded in this report after commit.
- **Product language:** Rust.
- **Release binary SHA-256:** `0e3f23dcc30da9012c2d8a7cb7d2e5cc46e7539a9f7f60c05fefe0c7eebb54c0`.
- **Installed binary:** `/home/sysadmin/.local/bin/arch-notebook`, same digest.

## Behavior

SQL Review's **Import/export query package** panel imports versioned saved-search definitions and loads them into the review draft without executing SQL, creating a snapshot, or writing the case file. Optional name, database, and series overrides let an analyst adapt a package to the receiving case; blank values preserve package defaults and relative database paths rebase from the exporting case. The selected saved definition can be exported to a JSON package. Case changes remain subject to the normal explicit case-save action.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Formatting and whitespace | **PASS** | `cargo fmt --all -- --check` and `git diff --check`, exit 0. |
| Workspace compile check | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline`, exit 0. |
| Rust unit suite | **PASS** | `cargo test --workspace --locked --offline`: 100 passed, 0 failed, 3 ignored. Includes UI visibility and import/export behavior tests, duplicate-name rejection, path rebasing, no query execution, no snapshot creation, and package round-trip. |
| Optimized release build | **PASS** | `TMPDIR=/home/sysadmin/.cache/arch-notebook-build-tmp CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline`, exit 0; 2m51s. |
| Package CLI black-box regression | **PASS** | `scripts/test-saved-sql-search-package-acceptance.py`, exit 0. Checks path round-trip, chart-seed omission, duplicate import no-write, malformed bound-seed rejection, source overwrite refusal/source unchanged, and import without execution or snapshot changes. [Aggregate JSON](artifacts/SQL-SEARCH-PACKAGE-02-cli.json), SHA-256 `5c4108be5cbbde73cba0a90db53c3300abfd0cb2cc93d25822152d8ca2e373d4`. |
| Isolated user installer | **PASS** | `scripts/test-linux-user-install-acceptance.py`, exit 0. Binary, launcher, menu shortcut, SVG/PNG icons, MIME handler, and helper checks passed. [Aggregate JSON](artifacts/SQL-SEARCH-PACKAGE-02-install.json), SHA-256 `adf0c4c6c41a1220bf387af0a0883fe90cd67844f2df5deef59790a18f2f8871`. |
| Current desktop install | **PASS** | Installed release matches the build; menu and Desktop shortcut hashes match, and repository icon/MIME assets are installed. |
| Clinton corpus regression | **PASS, bounded** | Read-only saved-run preflight/replay: 6,173 relationship rows, zero skipped, zero deltas, unchanged PDF/case/database. Analysis packet covered 2,813 entities with zero labels outside SVG bounds. PDF extraction completeness, Qwen analysis, and human content review were not established. [Aggregate JSON](artifacts/CLINTON-CORPUS-QA-06-ui-package-current.json), SHA-256 `9f2bfe3047570d69305d4c010d07e2fdba45e4e871dddd43be104b460c27f4bc`. |
| Native pointer/keyboard and visual review | **NOT RUN** | Headless UI checks do not establish native input behavior or rendered appearance. |
| IBM i2 Analyze / Analyst's Notebook interoperability | **NOT RUN** | No IBM endpoint, schema validation, or application validation was used. |

Reports contain aggregate counts and hashes only; they exclude email names, subjects, bodies, and individual relationship rows.
