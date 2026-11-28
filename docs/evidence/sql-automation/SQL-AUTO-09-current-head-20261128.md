# SQL-AUTO-09 — current source build and SQL automation acceptance

- **Task / owner:** Revalidate Rust build, SQL automation, scale, scheduler recovery, and primary Clinton QA on the current feature-branch source; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Source commit:** `f9434d09aa3aac4ec6ffca652efdc3110336b9b8` (`Resume interrupted scheduled SQL batches safely`).
- **Runtime binary:** `/tmp/arch-notebook-target/release/arch-notebook`; SHA-256 `18c01b58b05b8d4b626aa38f497d107080b117cb480535ccfb0030f85899811d`.
- **Host:** Linux x86_64. Fixture captures and generated artifacts were kept under `/tmp`; the Clinton case, SQLite database, and source PDF were read-only inputs.

## Acceptance results

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Rust formatting, check, tests, debug build, release build | **PASS** | `cargo fmt --all -- --check`, locked offline workspace check/builds, and full workspace tests passed. Tests: 74 passed, 0 failed, 3 ignored. |
| SQL automation and adaptation harness | **PASS** | Read-only capture and schema preflight, intentional failed-run retention, structured missing-column diagnosis, reviewed repair, batch resume/configuration mismatch protection, comparison export, and CSV/TSV round trips passed. The expected schema-drift preflight result was `FAIL`; it did not modify the saved case. Source DB hashes stayed unchanged during captures. |
| Joined-header scale workload | **PASS** | Two snapshots captured all 7,121 fictional header-shaped records, preserving 7,121 composite IDs and 4,121 repeated endpoint pairs; zero rows skipped. Nullable attributes were handled and the source SQLite file stayed unchanged during each capture. No email bodies or real people were used. |
| Scheduled process interruption and recovery | **PASS** | The scheduled process was SIGKILLed after its first job checkpoint. The next invocation resumed the same batch, skipped the completed job, captured the pending job, and saved two runs. Manual use of the scheduled-only switch was rejected without changing the case. |
| Linux schedule generation | **PASS, isolated** | Five cadences and generated service invocation passed; `systemd-analyze` accepted all five calendar expressions. `systemctl` was stubbed by the harness. |
| macOS schedule generation | **PASS, isolated** | Five cadences, generated runner, replacement rollback, failure behavior, and case-specific removal passed. `launchctl` was stubbed; the host's native launchd was untouched. |
| Primary Clinton corpus SQL QA | **PASS, bounded** | Read-only preflight and latest saved-run replay passed for 6,173 rows, zero skipped rows and zero deltas across 2,813 entities. PDF, case, SQLite, and executable hashes are recorded in the aggregate-only report; input hashes remained unchanged. |
| Clinton semantic/content review and source completeness | **NOT RUN / NOT ESTABLISHED** | This SQL acceptance does not validate email meaning, PDF extraction completeness, or Qwen output. |
| Native live scheduler dispatch / reboot recovery | **NOT TESTED** | A read-only `systemctl --user list-timers --all` succeeded and showed an unrelated `traffic-analysis.timer`; no Arch Notebook schedule was installed or changed. macOS dispatch was not available on this Linux host. |
| IBM Analyst's Notebook interchange validation | **NOT TESTED** | No IBM schema or application was used. |

## Commands and observed exit codes

Run from the repository root. Build artifacts used `/tmp/arch-notebook-target`.

```text
cargo fmt --all -- --check
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline
exit 0; 74 passed, 0 failed, 3 ignored

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --workspace --locked --offline
exit 0

CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --workspace --locked --offline
exit 0

python3 scripts/test-sql-automation-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --source-sql archive/research.databases/sample_connections.sql --work-dir /tmp/arch-notebook-sql-auto-f9434d0-20261128
exit 0; acceptance report result PASS

python3 scripts/test-sql-scale-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --work-dir /tmp/arch-notebook-sql-scale-f9434d0-20261128
exit 0; 7,121 rows, zero skipped

python3 scripts/test-scheduled-sql-batch-resume.py --binary /tmp/arch-notebook-target/release/arch-notebook --work-dir /tmp/arch-notebook-scheduled-resume-f9434d0-20261128
exit 0; same-batch recovery PASS

python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-linux-scheduler-f9434d0-20261128
exit 0; five cadence configurations PASS; systemctl stubbed

python3 scripts/test-macos-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-macos-automation.sh --runner scripts/run-macos-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-macos-scheduler-f9434d0-20261128
exit 0; five cadence configurations PASS; launchctl stubbed

python3 scripts/test-clinton-corpus-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite --report /tmp/clinton-corpus-qa-f9434d0-20261128.json
exit 0; aggregate result PASS; source inputs unchanged

timeout 10s systemctl --user list-timers --all
exit 0; one unrelated timer listed; no Arch Notebook timer installed
```

## Aggregate artifacts

These JSON files are the retained reports for this exact source commit. Their SHA-256 digests are recorded alongside them:

- Synthetic SQL capture/adaptation: [SQL-AUTO-09-synthetic-f9434d0.json](artifacts/SQL-AUTO-09-synthetic-f9434d0.json), SHA-256 `b1bb90bc3f2d893da0b6849ecca03a988a118b955e36cdb64aa428627982afd2`.
- Joined-header scale: [SQL-AUTO-09-scale-f9434d0.json](artifacts/SQL-AUTO-09-scale-f9434d0.json), SHA-256 `c3e1e1eb5ae81e17b7bd37e2da3dc048a30911b295f87164c5e84c91b700227b`.
- Interrupted scheduled batch recovery: [SQL-AUTO-09-batch-resume-f9434d0.json](artifacts/SQL-AUTO-09-batch-resume-f9434d0.json), SHA-256 `70b0475bf85ca83b6e07181c45ee9b64a1df61b2ebb80b3d3a9d03e0d5251797`.
- Linux cadence/service harness: [SQL-AUTO-09-linux-scheduler-f9434d0.json](artifacts/SQL-AUTO-09-linux-scheduler-f9434d0.json), SHA-256 `80a986e34128997d1076c4ebfdd618ae1cf6cc50234d9d9f0afab8bb557eeadf`.
- macOS cadence/runner harness: [SQL-AUTO-09-macos-scheduler-f9434d0.json](artifacts/SQL-AUTO-09-macos-scheduler-f9434d0.json), SHA-256 `7f4c2177fb8f4761115d8d28077cbef176559142e7f804e9f4c935edeb644e72`.
- Clinton aggregate QA: [SQL-AUTO-09-clinton-f9434d0.json](artifacts/SQL-AUTO-09-clinton-f9434d0.json), SHA-256 `af2122682e10f72dfd81db921826815b1c47c040da22545826fe44828127566a`.

The Clinton report contains aggregate counts and file hashes only, not email names, bodies, subjects, or relationship rows. The harness reports are generated from fictional fixtures.

## Next action

The implementation and isolated scheduler tests are green at this source commit. Next, continue the SQL automation roadmap with native interaction review of the Automation workspace and, when separately authorized, an isolated live systemd schedule install/dispatch/recovery check. Do not infer native launchd or IBM compatibility from the isolated harnesses.
