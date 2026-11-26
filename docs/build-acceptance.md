# Local build acceptance record

## Current SQL automation acceptance — 2026-11-26

**Task / owner:** Resume Arch Notebook SQL ingestion and automation acceptance; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source baseline:** `bc536880d29075234525c0fde4f6a860caf71101`; product source `a5208a0e3aa8c0fe3b3a1e3f02986e45e97b919a`.
**Installed artifact:** `/home/sysadmin/.local/bin/arch-notebook`, SHA-256 `765f8fa2e6384a2df4747890a9b954920a1668dc35557068f749f89afcea1fc7`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| SQL scale ingestion and longitudinal comparison | PASS | 7,121 composite relationship IDs, two snapshots, 1,500 entities, NULL handling, query-only preflight, and zero skipped rows; synthetic fixture. |
| SQL automation, failure diagnosis, recovery, and interchange | PASS | Preflight, baseline/current capture, history, comparison export, schema drift rejection, retained failures, CSV/TSV round trip, reviewed query repair, and batch resume all passed. Mismatched resume configuration was rejected. |
| Linux scheduler generation and service invocation | PASS, harness scoped | Five cadences and case-isolated units; generated service command captured the expected snapshot and CSV. `systemctl` was stubbed; live timer dispatch is unverified. |
| macOS LaunchAgent runner | PASS, simulation scoped | Five cadences and case isolation; runner simulation generated the expected capture and CSV. `launchctl` was stubbed on Linux; macOS dispatch is unverified. |
| Native Automation UI, IBM XSD/application compatibility | BLOCKED | Pointer-driven Automation UI review and IBM validation have not been performed. |

Exact commands, exit status, report hashes, and fixture caveats are recorded in [SQL-AUTO-02](evidence/sql-automation/SQL-AUTO-02-current-release-linux.md). No tests or builds were run for this documentation update. The pre-existing `src/main.rs` worktree diff was left unstaged and is outside this evidence change.

> **Historical record:** The original local build below is retained as written for provenance. Its dirty `main` worktree and “no tests” findings describe the 2026-11-25 candidate only; they do not describe the current feature branch. Current branch evidence is summarized in [Current continuation: UI-00 and local install](#current-continuation-ui-00-and-local-install).

[![Linux local build: passing](https://img.shields.io/badge/Linux%20local%20build-passing-brightgreen)](#feature-branch-increment-reusable-chart-templates)

Badge scope: recorded Linux compilation at `ada3a01`; runtime, package installation and release acceptance remain separate.

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

## Current continuation: UI-00 and local install

**Task / owner:** Continue the Rust Arch Notebook parity build; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline/source:** `abffef37101bd298defd1e243c257a26e370323f` (matches `origin/codex/ui-00-chart-baseline` at this record).
**Scope:** Reconcile current branch acceptance status across SQL automation, native chart interaction, IBM interoperability, and the installed Linux launcher.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Branch/worktree provenance | PASS | `HEAD` and `origin/codex/ui-00-chart-baseline` resolve to `abffef37101bd298defd1e243c257a26e370323f`; worktree was clean before this documentation update. |
| SQL automation | PASS, scoped | Rolling previous-UTC-month selection is exercised through the Linux generated-service harness and the macOS runner harness. Evidence: [SQL-SCHED-03](evidence/sql-automation/SQL-SCHED-03-dynamic-window-linux.md) and [SQL-SCHED-04](evidence/sql-automation/SQL-SCHED-04-macos-window-linux-harness.md). Actual systemd timer dispatch and native macOS `launchd` dispatch remain unverified. |
| Local Linux installation and launcher/icon sync | PASS, scoped | Release binary, desktop/menu entries, case-file association, automation installer, and icons match and validate as recorded in [LOCAL-INSTALL-01](evidence/deployment/LOCAL-INSTALL-01-linux.md). GUI relaunch/interaction was not verified. |
| UI-00 native chart acceptance | BLOCKED | The latest runtime record describes a screenshot marked “Not Responding” and unavailable stable compositor access; responsive interaction, repeated workload sampling, and visual inspection remain open. Evidence: [UI-00 recheck](evidence/vector-ui/UI-00-linux-recheck-e44d485.md). Do not advance to UI-01 until this gate passes. |
| IBM interoperability | BLOCKED | Internal supported-subset ANX round-trip passes, but IBM XSD and Analyst's Notebook application validation have not run. Evidence: [ANX-ROUNDTRIP-01](evidence/anx/ANX-ROUNDTRIP-01-linux.md). |
| Remote workflow result for this source | BLOCKED | GitHub Actions API was unreachable during this continuation; no workflow conclusion is asserted for `abffef3`. Earlier CI evidence applies only to the source SHAs named in those records. |

No build or test command was run for this documentation-only reconciliation. Next action is to repeat the UI-00 sequence in a stable native desktop session; retain the IBM validation and live scheduler dispatch gates separately.

## Feature-branch increment: aspect-aware entity type layout

**Task / owner:** Improve analyst-facing vector organization; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline:** `0e54d8d70378a1a9f47f1268f679e381d105e997`.
**Source commit:** `a09b49c` (`feat: adapt entity clusters to chart canvas`).
**Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `README.md`.
**Scope:** Place entity-type clusters in a canvas-aspect-aware grid, center sparse cluster rows, preserve deterministic type/name ordering, and leave pinned entity positions unchanged.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Rust unit tests | PASS | Offline `cargo test` exited 0; 20 passed, 0 failed, 0 ignored, including three entity-type layout regressions. |
| Offline integration check and release build | PASS | `cargo check --locked --offline` and optimized `cargo build --release --locked --offline` exited 0. |
| Installed release binary, launcher, and art synchronization | PASS | Installed executable matched release binary SHA-256 `54d57b05cd9ebb66de4b60274fb4062e06a96705f4a68f80eabb7649e249bdb7`; desktop entries validate and match; installed SVG/PNG match repository assets. |
| Native chart inspection | BLOCKED | Wayland display access is denied in this session; layout appearance, responsiveness, and input remain unverified. UI-00 remains open. |
| IBM interoperability | NOT IN SCOPE / STILL BLOCKED | This feature does not establish IBM ANX schema or application compatibility. |

See [UI-ENTITY-TYPE-LAYOUT-02 evidence](evidence/vector-ui/UI-ENTITY-TYPE-LAYOUT-02-linux.md) for exact commands, hashes, and limitations. The change is pushed on the feature branch; proceed with native UI-00 interaction review when a compositor session is available.

## SQL acceptance: current-branch scheduler recheck

**Task / owner:** Verify rolling SQL window scheduling against the current application release; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source:** `a09b49c6faddbd0401c9ac59b191e8151a830e70`; current branch head at recheck: `cb0ff938ae1328b3f9a9859b19edae874f82138c`.
**Scope:** Execute the Linux generated service command and macOS packaged runner against isolated synthetic databases using the current release binary.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Linux generated-service SQL scheduling | PASS, harness-scoped | Five cadence installers passed; generated service saved one scheduled capture and one-row CSV; prior-UTC-month start/end behavior passed. `systemctl` was stubbed. |
| macOS automation runner and rollback lifecycle | PASS, harness-scoped | Five LaunchAgent schedules, preflight, rollback and removal passed under a temporary `launchctl` stub on Linux. Live `launchd` was not run. |
| Source DB immutability | PASS | macOS harness confirmed unchanged source SHA-256 `7c62cccec9459451a0c00da3cbb35265be7bf36f7ddb81c6bf18527c0d1a3071`. |
| Live host scheduler dispatch | BLOCKED | systemd user manager access and a native macOS host are unavailable in this run. |

Exact commands, report hashes, window boundaries, and work directories are recorded in [SQL-SCHED-05](evidence/sql-automation/SQL-SCHED-05-current-branch-window-linux.md). These harnesses use synthetic SQLite data and make no claims about live scheduler dispatch.

## Feature-branch increment: geodesic navigation and responsive ANX parsing

**Task / owner:** Improve analyst chart navigation and exchange-file ingestion; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline:** `dff7acfb5a5ffa9c79b692e0e32841821c4f64c8`.
**Source commit:** `b637d965ae5ed4aeea9ec4300860523dfb95b71b`.
**Changed paths:** `src/main.rs`, `README.md`, `docs/analyst-notebook-parity.md`, `docs/notebook-workflows.md`, and `docs/research/social-network-graphing-and-delivery-review.md`.
**Scope:** Add defined unweighted, undirected shortest-hop layers from an entity; move desktop ANX XML parsing to a worker while preserving synchronous CLI/automation behavior and guarding against chart-switch races.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting, diff, and Rust unit tests | PASS | Formatting and diff checks exited 0; offline tests: 22 passed, 0 failed, 0 ignored. |
| Offline integration check and release build | PASS | `cargo check --locked --offline` and optimized release build exited 0. |
| ANX internal round-trip and rejection behavior | PASS, internal scope | 8 entities, 8 relationships, and one label retained; malformed shape and DTD external entity rejected without case mutation. IBM XSD/application validation remains NOT RUN. |
| Current-source SQL scheduler harnesses | PASS, harness-scoped | Linux generated-service and macOS runner simulation selected the expected rolling-month row; SQLite source unchanged in macOS simulation. Host dispatch remains unverified. |
| User install and desktop graphics synchronization | PASS, scoped | Installed binary equals the release artifact; desktop launcher validation passed; installed and repository icon hashes match. |
| Remote workflow for source commit | BLOCKED / unverified | GitHub Actions API was unreachable; no remote workflow conclusion is asserted for `b637d96`. |
| Native UI rendering | PASS, render-only | The installed app rendered the fictional UI-00 chart, navigator, and geodesic panel in a native window. See [UI-00 render evidence](evidence/vector-ui/UI-00-native-render-20261126.md). |
| Native chart/input and ANX dialog interaction | BLOCKED | Pointer/keyboard event injection remains unavailable; responsive interaction and visual parity against IBM are not established. |

Exact commands, artifact hashes, report hashes, and remaining interoperability/platform gates are in [UI-INGEST-NAV-01](evidence/vector-ui/UI-INGEST-NAV-01-linux.md). The source commit is pushed on this non-main branch.

## Feature-branch increment: reusable chart templates

**Task / owner:** Continue Analyst's Notebook parity, especially adaptability; orchestrator-owned implementation.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/analyst-notebook-parity`.
**Baseline:** `204b114f9188d59ffeacf913a13acdd49cc593c2`.
**Source commit:** `ada3a012b813ed75fd0f119b2976e53c5c3d3810` (`feat: add reusable chart templates`).
**Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `docs/analyst-notebook-parity.md`.
**Scope:** Case-local chart templates store entity-type representations/colors and relationship vocabulary; apply styles to matching chart entities; provide defaults for manually added entities; persist the active template with chart tabs; allow template deletion. Import mapping and SQL query transformation are outside this increment.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Feature source recorded on non-main branch | PASS | Source commit above is on `codex/analyst-notebook-parity`; subsequently pushed to GitHub; originally local at time of verification. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0 against the feature source. |
| Offline integration check | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Debug build | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0. |
| Release build | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0. |
| Template runtime behavior / independent review | BLOCKED | No automated tests or independent review were run. Build evidence does not verify save, apply, delete, or chart-switch interactions in a running UI. |
| IBM visual and ANX parity | BLOCKED | No IBM application or IBM XSD/fixture validation was available. |
| Release authorization | BLOCKED | No publication or deployment was requested or performed. |

The four build commands above completed with exit code 0. The artifacts were produced from the committed Rust source; documentation-only acceptance logging is committed separately. Their SHA-256 digests are:

| Artifact | SHA-256 |
| --- | --- |
| `/tmp/arch-notebook-target/debug/arch-notebook` | `7956f64c8c8ca576eee056d91649042955543471f422a4b26f6534fe454e2442` |
| `/tmp/arch-notebook-target/release/arch-notebook` | `b48b93962e3d3a5f265c9a7aa1da46eab73b91454a614e0b0b41b70b26949c53` |

The product change is committed and reproducibly builds locally. Next acceptance work is interactive template workflow review, automated verification when requested, and IBM-backed interoperability review when the vendor artifacts are available.

## Feature-branch increment: Clinton email candidate capture

**Task / owner:** TRANSCRIBE-to-Arch Notebook integration evidence and SQL capture progress; orchestrator-owned.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/analyst-notebook-parity`.
**Baseline:** `1c7f912170417aa40e30b227a72234a60de13ccd`.
**Source status:** The SQL progress implementation and documentation are pending in this candidate; this acceptance entry records a local dataset run and does not imply a trusted or released TRANSCRIBE handoff.
**Scope:** Install the application on this Linux user's desktop, prepare a local review-required candidate case from the Clinton email PDF in the TRANSCRIBE share, and capture its header-derived relationship rows from SQLite.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Local SQL capture | PASS | The saved case contains 2,813 entities, 6,173 relationship records, two snapshots, and two automation runs. Both snapshots report 0 skipped rows; the latest batch completed with 6,173 captured relationships. The later capture followed cleanup of three out-of-range normalized dates. |
| Case and source safety | PASS | The case is explicitly titled `CLINTON EMAIL PUBLIC RELEASE · LOCAL REVIEW REQUIRED`; all extracted records remain unreviewed candidates. The SQLite source and case are local share artifacts and are not included in this repository. Raw message bodies were not stored in the analysis database. |
| Data completeness and semantic validation | BLOCKED | Extraction yielded 4,767 header candidates from 7,121 PDF pages, not a verified complete set of 7,000 emails. No human validation or released TRANSCRIBE handoff was performed. The counts above describe the captured candidate graph only. |
| Qwen analysis | BLOCKED | The local Ollama endpoint could not be reached under the current execution sandbox, and no connected Qwen analysis tool was available. No LLM analysis is claimed. |
| Desktop installation | PASS | The Linux user installer placed the application launcher, icon assets, MIME definition, and desktop shortcut in the user's local application directories. `desktop-file-validate` passed both installed launchers; the installed executable reported version 0.1.0. Desktop filesystem trust metadata was unsupported, although the shortcut is executable. |
| Rust build and diff checks | PASS | `cargo fmt --all -- --check`, `git diff --check`, offline `cargo check`, debug build, and release build passed for the candidate source before installation. No automated tests were run. |

Local artifacts are held in the user's TRANSCRIBE share and excluded from Git. Email names, addresses, subjects, bodies, and row-level analysis are intentionally omitted from this repository record. The DB stored source/transcription provenance and records that Qwen was not run.

**Current Rust CLI follow-up (2026-11-25, source `aaf8ece`):** The retained case still opens in the release CLI; run and batch history list two `PASS` runs and two `COMPLETE` batches. Its disabled saved job passes a direct query-only validation without saving. Exporting the two saved snapshots produces three `DATE CHANGED` rows. Replaying the latest saved run against the local SQLite source passes with 6,173 records, zero skipped rows, and zero deltas; case and SQLite SHA-256 values match before and after the read-only checks. See [SQL-CANDIDATE-01 evidence](evidence/sql-automation/SQL-CANDIDATE-01-readonly-revalidation-linux-aaf8ece.md). This verifies saved-case compatibility and current query reproducibility only.

**CSV exchange follow-up (2026-11-25, source `f6c5994`):** The latest SQL capture exported to CSV, received a generated import design, and re-imported into an empty disposable case with 6,173 records created, 0 skipped, and all endpoint names, relationship labels, source references, and notes retained. The original case and SQLite source were unchanged. See [SQL-CANDIDATE-02 evidence](evidence/sql-automation/SQL-CANDIDATE-02-csv-roundtrip-linux-f6c5994.md). This is Arch Notebook internal exchange acceptance; IBM receiving-application validation, completeness, human review, release approval, and Qwen analysis remain unverified.

Next work: improve analyst-facing vector organization for larger relationship graphs, then validate that layout on synthetic fixtures and inspect it in the running desktop application. Dataset completeness, human review, and Qwen analysis remain separate gates.

## Feature-branch increment: analyst vector organization

**Task / owner:** Improve chart organization for analyst use; orchestrator-owned implementation.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/analyst-notebook-parity`.
**Baseline:** `7a5155e` (`feat: report SQL capture application progress`).
**Scope:** Make entity-type layout use compact, deterministic clusters and make alphabetical-grid column count track the most recently rendered canvas aspect ratio. Pinned entity positions remain unchanged.
**Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `docs/build-acceptance.md`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Unit tests | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0; 4 passed, 0 failed. |
| Offline integration check and debug build | PASS | Offline `cargo check` and debug `cargo build` completed with exit 0. |
| Interactive chart review | BLOCKED | The changed layout controls were not exercised in a running GUI; no visual acceptance is claimed. |

The initial grid implementation used a 1.6 chart aspect estimate. It now uses the most recently rendered chart canvas aspect, with a 1.6 default before the first chart frame and a 0.75–2.5 clamp for narrow and ultrawide windows. The layout action runs before the current frame's chart allocation, so a resize affects the next layout invocation.

**Follow-up verification:** Entity-type position calculation is now an isolated pure function with a regression test covering deterministic name order, type grouping, and exclusion of pinned entities from the generated positions. `cargo fmt --all -- --check`, `git diff --check`, offline tests (5 passed, 0 failed), offline `cargo check`, offline debug build, and offline release build all exited 0. Interactive chart review remains BLOCKED.

**Aspect-ratio follow-up:** Source commit `8dfa690` makes alphabetical-grid dimensions use the last observed chart-canvas aspect. A regression test checks that wider canvases allocate more columns and fewer rows, empty input returns an empty grid, and invalid aspect values use the stable default. `cargo fmt --all -- --check`, `git diff --check`, offline tests (6 passed, 0 failed), offline `cargo check`, offline debug build, and offline release build all exited 0. The installed release binary matches SHA-256 `8021eb8e1232840977a1c625ad4f79964c15f44d3293e1cb6b384e0d94d2ec24`; both launcher files pass `desktop-file-validate`. Visual review remains BLOCKED because the session cannot open an X display.

**Local desktop installation:** The vector organization release executable was installed to the user's local application bin. The latest binary digest is recorded above; the installed executable reports version `0.1.0`. Both the application-menu entry and desktop shortcut pass `desktop-file-validate`. Installed icon assets match the repository SVG/PNG digests above. This verifies local file installation and metadata, not an interactive visual review.

## Feature-branch increment: Analysis Method Card

**Task / owner:** SNG-DELIVERY-001, active-chart Analysis Method Card; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline:** `008537813d60dd76db0732b06f87b6266a02603a`.
**Product source commit:** `a5208a0e3aa8c0fe3b3a1e3f02986e45e97b919a`.
**Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `docs/analyst-notebook-parity.md`.
**Scope:** Add a transient, read-only chart summary and current graph-method notes; copy only aggregate/method text; clear cached results on graph structure and relationship date/direction edits.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Rust formatting, diff, and unit tests | PASS | Format and diff checks exited 0; 25 offline Rust tests passed, 0 failed, 0 ignored. |
| Offline check and debug/release builds | PASS | All three locked offline commands exited 0. Debug SHA-256 `dfab5049a21a08063a3bf244bcd16d164d51aed516d01d7f18e3034d1697c38d`; release SHA-256 `765f8fa2e6384a2df4747890a9b954920a1668dc35557068f749f89afcea1fc7`. |
| Local Linux install and launcher/icon sync | PASS, scoped | Installed binary matches release hash; menu and desktop entries compare byte-for-byte and both validate; MIME handler resolves to `arch-notebook.desktop`; release version is `0.1.0`; installed icon and automation-installer hashes match the repository. |
| Fictional method-card metrics and copy privacy | PASS, unit-tested | Fixture covers isolates, directed/undirected records, self and repeated/opposite-direction pairs, missing endpoints, and valid/blank/invalid dates. Copied text omits chart title, names, IDs, relationship labels, source IDs, notes, and evidence; case serialization remains unchanged. |
| Native method-card interaction and screenshot review | BLOCKED | The new installed window could be launched, but the compositor capture bridge failed and pointer/keyboard input is unavailable. The older UI-00 screenshot predates this feature. |
| Independent review provenance | NOT VERIFIED | The roadmap has prior code/security review statements, but no separate handoff/report artifact is checked in; this session did not perform independent review. |
| IBM visual or XSD/application validation | BLOCKED / not run | No IBM application or schema was used; this increment makes no IBM parity claim. |

See [UI-ANALYSIS-CARD-01 evidence](evidence/vector-ui/UI-ANALYSIS-CARD-01-linux.md) and the [phased implementation roadmap](roadmaps/social-network-graphing-roadmap.md). The native and independent-review gates remain open; this is not a release-readiness result.
