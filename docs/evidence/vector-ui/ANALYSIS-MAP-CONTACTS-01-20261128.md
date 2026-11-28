# ANALYSIS-MAP-CONTACTS-01 — ranked contact actions fit the analysis pane

- **Task / owner:** Correct and accept the Most connected contact-map controls in the native chart analysis pane / implementation session.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-nested-conditions` / local source baseline `5e2cf7fad4c9928eae4621e09db564edc6464abb`.
- **Scope:** Analysis-pane ranked entity rows and their contact-map action; no case schema, SQL behavior, or graph analysis semantics changed.
- **Fixture:** Disposable fictional SQL acceptance case with five entities and four relationships. No Clinton corpus or operational data appears in the screenshots.

## Change

The previous row put the full `Map contacts · <entity name>` control alongside the ranked entity label. In a 320 px analysis pane it ran past the pane edge and obscured the tail of longer entity labels. The action now reads `Map contacts`; its tooltip retains the selected entity name and action description. The ranked label omits redundant entity type, reserves width for the count/action controls, and truncates inside its own button while keeping the full label available on hover.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Reproduced narrow-pane defect | PASS | Before screenshot shows overflowing action text and clipped ranked labels at a 1440×900 application window. The fixture contains only fictional names. |
| Patched native render at default pane width | PASS | After screenshot shows all five ranked labels and all five `Map contacts` actions contained within the pane. Window: 1440×900 on a 2560×1440 desktop. The app was launched with the patched debug binary and isolated XDG data. |
| Minimum 240 px pane layout | PASS, headless | `headless_analysis_filter_controls_stack_and_fit_default_pane` fills long entity names, exercises default and minimum pane widths, and checks ranked label/action geometry remains inside the rendered clip. |
| Contact action behavior | PASS, headless | `headless_ui_drives_geodesic_and_detected_community_actions` clicks the top-ranked row's `Map contacts` action and verifies the topology analysis opens for that entity. |
| Full Rust workspace tests | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --workspace --locked --offline`: 81 passed, 0 failed, 3 ignored. |
| Formatting, workspace check, debug build | PASS | `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --workspace --locked --offline`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --workspace --locked --offline` exited 0. `git diff --check` exited 0. |
| Optimized release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --workspace --release --locked --offline` exited 0. Release SHA-256: `7a3dace1f04c4be70522b582c1d12a27cab47c4c85c60faff88bcab905abaf20`. |
| Isolated launcher/package acceptance | PASS | `python3 scripts/test-linux-user-install-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --report /tmp/arch-notebook-map-contacts-install-acceptance-final.json` exited 0; desktop/menu entries, synchronized SVG/PNG icons, MIME registration, and SQL helper all matched. |
| Current-user release install | PASS | `scripts/install-linux-user.sh /tmp/arch-notebook-map-contacts-release-final/arch-notebook-linux-x86_64` exited 0. Installed executable SHA-256 matches the optimized release. The menu entry and Desktop shortcut validate and match byte-for-byte; installed SVG and PNG hashes match repository art. |
| Installed-release SQL automation acceptance | PASS | `scripts/test-sql-automation-acceptance.py` against the current installed release passed two snapshots, expected add/remove/confidence/source-note deltas, schema-drift rejection with structured missing-column diagnosis, reviewed query repair, failed-run retention, and CSV/TSV round trips. The SQLite source remained unchanged during read-only capture/preflight/failure operations. Full row-level harness output remains in `/tmp/arch-notebook-native-sql-qa/`. |
| Native minimum-width screenshot / pointer interaction | BLOCKED | The X display stopped accepting the capture/input path before the 240 px native capture. The 240 px geometry and click behavior passed in the headless production UI. No native pointer action is claimed. |
| IBM visual or schema compatibility | NOT TESTED | No IBM Analyst's Notebook application or vendor XSD was used. |

## Artifacts

- Before: [ANALYSIS-MAP-CONTACTS-01-before-20261128.png](artifacts/ANALYSIS-MAP-CONTACTS-01-before-20261128.png), SHA-256 `b6169ef4b01fccf3ce6a4c1bcc2052dab849d22977d4e2087c0b5a8d8e68461e`.
- After: [ANALYSIS-MAP-CONTACTS-01-after-20261128.png](artifacts/ANALYSIS-MAP-CONTACTS-01-after-20261128.png), SHA-256 `5fcb1c0bb9c6bec1df93c4f7088a88fb1971d3d0e526152c9818bc4b2a8a1873`.
- Isolated installer report: `/tmp/arch-notebook-map-contacts-install-acceptance-final.json`.
- SQL harness aggregate: `/tmp/arch-notebook-native-sql-qa/installed-release-harness-output.json`.
- Previous installed executable rollback copy: `/tmp/arch-notebook-native-sql-qa/rollback/arch-notebook.before-final`, SHA-256 `5feb6e64b7d357b3a983d50dc47b0494e5c212e4e2f61b3d4343f8469d955ca2`.

## Limits and next action

This change fixes the observed layout issue and retains the existing contact-map behavior. It does not complete the remaining native SQL interaction sequence, live scheduled firing, broad IBM interoperability, or visual comparison against IBM's application. Continue with native Automation workspace review and vendor-backed ANX/import validation when those environments are available.
