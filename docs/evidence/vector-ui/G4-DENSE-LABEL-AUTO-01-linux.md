# G4-DENSE-LABEL-AUTO-01 · dense chart relationship labels

- **Task / owner:** SNG-DELIVERY-002 G4 readability follow-up; implementation session.
- **Baseline:** `6c283ba63df3abc9570854f440b3f5e6695c0517` on `codex/ui-00-chart-baseline`.
- **Candidate source:** `993dd560efca942fc4cc4c021d2fbefab1c96eef`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`; SHA-256 `d28f2420fa51680046764546af22be7a850827cc615873eb1633267aea1f3676`.
- **Review date:** 2026-11-27.
- **Scope:** Reduce relationship-label collisions in dense chart overviews while retaining route drawing, exact-record picking, and access to every record.

## Finding and change

The operator's already-running Arch Notebook review screen was open on fictional case `VECTOR UI SPARSE 30` (30 entities, 90 relationship records). The captured graph view showed many relationship labels stacked through the chart center and entity cards overlapping or clipped. The screenshot was inspected as render-only evidence; native input into the already-running window was not available in this review, so it does not establish an interaction result for this candidate.

Relationship labels and their attribute callouts now have session-scoped **Auto** and **All visible** modes. Auto is the default: at 36 or fewer viewport-visible routes it shows all labels; above that it shows labels for selected, path, source-focused, and community-focused records. **View… → Show all relationship labels** displays labels for every visible route. Label filtering leaves route rendering and route picking unchanged, fully offscreen routes remain culled, and every record remains available in Connections. The Auto label-geometry buffer is bounded to the threshold plus focused candidates. This change addresses label density; it does not resolve the entity-card overlap visible in the screen capture.

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
| Updated candidate native pointer/visual review | **NOT TESTED**; desktop capture showed the old running app, but native input could not be delivered to that window in this session |
| GPU, compositor, presentation, frame-rate, and input-to-display measurements | **NOT MEASURED** |

The existing screen capture was stored temporarily at `/tmp/arch-notebook-review-screen.png`; it is not committed because it also contains unrelated desktop content. Use the [G3/G4 native acceptance runbook](G3-G4-native-acceptance-runbook.md) to review the updated release binary with a fictional sparse-30 fixture and record screenshots, binary hash, and actual pointer actions. Native G4 remains open until that review passes.

## Reviewer handoff

- The reviewer checked source behavior after the candidate changes and returned PASS. In particular, Auto uses the post-culling visible-route count, source focus does not match two absent source values, and selection/picking remains independent of label presence.
- This report records headless and build evidence only for the updated candidate. The running-screen capture is evidence of the original display problem, not proof that the updated binary is running or accepted natively.
- **Gate status:** headless label behavior PASS; native G4 visual/input acceptance NOT TESTED; G3 Arrange/Restore native acceptance remains open; release/deployment readiness is not claimed.
