# Local build acceptance record

**Task:** `01a2016c-06c1-7b02-bb7d-a0ac3edcdd3c` — continue the Rust Arch Notebook parity build, including SQL automation and deployed application graphics.
**Repository:** `/home/sysadmin/github/arch_notebook`
**Branch / source base:** `main` / `7677248ebbc8732a38aa8c8187f2fe61c1799c7a`
**Recorded:** 2026-11-25
**Candidate state:** dirty worktree; no commit or immutable source-tree digest identifies the combined candidate. The checks below apply to the working tree at the time they ran.

This record accepts the local Linux Rust build and the listed packaging metadata checks. It is not a release-readiness decision or authorization to publish or deploy.

## Gate results

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Scope and source recorded | PASS | Task thread ID, repository, branch, base SHA, and local-build scope are recorded above. The worktree contains accumulated uncommitted changes. |
| Local format and integration build | PASS | Formatting, diff validation, offline Rust check, debug build, and release build all exited 0; exact commands are below. |
| Linux desktop metadata and icon files | PASS | `desktop-file-validate` accepted the desktop entry; SVG and 1024×1024 PNG files were recognized. |
| Packaging script syntax | PASS | `bash -n` accepted the Linux user installer and macOS bundler. This checks syntax only. |
| Behavioral and independent verification | BLOCKED | No automated tests or independent review were run. Source-identity navigation and UI behavior have build evidence only. |
| Native platform packages and runtime graphics | BLOCKED | The Arch package and macOS `.app`/`.icns` were not built here. This Linux run did not execute macOS `sips`/`iconutil`, install the app, or inspect its runtime window icon. |
| IBM interoperability and visual parity | BLOCKED | ANX was not validated against IBM's XSD or Analyst's Notebook. IBM's published guidance locates those artifacts in an installed Analyst's Notebook tree: [IBM ANX format support note](https://www.ibm.com/support/pages/node/546669). No IBM application review was performed. |
| Release authorization | BLOCKED | No release dossier, trusted approval signature, or human publication/deployment authorization is present. |

## Commands and results

The following local build chain completed with exit code 0:

```sh
cargo fmt --all -- --check
git diff --check
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline
```

The packaging and asset checks completed with exit code 0:

```sh
bash -n scripts/install-linux-user.sh scripts/package-macos.sh
desktop-file-validate packaging/arch/arch-notebook.desktop
file assets/icons/arch-notebook.svg assets/icons/arch-notebook.png
identify assets/icons/arch-notebook.png
```

No automated tests were run. Cross-platform packaging and live scheduler/UI behavior remain unverified.

## Artifact digests

These hashes identify the local binaries and icon assets produced or checked for this candidate; they do not bind the dirty source tree to a commit.

| Artifact | SHA-256 |
| --- | --- |
| `/tmp/arch-notebook-target/debug/arch-notebook` | `eca98ce1ea794650e8bcf89299900e1beebe53dc6fd4664b9d3ec60960a96aa8` |
| `/tmp/arch-notebook-target/release/arch-notebook` | `ad7e8ca413583cda5ffe3502b670c7e0dc33c3c23f7494a88a52fc47e7ce17f0` |
| `assets/icons/arch-notebook.svg` | `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8` |
| `assets/icons/arch-notebook.png` | `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88` |

## Candidate worktree paths

These modified and untracked paths were present during acceptance. The worktree contains accumulated changes; this list does not claim that every path was authored in this acceptance turn.

- `.github/workflows/build.yml`, `README.md`, `src/main.rs`
- `assets/icons/arch-notebook.svg`, `assets/icons/arch-notebook.png`
- `docs/analyst-notebook-parity.md`, `docs/automation.md`, `docs/build-and-deploy.md`, `docs/chart-legends.md`, `docs/evidence-handoff.md`, `docs/notebook-workflows.md`, `docs/project-reference.md`, `docs/sql-ingestion.md`
- `packaging/arch/PKGBUILD`, `packaging/arch/arch-notebook.desktop`, `packaging/arch/arch-notebook-case.xml`, `packaging/arch/arch-notebook.install`
- `packaging/systemd/arch-notebook-automation.service`
- `scripts/install-linux-automation.sh`, `scripts/install-linux-user.sh`, `scripts/install-macos-automation.sh`, `scripts/package-macos.sh`, `scripts/run-macos-automation.sh`

## Next acceptance work

1. Run the repository's automated tests and record their exact results.
2. Build and inspect the Arch package and Linux archive installation on Linux; build the macOS `.app` and inspect its Dock/Finder icon on macOS.
3. Exercise SQL scheduling, cancellation, resume, and failure review with a disposable case on supported platforms.
4. Validate ANX fixtures with IBM's XSD and application, then update the parity map with the results.
5. Repeat the checks against a clean, immutable source commit before any release decision.
