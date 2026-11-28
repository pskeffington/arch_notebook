# Clinton corpus topology regression on current product source — 2026-11-28

## Handoff

- Task: rerun the ignored production-topology regression against the primary retained Clinton QA case.
- Repository/branch: `/tmp/arch-notebook-remote-candidate`, `codex/ui-00-chart-nested-conditions`.
- Source commit: `725efb7885eaa8934253f79f7d3ab499ecc3d9c8` (product source is unchanged from `c653f31`).
- `src/proximity.rs` SHA-256: `44e614899ff583cedcecca6f8f15cd98a6fc14fb01c306ddd34ee1780d2bdb3f`.
- Retained case SHA-256 before and after test: `6143c16d3f66964991f80184d6e2a4d165553bf9443ac3cb6bfbeea930fa255b`.
- Scope: read-only topology sampling and deterministic sampled-group crossing ordering. No private case records or identities are included here.

## Commands and observed results

| Command | Exit | Result |
| --- | ---: | --- |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | 0 | 110 passed, 0 failed, 3 ignored. |
| `ARCH_NOTEBOOK_CLINTON_CASE_PATH=/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline clinton_case_topology_sampling_stays_bounded_and_covers_hops -- --ignored --nocapture` | 0 | 1 passed, 0 failed; reported `aggregate Clinton sample crossings: 23 -> 3`. |
| `sha256sum /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json` | 0 | Case digest matched the pre-run digest; the regression only reads the case. |

## Acceptance

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Current-source full Rust suite | PASS | 110 passed, 0 failed, 3 ignored. |
| Real Clinton graph counts and topology sample | PASS, bounded | Regression asserts 2,813 entities and 6,173 relationship records; sampled groups cover the analyzed hop columns. |
| Deterministic crossing ordering | PASS | Current-source run reduced proper sampled segment crossings from 23 to 3. The synthetic and Clinton tests use the same bounded deterministic ordering path. |
| Case integrity | PASS | Case SHA-256 stayed `6143c16d...fa255b`; no case write is performed by this test. |
| Native rendering, pointer/keyboard review, and IBM visual parity | NOT TESTED | This Rust regression evaluates topology and UI data preparation, not a native desktop interaction or IBM application. |

This run confirms the bounded topology regression on the current product source and real QA graph. It does not validate email extraction completeness, semantic findings, or human/legal conclusions. See [prior crossing-reduction implementation evidence](PROXIMITY-MAP-CROSSING-REDUCTION-01.md) for the fixture, method, and earlier acceptance run.
