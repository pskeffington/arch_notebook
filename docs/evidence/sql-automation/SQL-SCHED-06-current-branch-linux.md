# Current-branch SQL scheduler harness recheck

- **Task / owner:** Recheck generated Linux systemd and macOS LaunchAgent SQL scheduling against the current release binary; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source commit:** `750b6cb9f9214089678cc3341dae5ecd7c65005b`.
- **Date / host:** 2026-11-26, Linux x86_64.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`; SHA-256 `f8c14cabd73164b27218068685d75c62ecd2e9d854287299b9aca534ec373032`.
- **Fixture:** `archive/research.databases/sample_connections.sql`, fictional synthetic SQLite data.

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| Linux installer and generated service invocation | **PASS, harness-scoped** | `python3 scripts/test-linux-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-linux-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-sched-linux-20261126-01`, exit 0. Report SHA-256 `8dacdb479cf509551b5bf64b282cb978e6c784e758d777e4c8da533a42cb3147`. |
| macOS installer and generated runner | **PASS, Linux stub harness** | `python3 scripts/test-macos-sql-scheduler-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --installer scripts/install-macos-automation.sh --runner scripts/run-macos-automation.sh --source-sql archive/research.databases/sample_connections.sql --sql-acceptance scripts/test-sql-automation-acceptance.py --work-dir /tmp/arch-notebook-sched-macos-20261126-01`, exit 0. Report SHA-256 `36e238619aa2b1f08bc9fe2839bbf4fa713520d17e1a23e052d3448b2cfa0bd1`. |
| Linux host systemd timer dispatch | **NOT VERIFIED** | The Linux test uses a temporary `systemctl` stub. It invokes the generated service command and verifies a scheduled-trigger capture and CSV export, but does not establish live timer dispatch. |
| Native macOS launchd dispatch | **NOT RUN** | The LaunchAgent test uses a temporary `launchctl` stub on Linux and confirms the real macOS scheduler was untouched. |

## Verified cases

Both harnesses passed five schedule cadences, real Arch Notebook CLI preflight, expected installation rejection after failed SQL preflight, preservation of special-character paths, and execution of the generated scheduled runner against synthetic SQLite. Each generated service/runner saved one scheduled capture and one-row chart CSV. Rolling UTC calendar-window boundaries included the start and excluded the end. The Linux harness preserved unrelated case schedules during removal; the macOS harness verified failed first install leaves no agent, failed replacement restores the previous plist/agent, and case-specific removal preserves legacy compatibility. Each harness invokes its own nested full SQL automation acceptance flow.

## Limits and next action

These results verify installer/runner behavior under isolated command stubs and execution of the generated Arch Notebook CLI command. They do not prove live Linux systemd or native macOS launchd delivery, restart after host sleep/reboot, native Automation UI behavior, IBM compatibility, or release readiness. Perform those checks on supported native hosts when available.
