# Vector UI optimization — literature matrix

Pass UI-VEC-001 | 2026-11-25 | Initial source-linked research, not implementation acceptance.

Initial-pass baseline: main README and Cargo.toml inspected; that pass did not independently pin a baseline commit. The subsequent roadmap source audit inspected commit `a1cca830de5324080e15aa06ec8f2dc710eba3c0` and confirmed egui/eframe 0.31.1 in Cargo.lock, with the Glow backend enabled in Cargo.toml. Retain native rendering pending profiling. Reported TRANS PDF/email ingestion success is a separate user observation, not a rendering benchmark. No case records included.

Implementation queue: [ROADMAP.md](../../ROADMAP.md). Codex start instructions: [CODEX_NEXT.md](../../CODEX_NEXT.md). Measurement and native acceptance: [vector UI protocol](../vector-ui-acceptance.md). The roadmap refines the initial gates below against actual source; it does not mark the pending scholarly checks complete.

## Evidence matrix

| ID | Publication / type | Stable source | Inspected locator / evidence | Engineering application | Validation boundary |
|---|---|---|---|---|---|
| V01 | Barnes & Hut (1986), A hierarchical O(N log N) force-calculation algorithm; primary research letter | https://doi.org/10.1038/324446a0 | Publisher abstract: hierarchical spatial subdivision for approximate N-body forces | Compare exact repulsion against quadtree approximation for 2D layout | Original domain is gravity; no Arch benchmark; full page/equation pending |
| V02 | Jacomy, Venturini, Heymann & Bastian (2014), ForceAtlas2; primary research | https://doi.org/10.1371/journal.pone.0098679 | Abstract; Approximate repulsion; Preventing overlapping discussion; Performance Optimization. Explicit adoption of Barnes-Hut; approximation may be slower for small networks | Optional force layout, node-size collision handling, adaptive iteration, explicit stop and pinned positions | Publisher + PubMed identity checked; equations/page verification pending |
| V03 | Ghoniem, Fekete & Castagliola (2005), On the Readability of Graphs Using Node-Link and Matrix-Based Representations; controlled experiment | https://doi.org/10.1057/palgrave.ivs.9500092 | Publisher abstract: seven tasks; representation effectiveness varies with size/density | Add synchronized adjacency matrix as optional dense-network view | Abstract only; study threshold not a universal cutoff |
| V04 | Nachmanson, Prutkin, Lee, Riche, Holroyd & Chen (2015), GraphMaps: Browsing Large Graphs as Interactive Maps; conference research | https://www.microsoft.com/en-us/research/publication/graphmaps-browsing-large-graphs-interactive-maps/ | Institutional publication abstract: layered semantic zoom, viewport intersection and stable geometry | Preserve coordinates while zooming; limit visible labels and primitives | Proceedings DOI, full text and page pending |
| V05 | Di Giacomo, Didimo, Liotta, Montecchiani & Tappini (2024), Comparative Study and Evaluation of Hybrid Visualizations of Graphs; primary user study | https://doi.org/10.1109/TVCG.2022.3233389 | IEEE abstract: hybrid views evaluated for globally sparse, locally dense networks | Optional cluster-local matrix with node-link overview | Abstract only; no universal winning view inferred |
| V06 | Di Bartolomeo, Crnovrsanin, Saffo, Puerta, Wilson & Dunne (2024), Evaluating Graph Layout Algorithms: A Systematic Review of Methods and Best Practices; secondary systematic review | https://doi.org/10.1111/cgf.15073 | Publisher abstract and section 15: heterogeneous datasets and evaluation metrics; 206 reviewed papers | Use computational and task-based evaluation with graph size, density and reproducible fixtures | No local reproduction; independent registry and page pending |
| V07 | Nachmanson & Chen (2026), Browsing Large Graphs with Tile Pyramids and Sleeve Routing in the Browser; institutional research record | https://www.microsoft.com/en-us/research/publication/browsing-large-graphs-with-tile-pyramids-and-sleeve-routing-in-the-browser/ | Abstract: ranked labels, semantic zoom tiles, constrained Delaunay and funnel routing; nine graphs up to 32,768 nodes and 236,978 edges | Investigate tile hierarchy only after simple culling; consider obstacle-aware routes | Publication venue/DOI and review status pending; browser results not transferable to native Rust without testing |
| V08 | egui Painter API; technical documentation, not scholarly study | https://docs.rs/egui/latest/egui/struct.Painter.html | with_clip_rect; extend; screen coordinates; pixels_per_point | Clip and batch vector shapes; check DPI; avoid unnecessary tessellation | Retrieved latest 0.37 docs, not installed 0.31; confirm API in locked source before implementation |

Exact PDF/printed pages, correction/retraction checks and independent registry verification remain pending except V02 identity cross-check. No source equations transcribed. Historical sources retained for algorithm provenance; contemporary sources for evaluation. Only explicit lineage currently established: V02 adopts V01; other links are proposed applications.

## Engineering model (proposed, not source equations)

World-to-screen transform s = o + z(p - c), z > 0, where o is viewport origin, c camera origin and p world position. Inverse p = c + (s - o)/z. Store geometry in world coordinates and use consistent inverse transform for hit testing. Render edge paths, arrowheads, icons and labels separately. Cull by intersecting conservative bounds (including edges crossing the viewport with both endpoints offscreen). Separate model identity from visual aggregation; selection and evidence must resolve stable IDs.

Implementation qualification from the pinned source: existing `Entity.x/y` values are normalized chart coordinates. `App::update` first maps those values into the viewport, then applies zoom around the viewport center and screen-space pan. The abstract model above is not authority to change persisted coordinate semantics. UI-01 must preserve that storage convention and test the matching forward/inverse mapping. Entities have UUIDs, but links can have empty or source-scoped connection IDs; derived render handles require a case/topology revision and cannot assume a global link identifier.

Measure node overlaps, label overlaps, edge crossings excluding shared endpoints, layout displacement of persistent nodes, p50/p95/p99 frame time, input-to-visible response, memory and task accuracy/time. Compare baseline and alternatives on identical seeded fixtures. Report hardware, build SHA, dependency lock, DPI, graph sizes and distributions. No universal graph size threshold.

## Implementation gates

| Gate | Work | Acceptance |
|---|---|---|
| G0 | Locate chart paint, layout and hit-test paths; profile release build | Capture baseline on Linux and macOS; no data changes |
| G1 | Camera transform, clip, conservative culling and shape batching | Round-trip transform, crossing-edge visibility, pointer picking, DPI tests; compare frame times |
| G2 | Cached geometry and text; spatial index; dirty flags | Pan/zoom does not trigger full layout; cache invalidation tests on edits and style changes |
| G3 | Semantic zoom, stable labels, selected-neighbor focus | Selected items and source links remain accessible; visible/total counts and explicit aggregation |
| G4 | Optional layout modes: existing, force-directed, hierarchical/radial as appropriate | Seeded repeatability, pinned nodes, no silent changes to graph topology; compare overlaps and task accuracy |
| G5 | Parallel-edge routing, direction arrows, dense-view matrix | Preserve multiplicity and provenance; test loops, reversed edges and overlapping paths |
| G6 | Comparative usability and performance acceptance | Synthetic graphs of 100/1k/10k nodes, sparse and dense variants, hubs, disconnected groups and multiedges; report actual counts and frame distributions; use fictional email fixtures |
 
Proposed interaction budget: p95 frame time <=16.7 ms for declared interactive workload on documented reference hardware; if unmet report measured limits rather than claim success. Proposed task tests: locate actor, follow directed path, distinguish parallel relationships, open source evidence, return to prior view. Record task accuracy and completion time; no fabricated participants.

Priority: G0–G2 before adding a new GPU backend or SVG library. G3 next for readability. Retain existing identity/snapshot tests and add fixture-driven ingestion and chart regressions in UI-00. No raw operational email documents, private identities or source content in public benchmark fixtures.

## Source reconciliation and executable queue — 2026-11-25

The source audit found four existing unit tests for connection identity and legacy snapshot reconciliation; an extensive chart/ingestion regression suite is not established. UI-00 retains those tests and adds the fixture-driven checks. The current CI workflow builds/packages but does not run `cargo test`.

| Initial gate | Source-grounded follow-through |
| --- | --- |
| G0 | UI-00 establishes native measurements, fictional fixtures and an executed test gate. |
| G1 | Camera controls and `painter_at(rect)` clipping already exist. UI-01 consolidates transforms and paint/pick geometry; UI-02 adds conservative CPU culling. |
| G2 | UI-02 adds measured revision-aware caching and candidate indexing with mutation tests. Existing network-group caches are retained. |
| G3 | UI-03 controls label detail, focus and keyboard access while keeping records/evidence reachable. |
| G4 | Connectivity, hierarchy and grid layouts already exist. UI-04 evaluates an optional new layout after method verification. |
| G5 | Parallel lanes and direction arrows already exist. UI-01 fixes loop/picking correctness; advanced routing and dense matrix views remain conditional. |
| G6 | UI-05 records Linux and Foundation/Mac evidence under a defined measurement and task protocol. |

V08 originally used newer documentation. Cargo.lock/source inspection closes the installed-version question, not API compatibility with the cited latest page. R1 must inspect matching 0.31.1 API documentation or crate source before adopting an API. Remaining publication, page, equation and external registry checks retain their existing pending status. No native timings or user-study results were added by this roadmap pass.
