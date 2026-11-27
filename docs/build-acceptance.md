# Local build acceptance record

## Current topology renderer verification — 2026-11-27

**Repository / branch / source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `a9f56cced2ec12c2f0f787520942939a7603b006`.
**Candidate:** current-source Linux debug application, SHA-256 `74c900167f450e28f391506d043bfb17fe177382afe19d6f8fdd16bcab24e606`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Offline Rust suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0; 69 passed, 0 failed, 1 opt-in benchmark ignored. |
| Current-source Linux debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-current-target cargo build --locked --offline` exited 0. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native initial render on synthetic chart | PASS, render-only | Current candidate rendered all 30 entity cards inside the chart viewport with dense Auto relationship text suppressed; [G4-DENSE-LABEL-03](evidence/vector-ui/G4-DENSE-LABEL-03-current-topology-render-20261127.md). |
| Clinton corpus SQL and redacted graph packet on current-source binary | PASS | Read-only preflight and latest-run replay pass for 6,173 rows with zero skipped rows/deltas; packet renders all 2,813 entities with default redaction and no out-of-viewport labels. Inputs remained unchanged; [CLINTON-CORPUS-QA-02](evidence/sql-automation/CLINTON-CORPUS-QA-02-current-debug-20261127.md). |
| Clinton PDF text-layer coverage and Qwen | NOT ESTABLISHED / BLOCKED | `pdftotext` returned nonempty text for all 7,121 pages, but header fields on 4,658 pages do not establish unique email extraction or semantic completeness. Local Ollama loopback and service-bus access are denied; no remote inference was attempted. See [CLINTON-CORPUS-EXTRACT-01](evidence/sql-automation/CLINTON-CORPUS-EXTRACT-01-text-layer-coverage-20261127.md). |
| Current-source ANX round-trip and safe rejection | PASS, Arch Notebook subset | Current debug binary round-tripped the fictional 8-entity/8-relationship fixture and rejected unsupported-shape and DTD inputs without changing the case. IBM XSD/application compatibility remains NOT TESTED; [ANX-ROUNDTRIP-03](evidence/anx/ANX-ROUNDTRIP-03-current-debug-20261127.md). |
| Native interaction, Clinton visual review, renderer performance, IBM application parity | NOT TESTED | No native input/performance measurements, real corpus capture, or IBM application/schema validation in this run. |
| Exact-head cross-platform CI | PASS | Manual workflow run [46059559135](https://github.com/pskeffington/arch_notebook/actions/runs/46059559135) completed successfully for `ab6e29318fca406d748f6694c89b3e25cd06b347`: Linux tests/SQL/ANX/installer, Linux x86_64, Linux ARM64, Arch package, macOS Intel, and macOS Apple Silicon. |
| Hop-group contact direction rendering | PASS, headless UI | Reciprocal directed group contacts render opposing arrowheads; undirected ties remain unarrowed. Full Rust suite and debug build pass; exact-head CI [46062451089](https://github.com/pskeffington/arch_notebook/actions/runs/46062451089) passes all six jobs. Native visual review remains open; [PROXIMITY-MAP-DIRECTION-01](evidence/vector-ui/PROXIMITY-MAP-DIRECTION-01-linux.md). |
| Hop-group map legend and node metrics | PASS, headless UI and exact-head CI | Legend explains node-size and relationship-count line-width encodings; node label distinguishes internal records, and hover discloses exact group metrics. The exact candidate passed all six CI jobs in [run 46064081779](https://github.com/pskeffington/arch_notebook/actions/runs/46064081779); native visual review remains open; [PROXIMITY-MAP-LEGEND-01](evidence/vector-ui/PROXIMITY-MAP-LEGEND-01-linux.md). |
| Hop-group map row spacing | PASS, measured geometry and headless UI | 52 px row pitch is checked against egui's actual label galley height and maximum node radius, with at least 4 px required clearance. Full suite (69 passed, 1 ignored), warning-free locked debug build, formatting, and diff checks pass; native visual review remains open; latest source CI awaits exact-head results; [PROXIMITY-MAP-SPACING-01](evidence/vector-ui/PROXIMITY-MAP-SPACING-01-linux.md). |
| Hop-group contact-line preview, selection, and record navigation | PASS, headless UI | Hover previews exact direction, pair, and record counts in a reserved status line; clicking selects the nearest interaction, opens exact-record detail, and highlights the tie. Group-node click-to-focus remains intact. Full suite/build/format checks pass. Native pointer/contrast review and latest exact-head CI remain open; [PROXIMITY-MAP-PICK-01](evidence/vector-ui/PROXIMITY-MAP-PICK-01-linux.md). |
| Topology metric label grammar | PASS, headless UI | Singular and plural entity, record, endpoint-pair, group, and hop-layer counts are rendered with matching nouns in map labels and summaries. The focused production UI regression verifies singular group and interaction labels. Full suite: 69 passed, 0 failed, 1 ignored; debug build and formatting/diff checks pass. Exact-head CI and native visual review are pending. |

The native screenshot resolves the left-edge clipping observed in the prior candidate, for this synthetic startup view only. It does not close G3/G4 native interaction or broader UI-00 acceptance.

## Analysis pane filter readability — 2026-11-27

**Task / owner:** Improve chart-analysis navigation at the default pane width; implementation session.
**Repository / branch / baseline / product source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `487473d0ff655e0b9fc0584fcd8dd59a7e320dbb` / `6dd04e4bfd16e3bb14eafa7b7634474ed3a51322`.
**Candidate binary SHA-256:** `766229457ca4de9975ace1a044bdde1e8802b6bc9ccd4f5bb60bdf904a83274f`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Unicode-safe long label handling | **PASS** | `analysis_filter_label_truncation_is_unicode_safe_and_stable` covers short values, exact 20-character limit, and Unicode character boundaries. |
| Default-width analysis-pane control layout | **PASS, headless UI** | `headless_analysis_filter_controls_stack_and_fit_default_pane` verifies separate rows, width-scaled clipping, and full-value hover disclosure at 1440×900/default 320px pane width and 1000×650/minimum 240px pane width, including wide Unicode labels. |
| Full Rust suite | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 60 passed, 0 failed. |
| Locked compile and builds | **PASS** | `cargo check --locked --offline`, debug build, and release build exited 0 with `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target`. |
| Formatting and diff validation | **PASS** | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Per-user launcher sync | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-sync-ui-filters/arch-notebook-linux-x86_64` exited 0; installed executable matches the release SHA above, desktop entries validate and match, and installed icon and helper files match repository assets. |
| Exact-candidate native process/window creation | **PASS, scoped** | The exact launch command remained live; `xwininfo -root -tree` listed an Arch Notebook X11 window at 1440×900. |
| Exact-candidate native visual review | **BLOCKED** | `spectacle --new-instance --activewindow --background --nonotify --no-decoration --output /tmp/arch-notebook-analysis-filters-6dd04e4.png` aborted with exit 134. No current-candidate screenshot or visual review is claimed. Host input injection also remains unavailable. |
| GitHub Actions for product changes | **PASS, ancestor candidate** | Manual workflow run [45934212852](https://github.com/pskeffington/arch_notebook/actions/runs/45934212852) completed successfully for `61fb5ebbe731761b9f53e29c0a3f8c8429e18b9e`, which includes the UI filter product changes. It predates the benchmark test/script and this evidence-only update; those newer changes are not covered by that run. |

Detailed source semantics, headless assertions, host limitation, and residual gates: [UI-ANALYSIS-FILTERS-01](evidence/vector-ui/UI-ANALYSIS-FILTERS-01-linux.md).

## Headless egui CPU baseline — 2026-11-27

The eight-case 100/1,000-entity synthetic workload benchmark completed successfully. It records full-app frame-build and egui CPU tessellation separately; the normal offline Rust suite reports 60 passed, 0 failed, and one opt-in benchmark ignored. The matched route-culling A/B shows 20.9–45.1% lower frame-build medians and 29.8–77.3% lower CPU tessellation medians on offscreen-panned synthetic charts; neutral-view shape counts are unchanged. GPU/presentation results remain unmeasured. See [headless CPU baseline](evidence/vector-ui/G4-EGUI-TESSELLATION-01-linux.md), [matched culling evidence](evidence/vector-ui/G4-CULLING-AB-01-linux.md), and [raw paired results](evidence/vector-ui/artifacts/G4-culling-ab-20261126.json).

## Analysis packet CLI output privacy — 2026-11-27

The JSON and human-readable success summaries no longer echo the caller-supplied packet output directory. The full offline suite passed (61 passed, 0 failed, 1 opt-in benchmark ignored), the locked release build passed, and the packet acceptance harness passed redaction, selected-chart, source immutability, SVG, failure-safety, permission, and stdout path-omission checks. Exact-head cross-platform CI passed in [run 45970460460](https://github.com/pskeffington/arch_notebook/actions/runs/45970460460). See [UI-ANALYSIS-PACKET-OUTPUT-PRIVACY-01](evidence/vector-ui/UI-ANALYSIS-PACKET-OUTPUT-PRIVACY-01-linux.md) and its [acceptance report](evidence/vector-ui/artifacts/UI-ANALYSIS-PACKET-OUTPUT-PRIVACY-01-linux.json).

## Conditional formatting in chart templates — 2026-11-27

Chart templates now carry up to 96 conditional attribute-formatting rules and apply them through bounded, visible template-owned legend blocks. The full suite (63 passed, 1 opt-in benchmark ignored), locked check/debug/release builds, and current-user launcher/icon sync pass. The complete template-ownership follow-up passed exact-head cross-platform CI in [run 45988117371](https://github.com/pskeffington/arch_notebook/actions/runs/45988117371). The prior native capture showed left-edge clipping; a current-source synthetic startup render shows all 30 cards inside the viewport, while template-generated legend interaction and the real Clinton chart view remain untested. See [CHART-TEMPLATE-CONDITIONAL-RULES-01](evidence/vector-ui/CHART-TEMPLATE-CONDITIONAL-RULES-01-linux.md), [prior G4 render](evidence/vector-ui/G4-DENSE-LABEL-02-native-render-20261127.md), and [current G4 render](evidence/vector-ui/G4-DENSE-LABEL-03-current-topology-render-20261127.md).

## Current-user Linux Desktop shortcut — 2026-11-27

The Linux per-user installer now creates an executable Desktop shortcut byte-identical to the application-menu entry. An isolated temporary-HOME smoke passes for executable, desktop files, icons, SQL helper and MIME definition; the current-user release and icon assets are synchronized. Exact-head cross-platform CI passed in [run 45988117371](https://github.com/pskeffington/arch_notebook/actions/runs/45988117371), including the installer smoke and current application candidate. A later current-source debug candidate launched and rendered a synthetic chart, but native interaction and launcher-to-current-source synchronization remain unverified. See [LOCAL-INSTALL-08](evidence/deployment/LOCAL-INSTALL-08-linux.md) and [current topology render](evidence/vector-ui/G4-DENSE-LABEL-03-current-topology-render-20261127.md).


## SQL automation and scale recheck — 2026-11-27

**Task / owner:** Revalidate the installed release's SQL automation, recovery, interchange, and joined-header scale paths; implementation session.
**Repository / branch / verification source:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `282f9de3a755557c856b16f50145d9f37936e882`.
**Runtime source / binary:** product behavior from `e27ad5e842dd1c6d503f05c51844f12790502fa6`; installed release SHA-256 `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.

| Gate | Result | Evidence |
| --- | --- | --- |
| SQL automation, failure retention, diagnosis/repair, batch recovery, CSV/TSV interchange | PASS | Current installed release harness exited 0; expected failure statuses were retained and verified. |
| Joined-header scale | PASS | 7,121 synthetic records, 1,500 entities, 4,121 repeated endpoint pairs, two snapshots, zero skipped rows; source remained unchanged during captures. |
| Exact-source workflow status | NOT RUN | Actions API returned zero runs and commit status had no contexts for `282f9de`. |
| Production schema, native Automation UI, live scheduler, IBM validation | NOT TESTED | These require separate production-shaped inputs, native host interaction, scheduler execution, or IBM schema/application evidence. |

Commands, fixture descriptions, result hashes, and scope limits: [SQL-AUTO-07](evidence/sql-automation/SQL-AUTO-07-current-installed-release-linux.md).

## SQL scheduler runner recheck — 2026-11-27

**Verification source:** `179aff68d32e59e49f411a18f17e90144903712d`; installed runtime binary SHA-256 `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Generated Linux systemd service invocation | PASS, stubbed | Five cadences and scheduled capture/CSV invocation passed with a temporary `systemctl` stub. |
| Generated macOS LaunchAgent runner | PASS, stubbed on Linux | Five cadences, rolling UTC window, removal and rollback passed with a temporary `launchctl` stub; native macOS host untouched. |
| Live systemd/launchd delivery | NOT TESTED | Stub execution does not establish live scheduler dispatch. |

Exact commands, fixture boundaries, report hashes, and limits: [SQL-SCHED-07](evidence/sql-automation/SQL-SCHED-07-current-installed-release-linux.md).

## Current synchronized branch verification — 2026-11-26

**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source commit:** `d0ce0c77e9fcee445536d0f55aceebf70e5fbd84` (adds a headless viewport paint-submission regression to the `e27ad5e` product candidate).
**Release binary SHA-256:** `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Offline Rust suite, check and formatting | PASS | 57 tests pass, including `headless_chart_paint_culls_offscreen_relationship_labels`; `cargo check --locked --offline`, formatting, and diff validation exit 0. |
| Debug and release builds | PASS | Locked offline builds exit 0. |
| Remote CI for exact source commit | NOT RUN | GitHub check-runs endpoint for `e27ad5e` returned zero checks. The paint-submission test commit `d0ce0c7` has not yet been checked remotely. Local acceptance is recorded separately. |
| Current-user desktop/launcher install | PASS, scoped | Installed binary matches the release artifact. Menu entry and Desktop shortcut validate and compare byte-for-byte; MIME registration, automation helper, SVG and PNG match. See [LOCAL-INSTALL-04](evidence/deployment/LOCAL-INSTALL-04-linux.md). |
| Native window interaction and visual review | NOT RUN | `wayland-info` returns `failed to create display: Operation not permitted`; install checks do not establish a rendered or interactive window. |
| IBM ANX/XSD/application validation | NOT TESTED | The IBM schema and application were unavailable; internal interoperability checks do not replace vendor validation. |

## Latest pushed verification source — 2026-11-26

**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source commit:** `a2739d0114e742fd17d18866eb6fcbd250857ba1` (refines the headless regression to count offscreen `CONTACTS` label shapes).
**Product release binary SHA-256:** `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Exact-source offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0; 57 passed, 0 failed. |
| Formatting and whitespace | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Remote GitHub Actions and commit statuses | NOT RUN | Actions API returned `total_count: 0`; combined commit status returned an empty status list for this exact SHA. |
| Native UI and vendor compatibility | NOT TESTED | Native visual review and IBM ANX/XSD/application validation remain separate open gates. |

The change refines test coverage only; the optimized application binary remains byte-identical to the installed `4a373a…` release. Product behavior and install evidence are recorded under [LOCAL-INSTALL-04](evidence/deployment/LOCAL-INSTALL-04-linux.md), [COMMUNITY-PROXY-01](evidence/vector-ui/COMMUNITY-PROXY-01-linux.md), and [G4-VIEWPORT-CULLING-01](evidence/vector-ui/G4-VIEWPORT-CULLING-01-linux.md).

## Previous synchronized branch verification — 2026-11-26

**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source commit:** `e86814675eaee96ed6f20caff94d6d5b79b9a2c5`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Offline Rust suite and source check | PASS | 54 tests, formatting, diff validation, and offline `cargo check` pass for the graph-layout comparison change. |
| Release build and G0/G3 layout comparison | PASS, CPU only | Release build and 12 paired sparse/hub/disconnected/dense cases at 1,000–10,000 entities; [G3 release evidence](evidence/vector-ui/G3-connectivity-vs-force-release-20261126.md). Native arrange/restore remains unverified. |
| Current-release SQL automation and scale | PASS, synthetic | Current release passes capture, recovery, schema adaptation, CSV/TSV exchange, and 7,121-record joined-header scale checks; [SQL-AUTO-06](evidence/sql-automation/SQL-AUTO-06-current-release-linux.md). Live scheduling and native Automation UI remain unverified. |
| Local desktop/launcher installation | PASS, scoped | Binary, menu/desktop entries, MIME default, icons, and automation helper synchronized to the current build; [LOCAL-INSTALL-02](evidence/deployment/LOCAL-INSTALL-02-linux.md). Native window launch was not tested because the display is unavailable. |
| Independent review of the new Arrange/Restore test | NOT VERIFIED | This continuation performed implementation-owner review only; no separate reviewer handoff is checked in. |
| IBM ANX/XSD/application validation | NOT TESTED | The IBM schema and application were unavailable; internal interoperability tests are not a substitute. |

## Current-user desktop and launcher installation — 2026-11-26

**Task / owner:** Install the proximity-analysis candidate on this machine and synchronize its executable, application-menu entry, desktop shortcut, and icon art; implementation session.
**Product source commit:** `6b59e32524c1ee487394ca1235cef4716d8937ae`.
**Installed release binary SHA-256:** `bd80d18f428655b095484d2f84ddd29ce5c4d369b6169e77d6040766b11d6309`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Per-user executable, installer helper, menu entry, MIME association | PASS | Installed under `/home/sysadmin/.local`; binary hash matches the release artifact; registered case MIME handler is `arch-notebook.desktop`. |
| Desktop shortcut and launcher icon synchronization | PASS | `/home/sysadmin/Desktop/Arch Notebook.desktop` matches the menu entry byte-for-byte; SVG and high-resolution PNG match repository icon hashes. |
| Native application window | BLOCKED | Winit returned `NoCompositor`; X11 was unavailable. Installation checks do not establish a rendered or interactive window. |

Detailed paths, checks, and hashes: [LOCAL-INSTALL-02 evidence](evidence/deployment/LOCAL-INSTALL-02-proximity-linux.md).

## Origin-centered contact groups and proxy ranking — 2026-11-26

**Task / owner:** Combine selected-origin hop layers with contact groups, group-to-group record summaries, and direct cross-group proxy ranking; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline:** `941373e3b8fa537c7ee922c14ab0435e488a3009`.
**Product source commit:** `6b59e32524c1ee487394ca1235cef4716d8937ae`.
**Release binary SHA-256:** `bd80d18f428655b095484d2f84ddd29ce5c4d369b6169e77d6040766b11d6309`.
**Changed paths:** `src/proximity.rs`, `src/main.rs`, `docs/notebook-workflows.md`, `docs/analyst-notebook-parity.md`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting, diff validation, full Rust suite | PASS | Formatting and diff checks exited 0; 50 tests passed, 0 failed. |
| Locked offline compile and optimized release build | PASS | Both exited 0. |
| GitHub Actions for source commit | PASS | [Run 45909562127](https://github.com/pskeffington/arch_notebook/actions/runs/45909562127) completed successfully for the exact candidate SHA; Linux test/fixture, both Linux architectures, both macOS architectures, and Arch package jobs passed. Release attachment was skipped because no release was published. |
| Current-candidate native chart review | BLOCKED | Wayland launch failed with `NoCompositor`; no visual or pointer acceptance is claimed. |
| IBM application compatibility | NOT TESTED | No IBM application or XSD was used. |

See [origin-centered proximity groups evidence](evidence/vector-ui/PROXIMITY-GROUPS-linux-6b59e32.md) for the calculation, fixtures and limits. The follow-up [group contact map acceptance](evidence/vector-ui/PROXIMITY-MAP-01-linux-ffdd781.md) records the bounded visual overview, navigation behavior, current-user installation, local build results and remaining native UI gate. The main chart continues to draw its original entity and relationship records.

## Origin-centered group contact map — 2026-11-26

**Task / owner:** Add a bounded group-level contact overview to the proximity workflow; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source commit:** `26fe459dee702ab6c41da483f7f29d267c60fed7`.
**Release binary SHA-256:** `b6955b38b8af4596e0d88a9667d7d60637ffdd7536c63a806d2d8334e648172f`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Offline Rust tests, compile and optimized build | PASS | 51 tests passed; locked offline check and release build exited 0. |
| Local executable, desktop launcher and icon synchronization | PASS | Installed binary matches release hash; launcher entries validate; repository and installed icon hashes match. |
| GitHub platform/package matrix | PASS | [Run 45914257016](https://github.com/pskeffington/arch_notebook/actions/runs/45914257016) passed Linux tests/fixture determinism, both Linux architectures, both macOS architectures, and Arch packaging for the exact source commit. Release attachment was skipped because this was not a release event. |
| Native visual/input review | BLOCKED | Current Wayland diagnostic returned `Operation not permitted`; no native appearance or pointer acceptance is claimed. |

The bounded map is in the Analysis pane. It samples at most 120 groups and 40 hop layers on large cases; the complete group and exact interaction records remain listed below it. See the [detailed acceptance record](evidence/vector-ui/PROXIMITY-MAP-01-linux-ffdd781.md).

## UI-01 chart entity-card picking — 2026-11-26

**Task / owner:** Align entity picking and viewport transforms with the visible chart cards; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Baseline:** `0c5cf7bd59b1a66713e165943962354a1c7f6ae0`.
**Product source commit:** `e7b64653d798faeb858d5fb34c61bfd96a2db336`.
**Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `docs/evidence/vector-ui/UI-01-card-hit-geometry-linux.md`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting, diff validation, and unit tests | PASS | Formatting and diff checks exited 0; 48 offline Rust tests passed, including new transform round-trip, zoom-anchor, degenerate viewport, card-edge, and overlap-order tests. |
| Offline compile check | PASS | `cargo check --locked --offline` exited 0. |
| Strict Clippy | FAIL | Existing warning-as-error findings prevent a clean repository-wide Clippy run; no unrelated lint cleanup was included. See the detailed evidence record. |
| Current-candidate native chart interaction | BLOCKED | Desktop pointer/capture access is unavailable in this session; no visual acceptance is claimed. |
| IBM Analyst's Notebook compatibility | NOT TESTED | No IBM application or XSD was used. |

Full commands, results, and scope limits: [UI-01 card hit geometry evidence](evidence/vector-ui/UI-01-card-hit-geometry-linux.md).

## Synchronized-branch SQL automation recheck — 2026-11-26

**Task / owner:** Continue the Rust notebook build with emphasis on SQL automation, acceptance evidence, and synchronized progress; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source commit:** `8c08da9285c6104d35a66386676b244d44f1270e`; evidence baseline `991c4764715c71e682af784fbdbf9e2db3e833c7`.
**Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `1ef36e782956eaf1196a06d074ad8edf02591b45095d9acb805879b000cfe941`.

| Gate | Result | Evidence |
| --- | --- | --- |
| SQL capture, comparison, failure retention, reviewed schema adaptation, batch resume, CSV/TSV exchange | PASS | Current release automation harness passed; source SQLite remained unchanged during app captures and preflight. |
| Joined-header SQL scale | PASS | Fictional 7,121-row joined workload; two snapshots, 1,500 entities, composite identities, repeated pairs, zero skipped rows. |
| Linux systemd schedule generation and invocation | PASS, stubbed | Five cadences and generated service invocation passed using a temporary `systemctl` stub. |
| macOS LaunchAgent generation and runner simulation | PASS, stubbed on Linux | Five cadences, rollback, rolling-window output, and unchanged SQLite source passed using a temporary `launchctl` stub. |
| Native Automation UI | BLOCKED | App exits with Winit `Connection(NoCompositor)` despite compositor globals being visible to `wayland-info`; X11 is inaccessible. |
| Live scheduler dispatch and IBM application/schema validation | NOT TESTED | No live timer/LaunchAgent run or IBM XSD/application was used. |

Exact commands, fixture scope, and report hashes are in [SQL-AUTO-05](evidence/sql-automation/SQL-AUTO-05-current-branch-linux.md). This recheck updates the earlier SQL baseline for the synchronized release; it does not claim native UI, live scheduling, production-schema, or IBM interoperability acceptance.

## Chart viewport navigation — 2026-11-26

**Task / owner:** Improve analyst recovery from an over-zoomed chart; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source baseline:** `a356e0d06f9443aa3cb2f463392ebce8e7a16a18`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Fit chart access | PASS, source/test | Added **Fit chart** to Quick open search and `Ctrl/Command+0`; **View…** lists the shortcut. Quick-command search regression passed. |
| Rust suite and build | PASS | Focused command search: 1 passed; full suite: 34 passed; offline `cargo check` and optimized release build exited 0. |
| GitHub Actions for the pushed candidate | PENDING | GitHub Actions lookup found no PR-triggered run and no combined commit status contexts for `5cf1db5c573f69ce57f6322bb66bb2c958770de1`; no remote CI pass is claimed. |
| Current-candidate native interaction and Qwen review | BLOCKED | X display access failed, and the local Ollama endpoint refused inference requests. No current-candidate rendering, shortcut input, or Qwen review is claimed. |

Detailed evidence: [UI-NAV-FIT-01](evidence/vector-ui/UI-NAV-FIT-01-linux.md). The previous 1440×900 screenshot is contextual only; repeat visual and keyboard acceptance in an input-capable desktop session.

## Current-release SQL automation recheck — 2026-11-26

**Task / owner:** Revalidate SQL capture, recovery, scale and scheduler runners on the pushed route-geometry candidate; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source commit:** `fae574dc74af78ae19a950641ba2b14e652901c2` (branch HEAD `e51c83a` contains documentation updates only).
**Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `63d008c9d5431390663da1f58f803534908e3e5d4904d67054895b3391093088`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| SQL automation, diagnostics, recovery and chart interchange | PASS | Preflight, immutable capture/comparison, one-run override isolation, schema drift rejection, failed-run retention, reviewed schema repair, resumable batches, mismatch rejection, and CSV/TSV round-trip. |
| Joined-header scale ingestion | PASS | 7,121 records, 1,500 entities, 7,121 composite IDs, 4,121 repeated endpoint pairs, two snapshots, nullable properties, zero skipped rows. |
| Per-entity live SQL source lookup invariants | PASS, headless | Three focused tests and the 31-test suite cover multi-source composite identities, read-only behavior, unsafe/ambiguous query rejection, and bounded display; native inspector interaction remains pending. See [SQL-DRILLBACK-01](evidence/sql-automation/SQL-DRILLBACK-01-linux.md). |
| Linux scheduler generation and invocation | PASS, stubbed | Five cadences and generated service invocation passed; `systemctl` was a temporary stub. Live timer firing is not tested. |
| macOS LaunchAgent generation and runner simulation | PASS, stubbed | Five cadences and generated runner passed; `launchctl` was a temporary stub and the real macOS host was untouched. |
| Candidate GitHub Actions run | PENDING | Actions API returned zero runs for product source commit `fae574d`. |
| Native Automation UI and IBM XSD/application compatibility | BLOCKED / not tested | No native pointer-driven review or IBM application/schema validation was performed. |

Exact commands, fixture and binary hashes, report digests, and scope limits are in [SQL-AUTO-04](evidence/sql-automation/SQL-AUTO-04-current-release-linux.md). All harness data is fictional; the joined-header fixture contains no message bodies. Scheduler stubs establish generated runner behavior, not live host dispatch.

## Previous SQL automation acceptance — 2026-11-26

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

Historical commands, exit status, report hashes, and fixture caveats are recorded in [SQL-AUTO-02](evidence/sql-automation/SQL-AUTO-02-current-release-linux.md). This older acceptance record does not describe the current release binary above.

## Linux analyst packet acceptance — 2026-11-26

**Task / owner:** `SNG-DELIVERY-004`, local analyst finding-packet export; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Candidate source:** `bb5443ef27b4be03f35de58a9d07e000627e5ccb` (feature commit `84ddfd3`, followed by staging-cleanup hardening).
**Release artifact:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `d02aca2aa4f59a229f7f12c533ca8bcb9abd7782d2d94ec7d4ff43eca5fc99c4`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Format, Rust tests, offline check, debug/release builds | PASS | Formatting, `git diff --check`, 27 tests, locked offline check, debug build, and optimized release build all exited 0. |
| CLI acceptance on fictional data | PASS | Default redaction, explicit disclosures, 1-entity alternate-chart selection, source immutability, valid SVG, parallel/directed/self-loop semantics, no-overwrite, duplicate JSON rejection, and 0700/0600 output permissions passed. |
| Rendered SVG | PASS, implementation-owner inspection | `rsvg-convert` rendered a 1200×800 SVG with bounded labels, parallel lanes, direction cues, self-loop, and legend. This is a compact overview, not the interactive chart. |
| Independent code/security review | BLOCKED | No independent review handoff is present; implementation-owner review is recorded separately. |
| IBM application and macOS support | BLOCKED / not tested | IBM ANX XSD/application validation was not performed. Atomic packet-directory publication is Linux-only; macOS was not cross-compiled or run. |

Exact commands, report and artifact digests, and scope limits are in [PACKET-01](evidence/analyst-packet/PACKET-01-linux.md). This CLI handoff is separate from SQL automation acceptance above and does not establish IBM interoperability or native UI acceptance.

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

### UI-01 geometry and coincident-route follow-up

| Check | Result | Evidence |
| --- | --- | --- |
| Loop geometry, label separation, arrowhead placement, and selection regression | PASS | [UI-01 self-loop evidence](evidence/vector-ui/UI-01-self-loop-linux.md); focused and full offline Rust suites pass. |
| Distinct routes and exact picking for links sharing endpoint positions | PASS, source test | [UI-01 coincident-route evidence](evidence/vector-ui/UI-01-coincident-routes-linux-fae574d.md); all nine fixture routes differ and each route sample selects its exact record. |
| Independent code review | PASS | [UI-01 review handoff](evidence/vector-ui/UI-01-coincident-routes-review-linux.md); source route/picking review and disclosure wording recheck passed. This does not include security review. |
| Earlier self-loop candidate Linux render | PASS, render-only | Screenshot inspected at 1440×900; input was unavailable. It does not visually verify the coincident-route follow-up candidate. |
| Current-candidate native interaction and UI-00 baseline | BLOCKED | Current release launch stayed alive for the bounded smoke interval, but display capture and pointer/keyboard input are unavailable; repeated interaction and performance sampling remain open. |
| IBM application/schema interoperability | NOT TESTED | Requires validation in the IBM application/schema environment. |

The source changes repair self-link and coincident-position routes while preserving record-level picking. They do not close UI-00 or establish IBM visual parity. Continue with an input-capable native session and the established UI-00 sequence before treating UI-01 as natively accepted.

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

## SQL acceptance: synchronized branch automation and scale recheck

**Task / owner:** Revalidate SQL automation behavior on the current pushed feature-branch release; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Product source:** `750b6cb9f9214089678cc3341dae5ecd7c65005b`; release binary SHA-256 `f8c14cabd73164b27218068685d75c62ecd2e9d854287299b9aca534ec373032`.
**Scope:** Re-run the full synthetic SQL automation recovery workflow and 7,121-row joined-header scale harness against the release binary.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| SQL automation/recovery workflow | PASS | Query-only preflight, override isolation, schema drift diagnosis and reviewed repair, failed-run retention, batch resume, mismatch rejection, and CSV/TSV round-trips passed. The app left the SQLite source unchanged. |
| Joined-header SQL scale | PASS | 7,121 result rows, 1,500 endpoint entities, 7,121 distinct composite record IDs, nullable attributes, two snapshots, and deliberate confidence decline all passed; source unchanged during each capture. |
| Native GUI and live scheduled dispatch | BLOCKED | The graphical display rejected `wayland-info` and X11 access. Linux systemd and macOS LaunchAgent live dispatch were not exercised by these CLI harnesses. |
| IBM interoperability | BLOCKED | IBM XSD/application validation was not performed. |

Exact commands, exit codes, paths, and report digests are in [SQL-AUTO-03](evidence/sql-automation/SQL-AUTO-03-current-branch-linux.md). These automated synthetic results do not establish interactive, live-scheduler, production-schema, IBM-compatibility, or release acceptance.

## UI-00 acceptance: current fixture reproduction

**Task / owner:** Verify UI-00 fixture determinism and bounded workload generation; implementation session.
**Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
**Source commit:** `4fda60ad5d98caa198364ab1c4c40da8af8d1101`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Correctness fixture | PASS | Two generated copies matched each other and `fixtures/chart/correctness.case.json`; JSON parsing passed. 8 entities, 8 relationship records, fixture SHA matches the manifest. |
| Workload families | PASS | Sparse/hub/disconnected/dense at 1,000 nodes each reproduced byte-for-byte with 3,000/3,000/2,000/10,000 links. Dense 10,000-node output reproduced twice at the 100,000-link cap. |
| Native baseline and measurements | BLOCKED | Display access is denied. No interaction or rendering metric is claimed. |
| CI for current candidate | PENDING | Workflow source runs `cargo test` and generator checks, but no implementation PR run is associated with this candidate. Local reproduction does not substitute for CI. |

See [UI-00 fixture reproduction evidence](evidence/vector-ui/UI-00-fixture-reproduction-linux.md) for exact commands, hashes, and the corrected local summary assertion.

## SQL scheduler acceptance: current-release installer and runner recheck

**Task / owner:** Verify isolated Linux systemd and macOS LaunchAgent installation/runner paths with the current release; implementation session.
**Product source:** `750b6cb9f9214089678cc3341dae5ecd7c65005b`; release binary SHA-256 `f8c14cabd73164b27218068685d75c62ecd2e9d854287299b9aca534ec373032`.

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Linux unit installer / generated-service invocation | PASS, harness-scoped | Five cadences, failed-preflight rejection, special paths, case-specific removal, scheduled capture and CSV export passed under temporary `systemctl` stubs. |
| macOS LaunchAgent installer / generated runner | PASS, Linux harness | Five cadences, failed install/replacement rollback, special paths, scheduled capture and CSV export passed under temporary `launchctl` stubs. |
| Live host dispatch | NOT VERIFIED | No live systemd timer or native macOS launchd dispatch occurred. |

See [SQL-SCHED-06](evidence/sql-automation/SQL-SCHED-06-current-branch-linux.md) for commands, exact report hashes, and scope. These installer/runner harness results are separate from live scheduler acceptance.

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

## Linux SQL scheduler calendar parser — 2026-11-27

The Linux scheduler acceptance now asks `systemd-analyze calendar` to validate the five emitted cadence expressions, while retaining isolated unit-install and service-invocation checks. All five cadence expressions and the complete synthetic SQL scheduler harness pass. Exact-head cross-platform CI passed in [run 45988117371](https://github.com/pskeffington/arch_notebook/actions/runs/45988117371). Live timer delivery remains untested because the harness uses a stub and the current sandbox cannot access the user systemd bus. See [SQL-SCHED-08](evidence/sql-automation/SQL-SCHED-08-systemd-calendar-parser-linux.md) and its [acceptance JSON](evidence/sql-automation/artifacts/SQL-SCHED-08-linux.json).

## Primary Clinton corpus local QA — 2026-11-27

Use the real review-required Clinton candidate case as the primary local QA batch after each candidate build; use synthetic CI fixtures as supplemental coverage. The current Rust release passes the saved job's read-only query preflight and replays its latest saved run against the local SQLite source: 6,173 rows, zero skipped, zero added/removed/changed records. Default-redacted network analysis covers 2,813 entities, with 643 structural communities and 291 proxy candidates. Its report SVG maps the out-of-range saved coordinates into 704 rows (largest row: 4), with all labels in bounds. Entity drag geometry uses one map for nodes, routes, hit targets and navigator throughout a gesture. Saved bounds travel with the active chart, packet export and chart tabs; Fit all explicitly rebases the frame. Labels do not affect entity bounds. PDF, case, and database hashes are unchanged. No raw row-level content is stored in the report or Git. PDF extraction completeness, Qwen analysis, human content review, live scheduler delivery, native interaction on this case, and IBM validation remain open. See [CLINTON-CORPUS-QA-01](evidence/sql-automation/CLINTON-CORPUS-QA-01-linux.md) and its [aggregate-only report](evidence/sql-automation/artifacts/CLINTON-CORPUS-QA-01-linux.json).
