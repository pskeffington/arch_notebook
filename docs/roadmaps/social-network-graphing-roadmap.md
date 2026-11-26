# Social network graphing and delivery roadmap

- **Task:** SNG-DELIVERY-001
- **Owner:** Orchestrator (`software-delivery-orchestrator`)
- **Planning baseline:** `5034a4f0a278597ae80372494be7d537b9331cff` on `codex/ui-00-chart-baseline`
- **G1 implementation baseline:** `008537813d60dd76db0732b06f87b6266a02603a`
- **G1 candidate commit:** `a5208a0e3aa8c0fe3b3a1e3f02986e45e97b919a`
- **Status:** G1/G2 PASS and automated G3 fixture checks PASS. G3 remains BLOCKED on native method-card interaction and independently verifiable review evidence; no release or deployment approval.
**Research input:** [Social network graphing and delivery review](../research/social-network-graphing-and-delivery-review.md).

The application is a generic entity/link notebook. A chart is a social network only when its entities and relationship records represent defined social actors and relations. Each view must describe the graph it actually analyzes; layout position and structural measures do not establish real-world influence, intent, or causation.

The implementation baseline advanced after planning when the ANX worker regression test was committed. At planning time, the implementation agent also found unrelated README, build-acceptance, prior UI evidence, and native-render changes; those were preserved and committed before this roadmap was added. They remain outside SNG-DELIVERY-001.

## Phase gates

| Gate | Stage and owner | Scope / dependency | Acceptance gate | State |
| --- | --- | --- | --- | --- |
| G0 | Research and scope — researcher, then planner | Inspect repository behavior and the literature review; establish the user task, graph definitions, baseline and file ownership. No implementation. | Baseline commit and dirty work are recorded; relationship semantics are not inferred; bounded task, risks, dependencies, and acceptance criteria are approved. | **PASS** for this roadmap: literature review and current graph-feature inventory are available. |
| G1 | Analysis Method Card — implementer (`code-implementation`) | Depends on G0. Implement only the bounded feature below in the existing Analysis sidebar and relevant operator documentation. | Summary matches the active chart; counts and view assumptions are accurate on a fictional fixture; chart switching and edits cannot leave stale results; copying summary metadata does not modify the case, evidence, or SQL source. | **PASS** |
| G2 | Integration and documentation — integrator (`build-integration`) and documenter (`documentation-maintenance`) | Depends on G1. Integrate the patch, run repository-prescribed formatting/check/build gates, and align workflow docs with observed behavior. | `cargo fmt --all -- --check`, locked offline `cargo check`, debug build, and release build pass using `/tmp/arch-notebook-target`; commands and exit results are recorded; no undocumented behavior or new dependency. | **PASS** |
| G3 | Independent verification and review — verifier, code reviewer, security reviewer | Depends on G2 and a candidate change set. Reviewers must not approve their own implementation. | Fictional fixtures cover isolates, parallel links, directed/undirected links, self-links, and dated/undated records; UI refresh and copy behavior are inspected; no unresolved blocking correctness, privacy, or maintainability finding. Any inaccessible native interaction is recorded as **BLOCKED**, not passed. | **BLOCKED:** native interaction; independent review evidence is not present in the checked-in handoff |
| G4 | Local finding-packet export — implementation session | Depends on G1 summary semantics. Adds Linux CLI export of a selected persisted chart as editable finding template + deterministic SVG overview; no inferred findings or release. | Default redaction; per-field explicit opt-ins; source case unchanged; full chart scope stated; stable output; chart ambiguity/missing endpoints and existing output fail closed; restrictive output permissions; offline tests/builds. Independent code/security review remains a separate gate. | **PASS for automated Linux acceptance**; independent code/security review is **BLOCKED/not evidenced**. Packet directory publication is explicitly unsupported on non-Linux platforms pending portable no-replace implementation. Native G3 UI interaction remains **BLOCKED** separately. |
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
- **G4 automated Linux acceptance PASS:** final formatting, diff check, 27 Rust tests, locked offline check, debug build, and release build exited 0. The current release binary passed `scripts/test-analysis-packet-acceptance.py` (exit 0; SHA-256 `d02aca2aa4f59a229f7f12c533ca8bcb9abd7782d2d94ec7d4ff43eca5fc99c4`). Report: `/tmp/arch-notebook-packet-acceptance-20261126-05/acceptance-report.json` (SHA-256 `e9385a1b42d92f971d358a17b296bba6a86853e82406fc88101791d0816b8682`). SVG parsed and rendered to 1200×800; PNG SHA-256 `390ff37c46ba668f1597e50c6d7fac13e9eefd527424acd4bb0beb08bd2985a1`. The harness checks fixture/source immutability, default redaction and each opt-in field, chart selection/counts, duplicate JSON rejection, existing-destination refusal, valid SVG, parallel edges, direction/self-loop cues, and private Unix permissions.
- **G4 implementation-owner review:** reviewed redaction and XML/Markdown escaping, chart selection, missing-endpoint rejection, destination no-replace publication, restrictive permissions, and rendered overview. No independent code/security reviewer ran in this session; independent review is not claimed. Non-Linux export is explicitly unsupported and was not cross-compiled.
- **G3 code/security review provenance:** the roadmap previously recorded independent code-review and security-review passes, but no separate handoff/report artifact is present in the repository. This session performed implementation-owner code-path self-review only; the earlier statements are not treated as independently verified here.
- **G3 native UI BLOCKED:** existing [UI-00 render-only evidence](../evidence/vector-ui/UI-00-native-render-20261126.md) records host input injection as unavailable. It predates the method card, so the new card's click, refresh, and clipboard interaction have not been manually exercised. Do not report native interaction acceptance as passed.
- **UI-01 geometry and picking follow-up:** the recorded parallel-link lane collision is repaired; the fixture checks non-overlapping routes and pointer-to-exact-record resolution for every parallel/reversed link, and the full 27-test offline suite passes. See [UI-01 lane and selection evidence](../evidence/vector-ui/UI-01-parallel-lanes-linux.md). This source-level result does not satisfy the native UI-00 interaction/visual gate or claim IBM parity.
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
