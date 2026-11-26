# Social network graphing and delivery roadmap

- **Task:** SNG-DELIVERY-001
- **Owner:** Orchestrator (`software-delivery-orchestrator`)
- **Planning baseline:** `5034a4f0a278597ae80372494be7d537b9331cff` on `codex/ui-00-chart-baseline`
- **G1 implementation baseline:** `008537813d60dd76db0732b06f87b6266a02603a`
- **G1 candidate commit:** `a5208a0e3aa8c0fe3b3a1e3f02986e45e97b919a`
- **Status:** G1/G2 and code-level G3 checks passed; native method-card interaction remains blocked; no release or deployment approval.
**Research input:** [Social network graphing and delivery review](../research/social-network-graphing-and-delivery-review.md).

The application is a generic entity/link notebook. A chart is a social network only when its entities and relationship records represent defined social actors and relations. Each view must describe the graph it actually analyzes; layout position and structural measures do not establish real-world influence, intent, or causation.

The implementation baseline advanced after planning when the owner committed the ANX worker regression test. The implementation agent also found unrelated local edits in `README.md`, `docs/build-acceptance.md`, `docs/evidence/vector-ui/UI-INGEST-NAV-01-linux.md`, and untracked native-render evidence. These remain outside SNG-DELIVERY-001 and must be preserved.

## Phase gates

| Gate | Stage and owner | Scope / dependency | Acceptance gate | State |
| --- | --- | --- | --- | --- |
| G0 | Research and scope — researcher, then planner | Inspect repository behavior and the literature review; establish the user task, graph definitions, baseline and file ownership. No implementation. | Baseline commit and dirty work are recorded; relationship semantics are not inferred; bounded task, risks, dependencies, and acceptance criteria are approved. | **PASS** for this roadmap: literature review and current graph-feature inventory are available. |
| G1 | Analysis Method Card — implementer (`code-implementation`) | Depends on G0. Implement only the bounded feature below in the existing Analysis sidebar and relevant operator documentation. | Summary matches the active chart; counts and view assumptions are accurate on a fictional fixture; chart switching and edits cannot leave stale results; copying summary metadata does not modify the case, evidence, or SQL source. | **PASS** |
| G2 | Integration and documentation — integrator (`build-integration`) and documenter (`documentation-maintenance`) | Depends on G1. Integrate the patch, run repository-prescribed formatting/check/build gates, and align workflow docs with observed behavior. | `cargo fmt --all -- --check`, locked offline `cargo check`, debug build, and release build pass using `/tmp/arch-notebook-target`; commands and exit results are recorded; no undocumented behavior or new dependency. | **PASS** |
| G3 | Independent verification and review — verifier, code reviewer, security reviewer | Depends on G2 and a candidate change set. Reviewers must not approve their own implementation. | Fictional fixtures cover isolates, parallel links, directed/undirected links, self-links, and dated/undated records; UI refresh and copy behavior are inspected; no unresolved blocking correctness, privacy, or maintainability finding. Any inaccessible native interaction is recorded as **BLOCKED**, not passed. | **BLOCKED: native interaction** |
| G4 | Finding-packet discovery — planner/researcher; later implementer only after explicit gate approval | Depends on G3 and analyst-task feedback. Specify a local export packet with chart image, method card, selected provenance, caption and limits. Evaluate an adjacency matrix only against a concrete dense-network task. | Separate task contract defines audience, sensitive-data handling, format, reproducibility and acceptance measures. No image/matrix export is included in G1. | **DEFERRED** |
| G5 | Release readiness — release controller (`release-readiness`); human release owner | Depends on required G2/G3 evidence and complete documentation. Separate from integration. | Release dossier and artifacts pass the applicable repository gates. Any deployment or publication requires explicit human approval; this roadmap grants neither. | **NOT STARTED** |

Dependency order: **G0 → G1 → G2 → G3 → G4**. G5 is a separate release decision after required evidence and owner approval. G4 is not a prerequisite for G1 or a release claim.

## G1 implementation contract — Analysis Method Card

**Objective:** Make the scope and mechanical assumptions behind the current chart analysis easy to read and copy into an analyst handoff.

**In scope:**

- Add a compact, read-only **Analysis Method Card** in the current Analysis sidebar for the active chart.
- Report only defensible, app-computed structural facts: active chart, entity count, relationship-record count, directed and undirected record counts, distinct endpoint-pair count with a parallel-record note, and dated versus undated record coverage. Include relationship-type or source-coverage summaries only when the counting rule is unambiguous and the UI remains readable.
- State assumptions for each existing analysis: weak components and articulation entities ignore direction and self-links and operate on unique neighboring entities; geodesic layers ignore direction, weight, date, and relationship type and treat each traversed link as one hop; shortest-path results follow the active endpoint, direction, time-flow, and timeline-range controls.
- If a copy action is included, copy the method and structural summary only by default. Do not copy entity names, IDs, evidence contents, or source notes unless a later approved task explicitly scopes that behavior.
- Refresh or invalidate the card when the active chart or relevant chart data changes. Do not persist the card into the case or alter source records.
- Update `docs/notebook-workflows.md` to describe only behavior that is implemented and observed.

**Out of scope:** statistical social-network modeling, inferred node roles, causal or behavioral claims, new centrality measures, automatic relationship interpretation, adjacency matrices, chart-image/report export, case-schema changes, network services, telemetry, deployment, or release publication.

**Acceptance checks for G1/G3:**

1. On a fictional fixture, entity and relationship counts reconcile with the active chart, including isolates and parallel records; direction counts remain record counts rather than collapsed pair counts.
2. Date coverage reports invalid/empty dates as undated and does not imply that missing records mean absent real-world ties.
3. Each method statement matches the current algorithm and controls; labels do not imply a social meaning, popularity, control, or causal conclusion.
4. Switching charts or editing graph structure refreshes the displayed summary. A copied summary contains only approved method/aggregate metadata and does not mutate the case, source database, evidence records, or snapshots.
5. Existing entity identity, repeated relationship records, source references, and analyst notes remain available through the current record views.

## Ownership, worktree, and rollback

- The orchestrator owns the task contract, status, sequencing, and final evidence handoff. Assign one implementer to G1; keep integration, verification, security, and code review to distinct owners.
- At the planning baseline, `src/main.rs` has an uncommitted user-owned ANX worker test. The implementer must preserve it and must not sweep it into the G1 change set. Work in an isolated branch/worktree when practical; otherwise capture and review the baseline diff before editing.
- G1 is additive UI and derived summary data with no persistence migration. Rollback is to remove the card and its derived helpers/docs; source records, cases, and SQL snapshots must remain unchanged.
- Keep build artifacts in `/tmp/arch-notebook-target`. Use only locked, offline repository commands. No external dependency or network integration is authorized by this roadmap.

## Gate evidence — 2026-11-26

- **G1 PASS:** `src/main.rs` adds an on-demand, transient active-chart method card; a fictional regression case checks isolates, directed/undirected records, self/parallel pairs, missing endpoints, date classes, clipboard redaction, and unchanged case serialization. `docs/notebook-workflows.md` documents count definitions and method assumptions.
- **G2 PASS:** `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, debug build, release build, and `git diff --check` each exited 0. No build warnings were reported.
- **G3 automated PASS:** `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0 with 25 passed and 0 failed, including `analysis_method_card_counts_records_pairs_isolates_and_date_quality` and `network_analysis_invalidation_discards_a_cached_method_card`.
- **G3 code-review PASS:** independent reviewer found no blocking algorithm, cache, clipboard, or maintainability issue. A duplicated operator instruction was consolidated and invalidation wording narrowed after review.
- **G3 security PASS:** independent security review found the copy payload contains only computed counts and fixed method descriptions. Aggregate counts can still be sensitive for a small known network; copy is an explicit user action.
- **G3 native UI BLOCKED:** existing [UI-00 render-only evidence](../evidence/vector-ui/UI-00-native-render-20261126.md) records host input injection as unavailable. It predates the method card, so the new card's click, refresh, and clipboard interaction have not been manually exercised. Do not report native interaction acceptance as passed.
- **Remaining G3 risk:** runtime visual/input acceptance for the new card is still required before claiming the complete user-facing gate. The previous native screenshot confirms only the earlier chart/navigation shell, not the method-card build.

## Risks and stop conditions

| Risk | Mitigation / stop condition |
| --- | --- |
| A count silently collapses or multiplies parallel records, self-links, or missing endpoints. | Define the denominator per field; omit any count whose semantics are ambiguous. Stop G1 if the count cannot be reconciled on the fixture. |
| A method note overstates what the selected algorithm uses. | Derive language from the implementation and document each view separately. Block G3 for any mismatch. |
| Expensive recomputation or stale values degrade interaction. | Compute on demand or cache by active-chart revision; verify invalidation on edits and chart switches. Stop if summary recomputation harms normal chart use. |
| A copy/export path discloses sensitive names, identifiers, or evidence. | Keep G1 copy output to aggregate/method metadata only. Stop if payload scope cannot be demonstrated and reviewed. |
| The chart is treated as a complete or representative social network without evidence. | Label the chart as a constructed graph; require boundary, source, missingness, and time context before making population-level claims. |
| Required review or native UI evidence is unavailable. | Record **BLOCKED** and the exact missing evidence; do not convert unavailable checks to PASS or advance the release gate. |
| The same implementation blocker persists. | Limit repair to three attempts per distinct blocker, then return to the orchestrator with evidence and a proposed next action. |

## Required handoff per gate

Record task and owner, source commit, changed paths, exact commands and exit codes, evidence references, `PASS`/`FAIL`/`BLOCKED` result, remaining risks, rollback status, and next owner. A release-readiness result is not release approval.
