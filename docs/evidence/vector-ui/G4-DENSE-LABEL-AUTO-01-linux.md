# G4-DENSE-LABEL-AUTO-01 · dense chart relationship labels

- **Task / owner:** SNG-DELIVERY-002 G4 readability follow-up; implementation session.
- **Baseline:** `6c283ba63df3abc9570854f440b3f5e6695c0517` on `codex/ui-00-chart-baseline`.
- **Candidate source:** `993dd560efca942fc4cc4c021d2fbefab1c96eef`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`; SHA-256 `d28f2420fa51680046764546af22be7a850827cc615873eb1633267aea1f3676`.
- **Review date:** 2026-11-27.
- **Scope:** Reduce relationship-label collisions in dense chart overviews while retaining route drawing, exact-record picking, and access to every record.

## Finding and change

The operator's already-running Arch Notebook review screen was open on fictional case `VECTOR UI SPARSE 30` (30 entities, 90 relationship records). The captured graph view showed many relationship labels stacked through the chart center and several entity cards clipped at the chart viewport edges. The image does not establish card-to-card overlap, nor whether the clipping came from a user-adjusted pan/zoom state. The screenshot was inspected as render-only evidence; native input into the already-running window was not available in this review, so it does not establish an interaction result for this candidate.

Relationship labels and their attribute callouts now have session-scoped **Auto** and **All visible** modes. Auto is the default: at 36 or fewer viewport-visible routes it shows all labels; above that it shows labels for selected, path, source-focused, and community-focused records. **View… → Show all relationship labels** displays labels for every visible route. Label filtering leaves route rendering and route picking unchanged, fully offscreen routes remain culled, and every record remains available in Connections. The Auto label-geometry buffer is bounded to the threshold plus focused candidates. This change addresses label density. The card clipping remains a native review item; the old screenshot does not establish whether it persists at the candidate's fit view.

## Verification

| Check | Result |
| --- | --- |
| Headless `App::update` regression: dense Auto suppression, click selection of a record whose label is suppressed, actual View checkbox to All, all 40 visible labels present, unchanged route-segment count, and Auto labels returning after pan | **PASS** |
| Full offline suite: `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, 61 passed, 0 failed, 1 ignored; exit 0 |
| Offline check: `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | **PASS**, exit 0 |
| Offline debug build: `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | **PASS**, exit 0 |
| Offline optimized build: `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | **PASS**, exit 0; candidate binary hash recorded above |
| `cargo fmt --all -- --check`; `git diff --check` | **PASS**, exit 0 |
| Independent read-only code review | **PASS**; reviewer did not run tests or edit files |
| Current-user candidate install | **PASS**; `/home/sysadmin/.local/bin/arch-notebook` hash matches the release binary, `--version --json` and `desktop-file-validate` passed, and the case MIME default resolves to `arch-notebook.desktop` |
| Fictional sparse-30 review fixture | **PASS**; generator reported 30 entities and 90 records, SHA-256 `bf88b5df8c2c8e259e41a52486d8f09a12f6a6665d3a60892c802d8c06fa872d` |
| Updated candidate native launch | **BLOCKED**; `/home/sysadmin/.local/bin/arch-notebook /tmp/arch-notebook-review-sparse30.case.json` exited 1 with `WaylandError(Connection(NoCompositor))` |
| X11 fallback launch attempt | **BLOCKED**; `env -u WAYLAND_DISPLAY timeout 10s /home/sysadmin/.local/bin/arch-notebook /tmp/arch-notebook-review-sparse30.case.json` returned 124, and the subsequent full-desktop capture showed no Arch Notebook window |
| Updated candidate pointer/visual review | **NOT TESTED**; launch failed before a window appeared. The earlier desktop capture shows the old running app only |
| GPU, compositor, presentation, frame-rate, and input-to-display measurements | **NOT MEASURED** |

The original screen capture was stored temporarily at `/tmp/arch-notebook-review-screen.png`; it is not committed because it also contains unrelated desktop content. The installed candidate and sparse-30 fixture are ready for review when a Wayland compositor/input session is available. Use the [G3/G4 native acceptance runbook](G3-G4-native-acceptance-runbook.md) to record screenshots, binary hash, and actual pointer actions. Native G4 remains open until that review passes.

### Existing-window follow-up

On 2026-11-27 at 01:30 local time, a fresh full-desktop capture showed the same `VECTOR UI SPARSE 30` window (30 entities, 90 relationships) still open, with dense relationship labels and cards outside the visible viewport. The chart navigator shows that the current view is a subset of the whole graph, so this capture cannot distinguish intentional pan/zoom from a fit/layout defect. It also cannot identify which executable the existing process loaded; the newer release was installed after that process was already running. Capture: `/tmp/arch-notebook-fullscreen-current.png`, 2560×1440, SHA-256 `d70a718ed74e21a62752324a159504a93fbaef6c9754773fe9a230264f9a0f7e` (not committed because it contains unrelated desktop content). Input injection through this sandbox still fails (`xdotool`: `Failed creating new xdo instance`), so **Fit all**, Auto/All label mode, and G3 Arrange/Restore could not be exercised. This is a current-state observation only, not a regression result for the installed candidate.

## Reviewer handoff

- The reviewer checked source behavior after the candidate changes and returned PASS. In particular, Auto uses the post-culling visible-route count, source focus does not match two absent source values, and selection/picking remains independent of label presence.
- This report records headless, build, and per-user install evidence only for the updated candidate. The running-screen capture is evidence of the original display problem, not proof that the updated binary rendered or was accepted natively.
- **Gate status:** headless label behavior PASS; native G4 visual/input acceptance NOT TESTED; G3 Arrange/Restore native acceptance remains open; release/deployment readiness is not claimed.
