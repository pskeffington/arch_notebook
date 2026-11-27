# PROXIMITY-MAP-CLINTON-UI-01 — real-corpus topology in the production headless UI

- **Task / owner:** Exercise the proximity topology view at retained Clinton-corpus scale; implementation session.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-baseline` / `ae7dc852446caeed87738dabcbdd976668a950a1`.
- **Candidate source:** recorded after commit; product implementation is `8e796bc8794e4865a3ec509703ae7ca3d8b581f4`.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Production Analysis-pane UI on retained Clinton case | PASS | `clinton_case_topology_map_renders_at_real_corpus_scale` clicks the production “Measure hop layers” action, scrolls the Analysis pane until the sampled topology is visible, and checks rendered headings and selected-group details at 1440×900. It confirms 2,813 entities, 6,173 relationships, 2,128 reachable entities from the deterministic highest-degree entity, 10 hop columns, and the 48-group sample bound. |
| Complete group index | PASS | The UI candidate’s `expanded_map_interaction_indices` has one row per analyzed group; the structural Clinton test also confirms every interaction is indexed at both endpoints. |
| Command | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target ARCH_NOTEBOOK_CLINTON_CASE_PATH=/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json cargo test --locked --offline clinton_case_topology_map_renders_at_real_corpus_scale -- --ignored --nocapture` exited 0: 1 passed. |
| Data safety | PASS | The test reads the local case JSON and creates the app’s headless context in memory. It does not invoke save or SQL automation and does not modify the retained case/database. |
| Exact-head CI | PASS | Workflow run [46073375126](https://github.com/pskeffington/arch_notebook/actions/runs/46073375126) for source `d3ee05fda2277e76ed9a5cf66cb2e7b9408ee2c3` passed Linux tests, Linux x86_64, Linux ARM64, Arch package, macOS Intel, and macOS Apple Silicon. The added Clinton-only test compiled under CI but was not executed there because its fixture is local-only; its local focused run passed separately. |
| Native rendering/input review | BLOCKED | X11 access failed with `unable to open display ":0"`; Wayland access failed with `Operation not permitted`. Headless output does not establish native appearance or pointer feel. |

This is structural UI coverage, not semantic review of the Clinton records, IBM compatibility validation, or a performance benchmark. No names or entity IDs are included in the test output or evidence.
