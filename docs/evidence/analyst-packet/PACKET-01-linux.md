# PACKET-01: Linux analyst finding-packet export

- **Task / owner:** `SNG-DELIVERY-004`, Arch Notebook local analyst handoff; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `25b1fc4c79a2b85c1daa01702113bf3bc8245024`; candidate source commit: `bb5443ef27b4be03f35de58a9d07e000627e5ccb` (feature `84ddfd3`, followed by failed-staging cleanup hardening).
- **Product:** Rust CLI export of a persisted chart as `finding.md` and `network.svg`.
- **Platform scope:** Linux only. Non-Linux packet publication returns an explicit unsupported-platform error; macOS was not cross-compiled or run.

## Acceptance results

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting and patch validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Rust regression suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0: 27 passed, 0 failed. Covers default redaction, opt-in escaping, selected-chart output, deterministic SVG output, ambiguous chart titles, missing endpoints, broken symlink refusal, no-overwrite behavior, and private Unix output permissions. |
| Offline check and builds | PASS | Locked offline `cargo check`, debug build, and release build all exited 0 using `/tmp/arch-notebook-target`. |
| Release CLI acceptance | PASS | `python3 scripts/test-analysis-packet-acceptance.py --binary /tmp/arch-notebook-target/release/arch-notebook --fixture fixtures/chart/correctness.case.json --work-dir /tmp/arch-notebook-packet-acceptance-20261126-05` exited 0. The fixture has 8 entities and 8 relationships; a separate selected chart returned 1 entity and 0 relationships. |
| Rendered SVG inspection | PASS, implementation-owner visual check | `rsvg-convert` produced a 1200×800 PNG. The overview has distinct lanes for parallel links, direction arrows, a self-loop, bounded/truncated entity labels, and a legend. Full entity text and relationship captions remain available in SVG tooltips. PNG SHA-256: `390ff37c46ba668f1597e50c6d7fac13e9eefd527424acd4bb0beb08bd2985a1`. |
| Independent code/security review | BLOCKED / not evidenced | This session performed implementation-owner review only. No independent reviewer handoff or report was available. |
| IBM/native acceptance | BLOCKED / not run | No IBM XSD, Analyst's Notebook, or Chart Reader validation was performed. This is a local report handoff, not ANX interoperability evidence. Native UI interaction was not tested. |

## Artifact identity

- Release binary: `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `d02aca2aa4f59a229f7f12c533ca8bcb9abd7782d2d94ec7d4ff43eca5fc99c4`.
- Acceptance JSON: `/tmp/arch-notebook-packet-acceptance-20261126-05/acceptance-report.json`, SHA-256 `e9385a1b42d92f971d358a17b296bba6a86853e82406fc88101791d0816b8682`.
- Input fixture: `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f` (unchanged by the run).

The acceptance report confirms that the source case copy and checked-in fixture were unchanged; entity names, relationship labels, IDs, source references, and chart title were omitted by default; explicit options disclosed only the selected classes; the SVG parsed as XML and retained all 8 relationship records, 5 directed markers, and 1 self-loop; duplicate `--json` was rejected with exit 2; an existing destination was rejected with exit 1 and preserved; and output permissions were `0700` for the packet directory and `0600` for both files. `rustix` is scoped to the Linux target because its atomic no-replace rename is used only there.

## Product behavior and limits

The command exports the saved active chart or a selected chart by title/UUID. It includes all persisted chart entities and relationships and ignores unsaved GUI selection, search, and timeline filters. It refuses missing relationship endpoints, ambiguous chart titles, and existing output paths. Linux publication uses staged files and atomic no-replace rename. The case file is read only. By default the report redacts the chart title and the SVG uses pseudonymous labels; users opt into names, relationship labels, IDs, and evidence-reference strings independently. Notes, evidence contents, database paths, and the case title are never exported.

The SVG is a compact overview based on stored positions, not a measure of social distance or a replacement for the interactive chart. Dense-network label collisions remain possible. No analyst findings are generated. Independent review, native Automation/UI interaction, IBM schema/application validation, and non-Linux publication remain open.
