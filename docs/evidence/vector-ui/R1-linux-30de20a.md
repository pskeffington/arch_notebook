# R1 Linux source verification follow-up

- **Task / owner:** Verify renderer API and source claims that inform the vector UI queue; Codex, Linux Operations. The PR #3 plan assigns Mac validation and the full parallel R1 review to Foundation; that work is not claimed here.
- **Repository / branch:** `pskeffington/arch_notebook`, `codex/ui-00-chart-baseline`.
- **Source commit:** `30de20a13a7f9f274002d0aabb9591abfb3d8459`.
- **Planning source:** PR #3 roadmap head `1c42009c69da3bee6caed0b2f25d6e81a470e69f`.
- **Lockfile:** `Cargo.lock` SHA-256 `4e14e9b55f812bfb1af0d7687039d4bc266a76f4a6f584dee8d7caa1349c01a3`.
- **Changed paths:** `docs/research/vector-ui-literature-matrix.md`, this evidence file, and `README.md` link update.
- **Status:** PARTIAL source verification. No renderer implementation, performance claim, or independent literature review is included.

## Findings and engineering decisions

1. **Pinned egui behavior is now checked against the local 0.31.1 source.** `Painter` uses logical screen points; `with_clip_rect` intersects its parent clip; `pixels_per_point` reports scale; and `extend` accepts shape iterators. The app chart creates `ui.painter_at(rect)` but still iterates links/entities and prepares routes/shapes in the drawing path. A clip rectangle alone therefore does not establish that caller-side route or text work was skipped. Keep the existing backend; measure conservative CPU culling before introducing caches or another renderer. Treat batching as a testable hypothesis.
2. **Force-layout papers do not justify adding a force layout now.** Barnes–Hut is an N-body approximation with a different source problem. ForceAtlas2 reports that approximate repulsion can be counterproductive on small graphs and that its overlap-prevention mode slows convergence. UI-04 remains optional and requires local seeded comparisons, bounded/cancellable behavior, pin preservation, and user-task improvement.
3. **Dense matrix views should remain optional and task-specific.** The seven-task study reports matrix advantages on most tasks beyond 20 vertices but a persistent node-link advantage for path-finding. Do not turn that study-specific result into a global node-count cutoff; keep node-link navigation and evidence access available.
4. **Semantic zoom is a later view-policy candidate.** GraphMaps uses precomputed layers, stable geometry, and viewport intersection. This supports investigation of overview/detail navigation but is broader than the UI-02 culling task and does not directly prescribe Arch Notebook’s implementation.
5. **Use hybrid-view and layout-review papers as evaluation guidance, not product guarantees.** The IEEE author version studies several hybrid metaphors on globally sparse/locally dense networks. The systematic review documents diverse datasets, metrics, and evaluation styles; our own reports must declare the actual workload and analyst task.
6. **The 2026 tile-pyramid paper is a future browser-specific candidate.** The institutional record describes MSAGLJS/WebGL results. No native Rust transfer or peer-review status is established, so UI-02 should first measure simpler culling.

The per-source bibliographic locators, exact qualifications, and links are in the [literature matrix](../../research/vector-ui-literature-matrix.md). Official sources inspected include the [Nature letter](https://doi.org/10.1038/324446a0), [PLOS ONE ForceAtlas2 article](https://doi.org/10.1371/journal.pone.0098679), [SAGE controlled study](https://doi.org/10.1057/palgrave.ivs.9500092), [Springer GraphMaps chapter](https://doi.org/10.1007/978-3-319-27261-0_1), [IEEE hybrid-visualization author version](https://ieeexplore.ieee.org/ielx7/2945/4359476/10004748.pdf), [Wiley review](https://doi.org/10.1111/cgf.15073), and the [Microsoft Research tile-pyramid record](https://www.microsoft.com/en-us/research/publication/browsing-large-graphs-with-tile-pyramids-and-sleeve-routing-in-the-browser/).

## Reproducible repository checks

```text
rg -n 'version = "0\.31\.1"' Cargo.lock
exit 0 (egui, eframe, epaint, egui_glow and related pinned packages found)

rg -n 'with_clip_rect|pub fn extend|coordinates are screen coordinates|pub fn pixels_per_point|set_clip_rect' /home/sysadmin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/egui-0.31.1/src/painter.rs
exit 0 (methods and coordinate/clip contract found in pinned local crate source)

rg -n 'ui\.painter_at\(rect\)|for .*self\.case\.links|for .*self\.case\.entities|connection_curve_points\(' src/main.rs | tail -30
exit 0 (chart painter and collection traversal confirmed)
```

No source equations were transcribed. Full-text review, exact equation/page crosswalk, corrections/retractions, independent registry verification, and macOS-specific source/runtime review remain pending. The crosswalk skill named in PR #3 was not present in the available local or repository skill catalogs; no separate reviewer invocation was used.

## Handoff

This partial source pass supports the queue order already set in PR #3; it does not satisfy the complete R1 gate. Keep UI-01 behind the unfinished UI-00 native interaction/visual baseline. Foundation can complete the independent source crosswalk and Mac-specific checks when that owner is available; Linux Operations should next complete native UI-00 input/capture evidence before changing chart geometry.
