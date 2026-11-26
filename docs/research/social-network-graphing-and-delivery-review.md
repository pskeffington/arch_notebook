# Social network graphing and delivery — focused literature review

**Reviewed:** 2026-11-26  
**Repository baseline:** `dff7acfb5a5ffa9c79b692e0e32841821c4f64c8` (`codex/ui-00-chart-baseline`)  
**Status:** Evidence-informed product guidance; not an implementation or external usability acceptance.

## Scope and approach

This is a focused narrative review of primary research, a published social-network reporting guideline, and a recent research-ethics recommendation paper. Sources were checked at publisher, PubMed Central, university, or author-hosted pages on the review date. This is not a systematic review: searches were bounded, no protocol or exhaustive database search was used, and no user study was run for Arch Notebook. The 2023 SoNHR guideline is written for health research, so its network-definition and reporting items transfer well as questions to ask, but it is not a universal compliance standard. The 2026 ethics article is a multidisciplinary commentary and recommendations paper, not an empirical visualization experiment.

Arch Notebook represents entities and relationship records. A chart should be called a *social network* only when its entities represent social actors and its ties operationalize a social relation. A graph of organizations, incidents, accounts, or communications is still a network, but each relationship needs its own substantive interpretation.

## Evidence matrix

| ID | Source and evidence type | Findings relevant to graphing or delivery | Transfer limits |
| --- | --- | --- | --- |
| S01 | Luke et al. (2023), [SoNHR reporting guidelines](https://doi.org/10.1371/journal.pone.0285236), PLOS ONE 18(12), e0285236; peer-reviewed reporting guideline | Define why a network is needed; identify node and tie meanings, direction/weight, network boundary/type, data collection, missingness, and transformations. State the unit of analysis and interpret metrics in the context of the real-world relation. Recommend encodings fit to the network and task, including categorical node color/shape, quantitative node size, restrained labels for large graphs, and link variation for small graphs. | Developed for health-science social-network reporting. The authors state it is guidance, not a formal checklist for every study. It does not prescribe one layout or a universal graph-size threshold. |
| S02 | Ghoniem, Fekete & Castagliola (2005), [node-link vs. matrix readability study](https://doi.org/10.1057/palgrave.ivs.9500092), *Information Visualization* 4(2), 114–135; controlled task study | Compared seven graph tasks across different sizes and densities. The paper reports complementary strengths: node-link diagrams support path tasks, while matrix views perform better for several lookup/count tasks as graphs become larger or denser. | Results depend on the tested tasks, graph samples, layout, interaction, and participant familiarity. The paper explicitly cautions against treating its size bands as universal. |
| S03 | Yoghourdjian et al. (2020), [“Scalability of Network Visualisation from a Cognitive Load Perspective”](https://doi.org/10.1109/TVCG.2020.3030459), IEEE TVCG / InfoVis; controlled shortest-path study | Tested shortest-path finding on graphs from 25 to 175 nodes with varying density. Participants had significant difficulty in the tested dense graphs above 50 nodes and sparse graphs above 100; global visual complexity had more impact than path straightness or length in their study. The authors motivate filtering or aggregation as complexity grows. | These are results for that experiment and task, not a safe cutoff for all users or networks. Do not hard-code the 50/100 values as a product threshold. |
| S04 | Nobre, Wootton, Harrison & Lex (2020), [“Evaluating Multivariate Network Visualization Techniques…”](https://web.cs.wpi.edu/~ltharrison/docs/nobre2020evaluating.pdf), CHI; validated user-study designs | Compared node-link diagrams with on-node encodings to adjacency matrices paired with tables. Node-link was better suited to tasks integrating topology with a few attributes; matrix-plus-table performed well for cluster tasks and when many attributes mattered. | Evidence covers selected tasks and two designs. It supports task-based alternatives, not a claim that one representation is always superior. The inspected copy is an author manuscript. |
| S05 | Ortiz Ruiz et al. and collaborators (2026), [“Recommendations for conducting ethical network research”](https://doi.org/10.1017/nws.2026.10034), *Network Science* 14, e17; multidisciplinary commentary | Calls for documenting node/tie construction, temporal/spatial boundaries, preprocessing and provenance; cautions against deterministic interpretations and reducing people to network position. Notes that small or familiar networks can be re-identified from structure and attributes, even when direct identifiers are removed. Recommends transparent, accessible presentation and contextualized stakeholder engagement where appropriate. | Ethical and governance decisions depend on data, jurisdiction, consent, purpose, and audience. This paper is a recommendation source, not legal advice or a blanket authorization to publish identifiable graphs. |

## Practices for producing an analysis graph

### Define the graph before drawing it

- State the question the graph should answer and why a relational view helps answer it.
- Define what each node represents and how records become nodes. Explain identity reconciliation, aliases, merges, and any aggregation across people, devices, organizations, events, or locations.
- Define each tie by its observed meaning and source. Distinguish contact from co-occurrence, report from confirmation, and communication from affiliation. Specify whether a tie is directed, binary or weighted, repeated, time-bounded, or typed.
- State the network boundary: included population/entities, source coverage, case/chart, time window, and relevant exclusions. Identify whether isolates are retained and whether the network is a complete bounded network, an ego network, or a sample.
- Describe missing, conflicting, duplicated, and uncertain records. State which transformations or filters changed the analyzed graph. Do not let a missing tie imply that no relationship existed.

### Match representation and layout to the task

- Use a node-link chart for following paths, inspecting local neighborhoods, and combining topology with a small number of attributes. Treat it as an overview and navigation device, not a substitute for record-level evidence.
- As density or attribute count makes edges and labels overlap, narrow the view with a stated time, tie-type, source, or neighborhood filter. For dense pair lookup, cluster comparison, or many attributes, provide a table or adjacency-matrix view as a complementary representation.
- Keep semantic meaning in the legend, not in unexplained position. A force-directed cluster, short visual distance, or central-looking position is a layout result; by itself it does not establish a social group, relationship strength, influence, or causation.
- Encode only a few important variables at once. Use color or shape for a defined category and size for a defined quantity with a visible scale. Make direction, parallel records, and weighted ties inspectable. Do not imply weights where the source only supports binary ties.
- Keep entity identity and evidence reachable when filtering, focusing, aggregating, or changing layouts. If a display summarizes or omits records, show the filter and counts and provide a path back to the represented records.
- Prefer stable positions when comparing snapshots so movement on the page is not mistaken for network change. When a layout is recomputed, distinguish layout motion from changed nodes or ties.
- Choose color palettes that remain distinguishable for common color-vision deficiencies; include labels, shape or line-style redundancy where color carries a critical distinction. Provide a concise legend and a text summary of the main pattern.

### Interpret measures as properties of the constructed graph

- Explain a measure's unit and denominator. Degree is the number of included ties under a stated counting rule; it is not automatically popularity, authority, or importance.
- Betweenness, articulation points, components, and hop distance describe graph structure under a particular node/tie definition. They do not prove real-world control, intent, trust, or causal influence.
- State whether direction, weight, date, relationship type, parallel records, and self-links were honored or collapsed for each result. Re-run important conclusions under plausible alternative boundaries or tie rules when the input is incomplete or ambiguous.
- Separate observed records, analyst interpretation, and hypotheses in text and visual captions. Include counterexamples and uncertainty, not only the most visually prominent node or cluster.
- Avoid generalizing a bounded or convenience-sampled network to a population without a sampling argument. A complete graph for one source and time window can still omit people, channels, or relations outside that collection process.

## Delivering results to another analyst or audience

A result should travel with enough context for another person to reproduce its meaning and challenge it. For each delivered figure or finding, include:

1. **Question and scope:** what decision or question this view addresses; included entities, sources, chart/case, and time window.
2. **Graph definition:** what nodes and ties mean; direction and weight; duplicate/parallel-record policy; inclusion, exclusion, and isolate policy.
3. **Method:** filters, transformations, identity reconciliation, metric definition, unit, software/version, layout, and any layout or analysis parameters that affect the claim.
4. **Evidence:** visible counts, a readable legend/caption, source references or stable record IDs, and a route to inspect supporting records. Never replace underlying records with an unsupported visual summary.
5. **Interpretation and limits:** state what the result supports, what it does not establish, material missingness, alternative explanations, and sensitivity to graph choices.
6. **Audience and privacy review:** check permission, confidentiality, small-network re-identification, non-participant exposure, and whether labels or attributes can be minimized or pseudonymized without breaking necessary record linkage. Do not assume removing names anonymizes a graph.
7. **Accessible alternative:** provide a text summary or table for people who cannot interpret the image or color encoding. If the graph is interactive, retain a static summary and explain filters used to produce the exported view.

**Suggested caption pattern:** “For [defined entities] in [source/boundary] during [dates], [tie definition and direction/weight rule] produced [N nodes, M relationship records, X components] after [transformations]. The highlighted [metric/subgraph] means [graph-level definition]. The view omits or cannot establish [limits]. Evidence: [record/source references].” Fill only values actually calculated and verified.

## Application to Arch Notebook

The current application is an entity/link analysis notebook, not a social-network-specific statistical package. It supports source references, typed entities and links, repeated relationship records, connectivity rankings, weak components, articulation entities, shortest paths, and geodesic hop layers. Their assumptions differ by view: for example, the component, bridge, and geodesic views treat links as undirected; geodesic layers also treat every link as one unweighted step. A chart layout does not change the source relationship facts.

Recommended delivery-focused product sequence:

1. Add a compact **analysis method card** for chart exports: active chart/case, source and time scope, filters, directed/undirected and weight treatment, entity/link counts, transformations, and metric definitions.
2. Make per-view assumptions visible beside results and include them in saved/exported summaries. Do not describe degree as popularity or a bridge entity as a real-world controller.
3. Provide a dense-network workflow that combines a filtered node-link view with sortable records and, after task validation, an optional adjacency matrix for pair/cluster comparison.
4. Add an exportable finding packet with a chart image, caption, relevant counts, method card, selected source references, and limitations. Keep it local and do not silently include direct identifiers in broadly shared outputs.
5. Validate each interaction against analyst tasks (locate a node, trace a route, compare groups, inspect parallel records, find source evidence) with accuracy and completion time. Do not use a graph-size threshold from another study as a substitute for local acceptance.

## References and review limits

Full source details and inspected locations are in the evidence matrix above. The review prioritizes primary empirical studies and reporting/ethics guidance. It did not assess every SNA measure, community-detection method, jurisdiction-specific privacy rule, or social-science domain. Recommendations should be adapted to the actual research question, collection process, data sensitivity, and intended audience. No operational or personal corpus was examined for this review.
