# SQL-SEARCH-CATALOG-01 — case-local saved SQL searches

> Follow-up: release build, current-user installation, and Clinton QA later passed at source `7df1bf6`; see [SQL-SEARCH-CATALOG-03](SQL-SEARCH-CATALOG-03-current-source-release-20261128.md). The blocked results below are the original attempt and remain historically accurate.

- **Task / owner:** Add reusable named query definitions to SQL Review; implementation and acceptance by the primary agent.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `8037cbc2d0a94444aebecb9937710e34e52006de`.
- **Candidate source:** working tree based on the baseline; commit recorded after this report is prepared.
- **Scope:** `src/main.rs`, `README.md`, `docs/sql-ingestion.md`, `docs/analyst-notebook-parity.md`, and `docs/build-acceptance.md`.
- **Product source language:** Rust.

## Change

SQL Review can save named query drafts to the current case, including the SQLite path, series, query text, and typed parameter defaults. Analysts can load, update, and delete a definition. Loading restores the editable SQL Review draft and never executes a query, captures a snapshot, or changes an automation job. For chart-seeded review, the bound chart ID value is removed while the seed parameter name is retained for later rebinding. The catalog is separate from scheduled jobs and their package interchange.

Existing cases without the optional `saved_sql_searches` field deserialize with an empty list. The case format version remains 19. IBM parity documentation describes the analogous analyst workflow while distinguishing IBM's administrator-configured i2 Analyze searches from this local SQLite feature.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Saved search model and legacy compatibility | **PASS** | `saved_sql_search_keeps_defaults_but_never_persists_bound_chart_seed_values`; confirms typed defaults round-trip and the old chart fixture loads with an empty catalog. |
| Headless SQL Review workflow | **PASS** | `sql_review_saved_search_ui_loads_query_without_running_or_capturing_it`; saves, loads, updates, deletes, verifies dynamic-seed restoration, and confirms no snapshots were captured. |
| Full Rust test suite | **PASS** | `cargo test --workspace --locked --offline`: 96 passed, 0 failed, 3 ignored. |
| Formatting and source check | **PASS** | `cargo fmt --all -- --check`, `git diff --check`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline` exited 0. |
| Release binary and user install acceptance | **BLOCKED** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline` did not complete. The existing binary remained timestamped `2026-11-28 06:24:22 -0500`, SHA-256 `b90491201384e11ae359fcf4821f8d332fecddd9e1d92f1b0bb477c8f0fa80cd`, so it is not evidence for this candidate. `/tmp` reached 96% usage with 789 MiB free; after clearing only Cargo incremental artifacts it had 1.4 GiB free, but the retry exited without producing a new binary. Install and corpus reruns were therefore not performed. |
| Native UI review | **NOT RUN** | X11/Wayland display is inaccessible in this session. |
| IBM application interoperability | **NOT RUN** | No i2 Analyze connector or IBM server-side query catalog is implemented. |

## Commands

```text
cargo fmt --all -- --check
exit 0

git diff --check
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline
exit 0; 96 passed, 0 failed, 3 ignored

CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline
did not complete; candidate release binary not produced
```

The Clinton corpus remains the primary real-data QA batch, but this saved-search candidate was not exercised by the installed binary. Existing prior Clinton QA evidence applies to the prior installed source only.

## Sources and limits

- [SQL Review usage guide](../../sql-ingestion.md#save-and-reuse-a-sql-search)
- [Analyst Notebook parity notes](../../analyst-notebook-parity.md#reusable-sql-review-searches)
- IBM [External Searches overview](https://docs.i2group.com/anb/10.0.2/i2_connect.html) and [searching external sources](https://docs.i2group.com/anb/9.4.0/search_i2c.html).

No claim is made for native interaction, i2 Analyze connectivity, PDF extraction completeness, Qwen analysis, or human review of email contents.
