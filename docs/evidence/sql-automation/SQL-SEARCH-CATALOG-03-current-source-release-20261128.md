# SQL-SEARCH-CATALOG-03 — current-source release and Clinton QA

- **Task / owner:** Verify and install the saved SQL search/Quick Open candidate; primary agent.
- **Repository / branch / source commit:** `arch_notebook` / `codex/ui-00-chart-nested-conditions` / `7df1bf6e95e7cadf8baf58796fdcbb3544f8f1ed`.
- **Build:** `TMPDIR=/home/sysadmin/.cache/arch-notebook-build-tmp CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/home/sysadmin/.cache/arch-notebook-target cargo build --release --workspace --locked --offline`; exit 0, release profile completed in 2m53s.
- **Release binary:** `/home/sysadmin/.cache/arch-notebook-target/release/arch-notebook`; SHA-256 `cf9948f467a6f620b17381724984fa131f82dbb8c2f09ad684f8b33b76a90aaf`.
- **Installed binary:** `/home/sysadmin/.local/bin/arch-notebook`; SHA-256 matches the release artifact.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Locked offline release build | **PASS** | Exact command above, exit 0, optimized release profile. |
| Isolated Linux per-user installer acceptance | **PASS** | `scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/arch-notebook-linux-x86_64/arch-notebook --report /tmp/SQL-SEARCH-CATALOG-03-release-install.json`, exit 0. Binary, menu/Desktop launcher equality, file modes, desktop-file validity, SVG/PNG icons, SQL automation helper, MIME definition, and version checks passed. |
| Current-user desktop installation | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-target/arch-notebook-linux-x86_64`, exit 0. The installed binary matches the release digest. Menu and Desktop launchers match. Installed icon assets and MIME definition each match repository asset digests. `arch-notebook --version --json` reports version `0.1.0`. |
| Primary Clinton corpus QA | **PASS** | `scripts/test-clinton-corpus-acceptance.py` with the release binary and retained local PDF/case/database, exit 0. One query-only job passed; replay passed for 6,173 relationship rows; zero skipped rows and zero added/changed/removed deltas; source inputs unchanged. |
| Default-redacted topology packet | **PASS, bounded** | 2,813 entities, 6,173 relationship records, all 2,813 entity centers rendered, zero labels outside the SVG viewport. Names, relationship labels, IDs, and evidence references excluded. |
| Human content review, PDF extraction completeness, Qwen analysis | **NOT RUN** | Not covered by the local corpus harness. |
| Native UI review and IBM interoperability | **NOT RUN** | No native input/display or IBM i2 Analyze integration was exercised. |

## Artifacts

- Isolated installer report: [SQL-SEARCH-CATALOG-03-release-install.json](artifacts/SQL-SEARCH-CATALOG-03-release-install.json), SHA-256 `ac425320f5fc554c931bc0392abc03e5f94a494a0cb84ff820158af526c407af`.
- Clinton corpus report: [SQL-SEARCH-CATALOG-03-release-clinton.json](artifacts/SQL-SEARCH-CATALOG-03-release-clinton.json), SHA-256 `72140b72aa42d6d19244c4347507b7f977696538f26c02f540070562ae9fbcdd`.

The corpus report contains aggregate results and input digests, not message names, email bodies, subjects, or individual relationship rows.
