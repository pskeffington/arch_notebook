# PROXIMITY-MAP-RENDER-CLEANUP-01 — improve dense topology rendering

- **Task / owner:** Clean up hop-group topology rendering for analyst review; implementation session.
- **Repository / branch / baseline:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline` / `b087dcf6bfd84c750d0f5061591bf5dbdb117cb2`.
- **Candidate source:** recorded after commit.
- **Scope:** `src/main.rs`, this evidence note, and `docs/build-acceptance.md`.

## Rendering changes

Hop columns and groups have more separation. Group nodes scale with member count, display both group number and member count, and keep a larger pointer target. Proxy rings remain distinct. Contact strokes use deterministic lanes and arrowheads sit outside the nodes, making multi-record contacts and direction easier to distinguish. The sample bound, exact-record interaction selection, and complete virtualized contact index are unchanged.

## Acceptance

| Check | Result | Evidence |
| --- | --- | --- |
| Production headless analysis UI | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` passed and continues to exercise group click, directional ties, line selection, and exact-record navigation. |
| Retained Clinton production topology UI | PASS | `clinton_case_topology_map_renders_at_real_corpus_scale` passed at 1440×900 against the local case (2,813 entities, 6,173 records, 10 hops, 48 sampled groups). |
| Retained Clinton topology sampling | PASS | `clinton_case_topology_sampling_stays_bounded_and_covers_hops` passed; the 48-group sample spans all hops and the full interaction index remains intact. |
| Native pointer/display review | BLOCKED | Sandbox cannot access X11/Wayland for an interactive screenshot review; the headless geometry does not establish visual acceptance. |
| IBM Analyst's Notebook visual parity | NOT TESTED | IBM application unavailable. |

All corpus UI tests read the retained local case and do not include names or IDs in Git. Native visual review remains a required follow-up.
