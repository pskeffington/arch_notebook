# SQL-SEARCH-CATALOG-02 — current-source runtime checks

> Superseded for release installation and Clinton acceptance by [SQL-SEARCH-CATALOG-03](SQL-SEARCH-CATALOG-03-current-source-release-20261128.md), which uses the optimized release binary at the same source commit. This record remains the debug-profile check.

- **Task / owner:** Verify the saved SQL search and Quick Open source against the existing Linux install contract and primary Clinton corpus; primary agent.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Source commit:** `a1f1c0d7af4c6304a667516218ab3128373adac5`.
- **Binary profile:** locked offline debug build, not release.
- **Binary SHA-256:** `8f9f1d2da5e5bfedd54eca4bbafeb13f355cb249161b9422f5952cbb0f864c62`.
- **Changed product source:** `src/main.rs` includes case-local saved SQL search catalog and Quick Open actions.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Locked offline debug build | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --workspace --locked --offline`, exit 0. |
| Isolated Linux per-user installer acceptance | **PASS** | `scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/debug/arch-notebook --report /tmp/SQL-SEARCH-CATALOG-02-debug-install.json`, exit 0. Installer, menu/Desktop match, mode, desktop file, installed-binary identity, SVG/PNG icons, SQL helper, MIME definition, and version checks passed in an isolated temporary home. This does not install the debug binary onto the user's desktop. |
| Primary Clinton corpus QA | **PASS** | `scripts/test-clinton-corpus-acceptance.py` with the debug binary and local PDF/case/database, exit 0. One query-only job passed; saved run replay passed with 6,173 relationship rows, zero skipped rows, and zero added/changed/removed deltas. Two snapshots and two saved SQL runs remain recorded in the case. Input hashes were checked unchanged. |
| Analysis packet output | **PASS, bounded** | Default-redacted packet: 2,813 entities, 6,173 relationship records, all 2,813 entity centers rendered, zero labels outside SVG viewport. Names, relationship labels, IDs, and evidence references were not included. |
| Release-profile build and desktop install | **BLOCKED** | Multiple locked offline release attempts failed to produce a new binary in the constrained build session. Existing release binary remained timestamped `2026-11-28 06:24:22 -0500`, SHA-256 `b90491201384e11ae359fcf4821f8d332fecddd9e1d92f1b0bb477c8f0fa80cd`. No current-source release install is claimed. |
| Human content review, PDF extraction completeness, Qwen analysis | **NOT RUN** | Outside this local acceptance harness. |
| Native UI review and IBM interoperability | **NOT RUN** | No native display or IBM i2 Analyze connection was available. |

## Aggregate artifacts

- Installer report: [SQL-SEARCH-CATALOG-02-debug-install.json](artifacts/SQL-SEARCH-CATALOG-02-debug-install.json), SHA-256 `a5000470211812a8eb54e136ac22ba2c84c367e5290db58c1820ec2848c96733`.
- Clinton report: [SQL-SEARCH-CATALOG-02-clinton.json](artifacts/SQL-SEARCH-CATALOG-02-clinton.json), SHA-256 `e6d4dc9a667f9fffcabff480064f600a45092dd988c337ac6691fe8ddf59cfb0`.

Reports contain aggregate counts, source-file digests, permissions, and results only. They do not contain email names, bodies, subjects, or relationship rows.
