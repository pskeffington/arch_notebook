# G4 headless paint-path regression review

- **Task / reviewer:** Independent read-only review of the headless chart pan/culling regression; `/root/vector_patch_review`.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Reviewed baseline:** `7eced0c251e6b20f1305877ba12d51d94795f377` plus the uncommitted test-only change later committed unchanged as `c406d814ba146d26cb1419b4a6f13b524fccd820`.
- **Scope:** Whether `headless_chart_paint_culls_offscreen_relationship_labels` drives the production chart pan path and observes route culling in egui output. Not a review of native rendering, performance, or release readiness.

## Result

**PASS, with measurement limits.** The test obtains a pointer location from a rendered `CONTACTS` label, sends pointer press/move/release events, and verifies that `App::update` moves the chart pan by more than 1,900 points. This confirms the chart drag path ran. The label is between the two entity cards, and the large movement exercises dragging rather than selecting a relationship.

After release, the test counts only `CONTACTS` text shapes. The production culling branch executes before relationship route strokes and labels, so the lower label count after panning demonstrates that offscreen relationship drawing was skipped. The test uses a threshold rather than an exact route count and does not time CPU painting or the renderer.

The reviewer made no edits and ran no tests. The implementation owner separately ran the focused regression and full 57-test suite; see [G4 viewport-culling evidence](G4-VIEWPORT-CULLING-01-linux.md).
