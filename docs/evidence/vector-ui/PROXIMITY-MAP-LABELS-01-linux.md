# PROXIMITY-MAP-LABELS-01 — singular-aware topology metrics

- **Task / owner:** Improve readability of topology graph metrics; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `fd122be7b7c119d3b2d8787a60e1fb4417023e2b`.
- **Candidate source:** `0f5871e` (`Pluralize topology metrics`).
- **Candidate binary:** Linux debug, SHA-256 `84787c5aa2daec0e3d12e5ff62d46506bc456e879f5d4fcede68047d9c57a65d`.

## Change

Topology labels now match singular and plural forms for entities, records, endpoint pairs, groups, and hop layers. This applies to the hop summary, layer rows, contact group map and list, contact line hover/detail summaries, and proxy metrics. Existing direction and exact-count information is retained. A headless production UI regression checks visible labels including `1 entity · 0 internal records` and `1 record / 1 endpoint pair`.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Focused headless topology UI regression | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` passed with singular entity, record, and endpoint-pair assertions. |
| Full offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 69 passed, 0 failed, 1 opt-in benchmark ignored. |
| Locked debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0. |
| Formatting and diff validation | PASS | `cargo fmt --all` and `git diff --check` exited 0. |
| Exact-head cross-platform CI | PENDING | Push failed because `github.com` DNS resolution was unavailable. Candidate is committed locally on the non-main feature branch; retry push and verify exact-head CI. |
| Native visual review | NOT TESTED | No native pointer, typography, contrast, or map-density review was performed for this candidate. |

This change only validates label text and map behavior through the headless UI fixture. It does not establish Clinton-corpus graph readability or IBM Analyst's Notebook visual parity.
