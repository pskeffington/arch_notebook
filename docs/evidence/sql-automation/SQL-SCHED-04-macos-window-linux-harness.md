# SQL-SCHED-04: Rolling SQL window through the macOS runner harness

- **Result:** PASS for the isolated LaunchAgent runner and scheduled SQL capture; actual macOS `launchd` dispatch remains unverified.
- **Owner:** Arch Notebook implementation session.
- **Baseline:** `5f175aab0b9d656f88b836585ed97b70bffe2328` on `codex/ui-00-chart-baseline`.
- **Changed paths:** `scripts/test-macos-sql-scheduler-acceptance.py`, `README.md`, and this evidence record.
- **Harness SHA-256:** `f225da3f3c706af26c7ede675988df0c80c7691eee7f44eb4e7662f6a47afdaf`.
- **Runner SHA-256:** `f2aeb3e6fb56d09e7a694e7ded4c806eea01756f6aacb7f1f5f02b5390204d1c`.
- **Installer SHA-256:** `961fd189d3ef6f6ff07ea9b2c72fa47ef026e67c92726259a447dbfff42ceac5`.
- **Binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `0614576294efd356f606081dc437d22672e0e0a48b4612f86b436a3e32f694b9`.
- **Source SQL fixture SHA-256:** `5bf58753d01310afa53366098c9ee6c5535f4ea74a5b4e5e84708275a133e9df`.
- **Work directory:** `/tmp/arch-notebook-macos-scheduler-window-acceptance-20261126b`.
- **Acceptance report:** `/tmp/arch-notebook-macos-scheduler-window-acceptance-20261126b/acceptance-report.json`, SHA-256 `25bb592d66e0a2e658f485e72a5d7bd73be5b7df4954ed881161c6548467c38d`.
- **Synthetic scheduled source:** `scheduled-window.sqlite`, SHA-256 `7c62cccec9459451a0c00da3cbb35265be7bf36f7ddb81c6bf18527c0d1a3071` after the run.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| `PYTHONPYCACHEPREFIX=/tmp/arch-notebook-pycache python3 -m py_compile scripts/test-macos-sql-scheduler-acceptance.py` | PASS, exit 0 | Harness syntax compiled. |
| `python3 scripts/test-macos-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-macos-automation.sh --runner scripts/run-macos-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-macos-scheduler-window-acceptance-20261126b` | PASS, exit 0 | Full report says `PASS`; five LaunchAgent cadences, preflight, rollback, removal, and legacy schedule checks passed. |
| Rolling-window boundary selection through the bundled runner | PASS | SQLite returned only `WINDOW-START` for `[2026-10-01, 2026-11-01)`. `BEFORE-WINDOW` and `WINDOW-END` were excluded. |
| Scheduled run, snapshot, and chart CSV | PASS | Runner invoked the Arch Notebook CLI with the scheduled trigger and chart-export directory; one scheduled run, one connection snapshot, and a one-row CSV were saved. |
| Read-only SQLite source | PASS | Harness SHA-256 check confirms `scheduled-window.sqlite` was unchanged by the scheduled CLI run. |
| `git diff --check` | PASS, exit 0 | No whitespace errors in the candidate changes. |

The harness creates a fresh SQLite source from the committed fictional SQL fixture, adds only synthetic boundary rows, and installs LaunchAgents using a temporary `launchctl` stub. It then executes the packaged `run-macos-automation.sh` with the real Arch Notebook CLI proxy. The source date boundaries are computed by SQLite itself in UTC. This is a Linux-hosted runner simulation, not a native macOS or live `launchd` test.

## Handoff

The rolling SQL window semantics now have generated-service acceptance on Linux and isolated runner acceptance for the macOS packaging path. Continue with native macOS scheduler dispatch and failure/restart review when a Mac host is available; native desktop Automation interaction and the UI-00 chart gate remain open.
