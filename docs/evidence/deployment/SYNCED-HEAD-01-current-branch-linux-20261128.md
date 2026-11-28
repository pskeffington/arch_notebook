# Current branch build and installed app synchronization — 2026-11-28

## Handoff

- Task: verify that the current Arch Notebook product branch is built and synchronized with the current-user Linux desktop launcher.
- Owner: Codex session (`/root`).
- Repository: `/tmp/arch-notebook-remote-candidate`.
- Source commit: `dac7300f26c4812dd66601bc5345d70dda5d3217`.
- Branch: `codex/ui-00-chart-nested-conditions`.
- Product source comparison baseline: `c653f31`.
- Worktree before this evidence update: clean.
- Scope: optimized Rust release build, binary identity, installed launcher, desktop entries, icons, MIME file, and SQL automation helper.

## Commands and results

| Command | Exit | Result |
| --- | ---: | --- |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | 0 | Optimized release completed successfully. |
| `python scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report /tmp/arch-notebook-current-build-install-acceptance.json` | 0 | Isolated installer acceptance returned `PASS`. |
| `git diff --quiet c653f31 HEAD -- src crates assets packaging scripts` | 0 | No product-source, package, or launcher asset changes after the installed product source commit. |
| `/home/sysadmin/.local/bin/arch-notebook --version --json` | 0 | `{"name":"arch-notebook","schema_version":1,"version":"0.1.0"}`. |

Release executable SHA-256: `46596680af31ba1d92314426e6fd69feb7179a9d8489038cbcca1ee48ac42c1a`.

Installed executable SHA-256: `46596680af31ba1d92314426e6fd69feb7179a9d8489038cbcca1ee48ac42c1a`.

Acceptance report: `/tmp/arch-notebook-current-build-install-acceptance.json` (SHA-256 `700eb03a583dd3afb59517c85108f06b81d87fc9f17fe071db2e04d44dce2821`). It records successful installation into an isolated temporary home, identical application-menu and Desktop launchers, passing `desktop-file-validate`, and matching release/package hashes for the binary, SVG, PNG, MIME definition, and SQL automation helper.

## Gates

| Gate | Result | Limits |
| --- | --- | --- |
| Locked offline release build | PASS | Release compilation only; does not assert runtime correctness for every workflow. |
| Installed app matches current product code | PASS | Product sources are identical to `c653f31`, and release/installed executable hashes match. Documentation at `dac7300` is newer than that product source. |
| Current-user launcher and graphics synchronization | PASS | Isolated installer acceptance validates app menu, Desktop shortcut, icons, MIME, and SQL helper. |
| Native pointer/keyboard review, live SQL scheduler dispatch, IBM application parity | NOT TESTED | These require interactive or external application/runtime validation and are not implied by packaging checks. |

## Next action

Continue product work from the current branch; rebuild and repeat acceptance whenever product sources or packaged assets change. Preserve the documented NOT TESTED gates until direct runtime evidence is available.
