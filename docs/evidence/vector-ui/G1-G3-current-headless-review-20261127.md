# G1–G3 current headless UI review

- **Review role:** Independent code reviewer (`vector_patch_review` session agent).
- **Reviewed source:** `487473d0ff655e0b9fc0584fcd8dd59a7e320dbb` on `codex/ui-00-chart-baseline`.
- **Scope:** Current community-exploration fixture and Arrange/Restore UI event-path assertions. Read-only review; reviewer did not run tests or edit files.
- **Result:** **PASS for headless UI coverage. Native UI acceptance is not established.**

## Findings

- The community fixture generates more than 100 detected communities. It verifies the last community is absent from initial rendered shapes, scrolls until its virtualized row appears, activates the row's show control, and checks entity selection and chart-fit target. This exercises actual headless UI scroll and click events for a distant row.
- Before inspecting a relationship, the test seeds an invalid chart-fit target. It invokes the rendered Inspect control and checks the stale target is replaced by exactly the relationship's two endpoints, while selection points to the source entity and inspected relationship.
- The Arrange/Restore fixture invokes the rendered View menu actions. It waits for the asynchronous layout worker, checks a pinned entity remains fixed and a movable entity changes, then checks Restore returns all positions and pin flags and clears the snapshot.

## Limits

These checks establish headless UI event paths, not native display behavior, visual layout quality, or native input acceptance. The Arrange/Restore fixture polls for at most 100 one-millisecond waits; this is a possible load-related flakiness limit and should be revisited if CI reveals failures.

Current-candidate native launch/render evidence is separate: [LOCAL-INSTALL-05](../deployment/LOCAL-INSTALL-05-linux.md). The candidate window rendered the fictional correctness fixture at 1440×900. Its native pointer and keyboard actions remain untested.
