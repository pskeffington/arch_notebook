# SQL-SEARCH-AUTOMATION-DRAFT-01 — seeded search handoff

- **Task / owner:** Make a chart-seeded saved search safely adaptable to recurring SQL automation; primary agent.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-nested-conditions` / `23283c5521de892d127a38bba60ef640fb6dd6cd`.
- **Candidate source commit:** recorded after this evidence commit.
- **Product language:** Rust.
- **Optimized release SHA-256:** `cda40195dd721a2a1013ece65ad10b674519c19acbb19892f47068fbee01fda4`.
- **Installed binary:** `/home/sysadmin/.local/bin/arch-notebook`, matching release.

## Behavior

When a chart-seeded saved search is selected in SQL Review, **Prepare unseeded automation draft** removes only the saved seed parameter value from the working SQL parameter draft. Other typed defaults and the SQL text remain unchanged. The action clears the one-off result preview and labels a prospective job; it does not create a job, execute SQL, schedule a run, change the saved search, or save the case. The UI warns that the retained SQL predicate may still depend on the removed parameter. The analyst must edit or reject that predicate, validate the current unseeded query, and use **Save and review in Automation**. The existing handoff creates a disabled job for explicit review.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Formatting and diff validation | **PASS** | `cargo fmt --all -- --check` and `git diff --check`, exit 0. |
| Workspace check | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline`, exit 0. |
| Full Rust suite | **PASS** | `cargo test --workspace --locked --offline`: 101 passed, 0 failed, 3 ignored. Regression verifies only the seed value is removed, typed parameters and SQL are preserved, stale preview is cleared, no job is created, and the action appears in the expanded SQL Review search catalog. |
| Optimized release build | **PASS** | `TMPDIR=/home/sysadmin/.cache/arch-notebook-build-tmp CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline`, exit 0. Binary contains the new warning/control and SHA-256 is above. |
| Existing saved-search package acceptance | **PASS** | [Aggregate JSON](artifacts/SQL-SEARCH-PACKAGE-03-cli.json), SHA-256 `65a98f9756c02022ed5701552a94c30beebd7fedf44da1957dbac597c8f3f714`. Import/export invariants remain intact. |
| Isolated install acceptance | **PASS** | [Aggregate JSON](artifacts/SQL-SEARCH-PACKAGE-03-install.json), SHA-256 `170ada293c670a213dae7d97f0c6a86b5345c105193137ae39559b369be9245c`; launcher, icons, MIME, helper, and version checks passed. |
| Current-user install | **PASS** | Installed binary matches release digest; current launcher and graphic assets remain synchronized. |
| Clinton SQL regression | **PASS, bounded** | 6,173 relationship rows, zero skipped, zero deltas, read-only inputs unchanged. Human content review, PDF extraction completeness, and Qwen analysis remain unestablished. [Aggregate JSON](artifacts/CLINTON-CORPUS-QA-07-seeded-automation-ui.json), SHA-256 `3ccf5fe8dc6f7e74ed16ddd65c44a43ecaffb5afd88265a8d5cdf7a2127e8438`. |
| Native UI appearance/input | **NOT RUN** | Sandbox cannot connect to X11 `:0`; Wayland access is denied; `/dev/uinput` is absent. |
| Live systemd dispatch | **NOT RUN** | User systemd bus returns `Operation not permitted`; no live schedule was installed or fired. |
| IBM interoperability | **NOT RUN** | No IBM schema or Analyst's Notebook application validation was performed. |

The Clinton artifact is aggregate-only and contains no message names, subjects, bodies, or individual relationship records.
