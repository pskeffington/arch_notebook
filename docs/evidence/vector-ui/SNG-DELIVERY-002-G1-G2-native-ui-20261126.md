# SNG-DELIVERY-002 G1/G2 native UI acceptance

- **Task/owner:** Native review of detected communities, community interaction drilldown, and hop-layer contact map; owner `/root`.
- **Source baseline:** `37c17ba72880744dfaf79616644d023c7c17d71e` on `codex/ui-00-chart-baseline`, plus the currently uncommitted compact-label UI patch in `src/main.rs`.
- **Release binary SHA-256:** `b02662cde4d789893b7607ac8a78d574fac27a57d4251f9e8780a750b074d929`.
- **Fixture:** disposable copy of `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`; the copy was kept in `/tmp` and not saved from the application.
- **Display:** KDE X11/XWayland, 2560×1440 desktop; application window 1440×900. Winit's Wayland launch returned `NoCompositor`; the same built binary launched through X11 successfully.

## Native interaction

The test used actual pointer and wheel input against the release application window:

1. Clicked **Detect communities**. The pane displayed six detected communities and four isolates.
2. Scrolled the Analysis pane, clicked **Show** for Community 6, and confirmed the chart selected and fit the singleton entity.
3. Selected the **Community 1 ↔ 2** interaction row, scrolled its record list, and clicked **Inspect #3 · REVERSED TEST RECORD**. The chart focused the two endpoints, **Avery North Coincident** and **Cedarline Fictional Cooperative**, and selected the specific relationship record.
4. Clicked **Measure hop layers** for a selected entity. The pane displayed hop layers, the bounded contact-group map, group-to-group relationship totals, and proxy summaries.

The narrow-sidebar review exposed clipped long action and record labels. The current candidate shortens the detection and inspect actions, places directional counts on a separate line, and displays endpoint details beneath each inspect action. In the captured 1440×900 window, the community counts and record details are readable without widening the pane.

## Evidence and result

- [Community interaction and exact-record focus screenshot](artifacts/SNG-DELIVERY-002-G1-G2-native-inspect-20261126.png), SHA-256 `2a83d25c4a008f3f16f2155c0fdaeab3b1b417e80c14ad660df8292dcf5e119c`.
- [Hop-layer contact-map screenshot](artifacts/SNG-DELIVERY-002-G1-G2-native-contact-map-20261126.png), SHA-256 `32a7cb89c5f652def8c009bee0ddb9e9bf2fd47fc5ad81dd0fe569ee5a9e5c4c`.
- **G1/G2 native action gate: PASS for this fictional chart and tested path.** The detected partition, late-community selection, interaction selection, exact record drilldown, chart focus, and contact-map rendering worked with real pointer/wheel input.
- The desktop command was started as `env -u WAYLAND_DISPLAY DISPLAY=:0 /tmp/arch-notebook-target/release/arch-notebook /tmp/arch-notebook-native-g5.case.json`; it remained live during review and was stopped with Ctrl+C afterward (exit 130, deliberate termination).

## Limits and remaining gates

This is one host and one small fictional chart. It does not establish keyboard behavior, accessibility, arbitrary window resizing, GPU/render timing, FPS, or broad performance. A native click on a contact-map node was not confirmed. G3 force-layout arrange/restore input review and G4 paint/presentation measurements remain open; the CPU geometry benchmark is not a rendering benchmark. No production data was used, no case was saved, and no release approval is implied.
