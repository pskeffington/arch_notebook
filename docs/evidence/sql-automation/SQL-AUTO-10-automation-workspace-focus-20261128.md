# SQL-AUTO-10 — focused Automation workspace

- **Task / owner:** Reduce navigation and file-exchange clutter in the SQL Automation workspace while preserving those fields in other case workspaces; Rust implementation and acceptance.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `3c7587596c06ea47d20aa35948afcf590f3141af`.
- **Source commit:** `7811d5256b50bc319c7c78b2a5e6e68a76ef41bc` (`Focus the SQL automation workspace`).
- **Changed product path:** `src/main.rs`.
- **Runtime binary:** `/tmp/arch-notebook-target/release/arch-notebook`; SHA-256 `979aeb9aef259dbbc410088f96a5241da659f07998e7d8e011ca71b924042976`.
- **Host:** Linux x86_64. Acceptance used a headless 1440×900 egui frame and local fictional/read-only SQL fixtures; no native schedule was installed.

## Change

The persistent Automation destination remains the route into the workspace. Inside Automation, the duplicate **CASE RECORDS** tab row is replaced with a **SQL AUTOMATION** heading. The generic Description and five interchange-path fields are hidden while Automation is active; the case path remains visible. Other data modes retain the existing navigation and file fields. Field values remain in app state and reappear after switching back to Connections.

## Acceptance results

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Automation workspace headless interaction | **PASS** | At 1440×900, the Automation destination, case path, SQL Automation heading, and repeatable-jobs heading are visible. Duplicate case tabs, Description, and import/export paths are absent. Connections restores those controls and their previous values. This is headless widget interaction, not native visual acceptance. |
| Rust formatting, check, tests, debug build, release build | **PASS** | Full suite: 75 passed, 0 failed, 3 ignored. All locked offline commands completed successfully. |
| SQL capture/adaptation and interchange | **PASS** | Read-only capture, expected schema-drift rejection without case write, retained failure diagnosis, reviewed repair, batch recovery/configuration-mismatch protection, comparison, and CSV/TSV round trips passed. |
| Joined-header scale | **PASS** | Two snapshots captured 7,121 fictional header-shaped records, 7,121 composite IDs, and 4,121 repeated endpoint pairs; zero skipped rows. Nullable attributes passed and the source stayed unchanged during each capture. |
| Scheduled batch interruption recovery | **PASS** | SIGKILL after the first checkpoint; next invocation resumed the same batch, skipped the completed job, captured the pending job, and saved two runs. Manual use of the scheduled-only switch was rejected without changing the case. |
| Linux and macOS scheduler-generation harnesses | **PASS, isolated** | Five cadence configurations passed on each platform harness. `systemctl` and `launchctl` were stubs; no native schedule was installed. |
| Primary Clinton SQL QA | **PASS, bounded** | Read-only preflight and saved-run replay passed for 6,173 rows, zero skipped rows and zero deltas across 2,813 entities. PDF, case, database and binary hashes are recorded; inputs remained unchanged. Qwen/content review and PDF completeness are not established. |
| Native UI review, live schedule dispatch, reboot recovery, IBM validation | **NOT RUN** | Headless UI and isolated scheduler harnesses do not establish those gates. |

## Commands and observed exit codes

Run from the repository root; build output used `/tmp/arch-notebook-target`.

```text
cargo fmt --all -- --check
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline
exit 0; 75 passed, 0 failed, 3 ignored

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --workspace --locked --offline
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline automation_workspace_hides_duplicate_navigation_and_file_exchange_fields
exit 0; 1 passed

python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-auto-current-20261128
exit 0; acceptance report result PASS

python3 scripts/test-sql-scale-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --work-dir /tmp/arch-notebook-sql-scale-current-20261128
exit 0; 7,121 rows, zero skipped

python3 scripts/test-scheduled-sql-batch-resume.py --binary /tmp/arch-notebook-target/release/arch-notebook --work-dir /tmp/arch-notebook-scheduled-resume-7811d52-20261128
exit 0; same-batch recovery PASS

python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-linux-scheduler-7811d52-20261128
exit 0; five cadence configurations PASS; systemctl stubbed

python3 scripts/test-macos-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-macos-automation.sh --runner scripts/run-macos-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-macos-scheduler-7811d52-20261128
exit 0; five cadence configurations PASS; launchctl stubbed

python3 scripts/test-clinton-corpus-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/clinton-corpus-qa-current-20261128.json
exit 0; aggregate result PASS; source inputs unchanged
```

The first targeted invocation used Rust's `--exact` flag without the test module path and exited 0 while matching zero tests. It was discarded; the corrected targeted command above ran one test, and the full suite also passed.

## Aggregate artifacts

- Headless Automation workspace result: [SQL-AUTO-10-ui-7811d52.json](artifacts/SQL-AUTO-10-ui-7811d52.json), SHA-256 `139fce9c0c260a8dbf950a17766a15f191df2f9eb6249a71987b56c631d33d6e`.
- Synthetic SQL capture/adaptation: [SQL-AUTO-10-synthetic-7811d52.json](artifacts/SQL-AUTO-10-synthetic-7811d52.json), SHA-256 `9f781e8938d74b067d81e4cafb6b17c8097634faa476e04d1c6913344ecbe080`.
- Joined-header scale: [SQL-AUTO-10-scale-7811d52.json](artifacts/SQL-AUTO-10-scale-7811d52.json), SHA-256 `c16c67713d78eb7ba4a4455a2c10f2adffd99940fa60f31881bec2f8f8af8792`.
- Interrupted scheduled batch recovery: [SQL-AUTO-10-batch-resume-7811d52.json](artifacts/SQL-AUTO-10-batch-resume-7811d52.json), SHA-256 `de1b97d93e3adad28885ea73c7b8797b3da8047c2d86957c1ca9df48c8c20124`.
- Linux schedule harness: [SQL-AUTO-10-linux-scheduler-7811d52.json](artifacts/SQL-AUTO-10-linux-scheduler-7811d52.json), SHA-256 `2c55225bab443c43ed5a592f68e353f97444ca2cad1c8e45d741aefa5ce1ea30`.
- macOS schedule harness: [SQL-AUTO-10-macos-scheduler-7811d52.json](artifacts/SQL-AUTO-10-macos-scheduler-7811d52.json), SHA-256 `e45e4720c4047eb753b7e4d077db42253cf1d3f32dfb43f642bb88557bda37fc`.
- Clinton aggregate QA: [SQL-AUTO-10-clinton-7811d52.json](artifacts/SQL-AUTO-10-clinton-7811d52.json), SHA-256 `eaeb661393e1ba23330d4c15d1c3a3f01d17559736f89bbf3d280d946ac49bca`.

The Clinton report contains aggregate counts and file digests only; it does not contain email names, bodies, subjects, or relationship rows. Other acceptance artifacts use fictional fixtures.

## Remaining work

Review the Automation workspace with native pointer and keyboard input at common analyst display sizes. Validate actual systemd timer installation/dispatch and recovery separately before claiming live scheduling acceptance. The broader IBM interaction and ANX interoperability gates remain open.
