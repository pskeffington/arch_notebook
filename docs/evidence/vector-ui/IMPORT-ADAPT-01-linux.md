# Delimited import adaptation regression evidence

- **Task / owner:** Resume the Rust Arch Notebook build with focused acceptance for analyst-adaptable delimited ingestion; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source baseline:** `3530ce9e773ed432de53ef096d18207f83e93565`.
- **Scope:** Test-only coverage of configured import transforms, transform ordering, row-filter operators, and transform configuration validation. No product behavior changed.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Transform applied before row filter | PASS | A value mapped by exact replacement is accepted by a filter on its transformed value and rejected by the old value. |
| Ordered transform pipeline | PASS | Replacement followed by uppercase produces the expected deterministic value. |
| Filter operators | PASS | Equality, substring, empty, and non-empty behavior are covered, including whitespace-only values for `is empty`. |
| Invalid transform configuration | PASS | Missing and unknown columns and replacement without a search string are rejected; a blank editor row remains ignored. |
| Focused Rust tests | PASS | `cargo test --offline delimited_import -- --nocapture`: 3 passed, 0 failed. |
| Full Rust test suite | PASS | `cargo test --offline`: 34 passed, 0 failed. |
| Formatting and whitespace | PASS | `cargo fmt --all`; `git diff --check` exited 0. |
| Interactive import preview / IBM import specifications | BLOCKED / not tested | No native pointer-driven review or IBM `.ximp`/`.oimp` validation was performed. These are Arch Notebook CSV/TSV design semantics. |

This is a headless regression for the pure transform/filter path used by the importer. It does not establish native UI interaction or IBM compatibility.
