# SNG-DELIVERY-002: Detected communities and vector graph optimization

- **Owner:** orchestrator; implementation proceeds in bounded stages.
- **Baseline:** `91ebd699b49b8438e168a126e47004db87b908c5` on `codex/ui-00-chart-baseline`.
- **User decisions:** detected communities; optimize both graph rendering and layout.
- **Status:** G0 definition and implementation plan recorded; implementation gates not yet started. Native visual and interaction measurements are blocked until an input-capable desktop session is available.
- **Scope:** local Arch Notebook chart exploration. No inference of real-world group identity, influence, intent, or causation; no changes to source records or SQL data.

## Product definition

The application will offer **detected structural communities** as an exploratory partition of the currently analyzed chart. Membership is computed from the chart's relationship records and is not an analyst assertion or a validated social group. Community labels are generated locally (for example, `Community 1`) and must not be confused with entity type, analyst labels, or known organizations.

For the first version, detection uses the active chart's **undirected, unweighted topology**: direction is ignored, each distinct unordered pair of present entities contributes one edge, parallel records do not inflate edge weight, self-links and records with missing endpoints are excluded, and isolates remain singleton communities. This prevents repeated capture records from silently becoming a claim of stronger social ties. The UI and method card must state these rules and the algorithm, resolution/seed settings, and analyzed chart scope. Filtered views must either compute on exactly the visible record set and disclose that scope or be explicitly unavailable; they must never imply otherwise.

Community interaction summaries must distinguish (a) relationship record count from (b) distinct endpoint-pair count. Each row/cell must drill back to the contributing chart records. For directed records, report records in each direction separately and show undirected records separately. A community-to-community table/matrix is a descriptive summary of recorded ties, not a measure of social cohesion or importance.

## Phased gates

| Gate | Work | Acceptance criteria | Current state |
| --- | --- | --- | --- |
| G0 — semantics and baseline | Confirm input graph, edge denominator, direction, filters, missing endpoints, isolates, reproducibility, and baseline timing methodology. Capture sparse, hub-heavy, disconnected, and dense synthetic graph baselines. | Definitions above are reflected in a method card. Baseline includes node/link counts, render/layout timings, and memory or geometry counts where measurable. No machine-specific target is claimed as universal. | **PASS for definitions; BLOCKED for native baseline measurements.** |
| G1 — community partition | Add a deterministic, documented community detection implementation and chart-level view. Evaluate Leiden as the preferred family because it targets connected communities; first establish an acceptable dependency-free implementation or an approved, portable dependency. | Fixtures cover cliques joined by bridges, disconnected components, isolates, cycles, ties between communities, repeated/reversed records, self-links, and missing endpoints. Membership is total, deterministic for fixed inputs/settings, and each non-singleton detected community is connected under the declared graph. Report algorithm and resolution/settings; no truth claims. | **NOT STARTED.** |
| G2 — group interaction exploration | Add a community interaction table or matrix, selection/highlight on the chart, and record-level drilldown. | Counts reconcile with fixtures for unique pairs and relationship records; direction is preserved in displayed record counts; filtering and scope are explicit; selecting a cell identifies exactly its contributing records. | **NOT STARTED; depends on G1.** |
| G3 — layout optimization | Prototype deterministic, community-aware force layout with pinned-node preservation and configurable iteration/quality controls. Keep layout separate from analysis and preserve exact node/link topology. | Compare against G0 on sparse, hub, disconnected, and dense fixtures; coordinates finite and bounded; stable for fixed seed/settings; pins unchanged; no nodes/links created or removed; layout can be reset; no claim that spatial proximity encodes social distance. | **NOT STARTED; depends on G0 and G1.** |
| G4 — vector rendering optimization | Measure before optimizing. Consider viewport culling, cached/prepared vector geometry, and batched draw work while preserving edge routing and picking. | Benchmarks show improvement on named fixtures; off-screen edges crossing viewport remain visible; multiedges remain individually selectable; labels, direction markers, self-loops, zoom/pan, and keyboard/accessibility behavior remain correct. | **NOT STARTED; depends on G0.** |
| G5 — integration and independent review | Integrate docs, run prescribed offline gates, obtain independent code/security review and runtime visual/input QA. | All automation and review findings recorded with source commit and artifacts; native UI is PASS only after actual input-capable review. No release/deployment claim. | **NOT STARTED.** |

## Algorithm and performance decision record

- **Community detection:** investigate Leiden first. Its published method addresses disconnected or poorly connected communities that can arise with Louvain; implementation feasibility, license, dependency policy, and graph-size performance still need repository-specific review. A fallback implementation must be named accurately and pass connectivity invariants; do not call a different algorithm “Leiden.” Modularity methods have a resolution limit, so the UI must expose limitations and avoid presenting one partition as uniquely correct.
- **Tie semantics:** v1 collapses each unordered entity pair to a single unweighted edge for membership. Relationship-record counts remain available separately in G2. Do not infer frequency/strength by counting duplicate records unless a later explicit analysis mode defines that denominator.
- **Layout:** a force-directed layout is only a display arrangement. A seeded layout can be evaluated for stable exploration, but position, cluster compactness, and visual distance are not measured social facts. Preserve current manual positions and pins through explicit user control.
- **Rendering:** no optimization is accepted based on visual intuition alone. Record baseline and candidate measurements with fixture characteristics and the same machine/build settings. Preserve semantic distinctions between records, pairs, and rendered strokes.

## Work sequence and rollback

1. Complete current SNG-DELIVERY-001 shortest-path and packet-review repairs, preserve their independently reviewed evidence, and keep native UI status blocked until desktop interaction is possible.
2. Build G0 benchmark fixtures and a repeatable local measurement harness; store generated measurements under `/tmp` until reviewed.
3. Implement G1 behind a chart-level opt-in view; if algorithm licensing or complexity is unresolved, deliver the researched decision and defer code rather than shipping an inaccurately named substitute.
4. Implement G2 against the declared community membership output.
5. Prototype G3 and G4 independently; retain existing layouts and renderer as rollback paths until acceptance passes.
6. Run G5 review and update this roadmap with exact baseline, changed files, commands, exit codes, artifact digests, and unresolved risks.

Rollback is to disable/remove the new community and optimized layout/render paths while retaining the existing chart, analysis, and layout behavior. Community assignments and optimized coordinates must not rewrite source entities, relationships, chart records, or saved evidence.

## Research basis

- Traag, Waltman, and van Eck, “From Louvain to Leiden: guaranteeing well-connected communities,” *Scientific Reports* 9, 5233 (2019), [DOI](https://doi.org/10.1038/s41598-019-41695-z). Supports evaluating Leiden when connectedness of communities matters; it does not establish that computed groups are true social groups.
- Fortunato and Barthélemy, “Resolution limit in community detection,” *PNAS* 104(1), 36–41 (2007), [DOI](https://doi.org/10.1073/pnas.0605965104). Documents scale-related limitations of modularity optimization.
- Jacomy et al., “ForceAtlas2, a continuous graph layout algorithm for handy network visualization designed for the Gephi software,” *PLOS ONE* 9(6), e98679 (2014), [DOI](https://doi.org/10.1371/journal.pone.0098679). Describes a visualization layout algorithm; layout geometry is not evidence about social distance or causality.
