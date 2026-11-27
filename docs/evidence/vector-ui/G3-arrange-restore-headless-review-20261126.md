# G3 headless Arrange/Restore regression review

- **Review role:** independent code reviewer (`vector_patch_review` session agent)
- **Implementation baseline:** `f7bb4918c85ac296f02d07f579e231a29f04b5c2` on `codex/ui-00-chart-baseline`; review covered the new headless Arrange/Restore test in `src/main.rs` and its 1 ms wait refinement.
- **Scope:** test validity and flakiness only; not native UI, force-layout algorithm quality, or release approval.
- **Outcome:** **PASS**.

The test drives the actual `View…` menu, clicks `Arrange chart`, advances UI frames until the asynchronous result is applied, checks that a pinned entity stays fixed and a movable entity moves, then opens the menu again and clicks `Restore positions from before force layout`. It verifies every entity's coordinates and pin flag match the initial snapshot and that the restore snapshot is cleared. It exercises the production UI event path without calling `auto_layout` directly.

The reviewer ran the focused test 10 times before the final wait refinement; all 10 runs passed (0.06–0.21 seconds each). The final refinement replaced tight `yield_now()` polling with 1 ms sleeps, bounded to 100 frames. The focused test then passed 1/1 in 0.09 seconds, and `git diff --check` passed. The test remains headless and does not establish native visual/input acceptance.
