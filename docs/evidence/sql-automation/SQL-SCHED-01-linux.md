# SQL-SCHED-01 Linux scheduler acceptance evidence

- **Task / owner:** Safely verify Linux SQL schedule installation, status and removal; Codex implementation and local verification.
- **Baseline:** `da95aaf4152acd50cf8133e543b0156f01102f10`; candidate `1e5a86f1cf0ee34c9cda7adca5b64c6bac6827d4` on `codex/ui-00-chart-baseline`.
- **Scope:** Run the real Rust CLI's SQL preflight and the Linux installer against temporary HOME/XDG roots; substitute a temporary `systemctl` stub so no live user timer is installed or changed.
- **Changed paths:** `scripts/install-linux-automation.sh`, `scripts/test-linux-sql-scheduler-acceptance.py`, `.github/workflows/build.yml`, this evidence file.

## Findings

The isolated scheduler flow found two installed-schedule status defects. Export-folder reporting assumed `--export-chart-csv-dir` was always the fourth `ExecStart` argument, but the generated command includes `--progress` first. Preflight policy reporting used the second `=`-separated field and returned the environment variable name instead of its value. The status parser now locates the export option by name and reads the final environment assignment field.

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| Five Linux cadences install separate service/timer pairs | PASS | Hourly, every 15 minutes, daily, weekly and monthly; each case gets distinct units. |
| SQL preflight invokes the real Arch Notebook CLI | PASS | Five runs use `--check-enabled-job-queries <case> --progress`; `--require-usable` is present for four enabled policies and absent for the explicit weekly opt-out. |
| Status restores saved schedule settings | PASS | Weekly Tuesday 15:45 and the `false` usable-connection policy are reported; export directory with quote, dollar and percent characters is recovered. |
| Unit command preserves special-character paths | PASS | Parsed `ExecStart` retains case, binary and chart export paths with spaces, quotes, `$` and `%`. |
| Failed SQL preflight changes no units or systemd state | PASS | A deliberately invalid query exits 1 before unit creation or any `systemctl` call. |
| Invalid monthly day is rejected | PASS | Day 29 exits 2 with the supported 1–28 boundary; existing units remain unchanged. |
| Case-specific removal is isolated | PASS | Removing one case deletes its service and timer while all four other cases remain. |
| Live user timer untouched | PASS | All operations used a temporary `HOME`, `XDG_CONFIG_HOME` and stub `systemctl`; no host timer was installed, enabled, disabled or restarted. |
| Remote CI | PASS | [GitHub Actions run 45709682338](https://github.com/pskeffington/arch_notebook/actions/runs/45709682338), source `1e5a86f1cf0ee34c9cda7adca5b64c6bac6827d4`: Linux scheduler/SQL/ANX tests, Linux x86_64 and ARM64, Arch packaging, and macOS Intel and Apple Silicon all succeeded. |
| Actual scheduled firing/restart recovery | NOT RUN | Requires native systemd timer/runtime acceptance; this harness validates unit generation and installer/status/removal behavior. |

## Commands and observed exit codes

From `/home/sysadmin/github/arch_notebook`:

```text
bash -n scripts/install-linux-automation.sh
exit 0

python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/debug/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py
exit 0 (5 cadence configurations; 27 stubbed systemctl calls)
```

The scheduler harness calls `test-sql-automation-acceptance.py` with a retained synthetic case so installation preflight uses a real SQLite database and the real Rust CLI. It prints a JSON report and removes its temporary tree after success.

## Limits and next action

This is not evidence that a native timer fired, survived sleep/reboot, recovered a partially completed batch, or emitted records into a real user journal. Run the interactive systemd service/timer workflow on a native Linux host, then exercise cancellation and resume. The macOS installer now has a separate [isolated LaunchAgent acceptance record](SQL-SCHED-02-macos-isolated.md); native macOS launchd behavior remains unverified.

## Generated service invocation follow-up — 2026-11-25

- **Source baseline:** `ca23dc8e1cd860440566ed5070ef42d266f74360` on `codex/ui-00-chart-baseline`.
- **Changed path:** `scripts/test-linux-sql-scheduler-acceptance.py`.
- **Acceptance script SHA-256:** `b9b093b3a8109ba365402ca6ffc22f00de5928cb8cbb866df7e0178ae4dccaa6`.
- **Rust CLI SHA-256:** `9bcc9ff9b1a505177373f5081094c1419c4079f92121b13c2c0ee67267ed94be`.

The scheduler harness now creates its own fresh SQLite database instead of reusing the SQL acceptance database after that test intentionally renames `event_date` to exercise schema adaptation. It parses and directly invokes one generated `ExecStart` using the disposable service environment and temporary case. This is an end-to-end invocation of the installed service command with the real Rust CLI; systemd itself remains stubbed.

| Check | Result | Evidence |
| --- | --- | --- |
| `PYTHONPYCACHEPREFIX=/tmp/arch-notebook-pycache python3 -m py_compile scripts/test-linux-sql-scheduler-acceptance.py` | PASS, exit 0 | Harness syntax compiled. |
| `python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/debug/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-linux-scheduler-invocation-20261125b` | PASS, exit 0 | Five cadence setups, 27 expected stub calls, plus generated service command exit 0. The command saved one successful `scheduled` run, a four-connection snapshot, and a CSV with four rows. Report SHA-256: `e3093e7932be3664bd410f71fdd43d110a4c3cd312362b2f58301d763489738e`. |
| Fresh scheduler SQLite database | PASS | `/tmp/arch-notebook-linux-scheduler-invocation-20261125b/scheduler.sqlite`, SHA-256 `cbb7a0a273b1b036145f89d652ab28adf24e1c8ac0177a98cf31f206c5b17d9c`. |
| `systemctl --user is-system-running` | BLOCKED | User-bus connection returned `Operation not permitted`. No live timer was installed or changed. |

The generated service invocation is not a timer firing. Native timer dispatch, reboot/sleep persistence, and cancellation remain pending.
