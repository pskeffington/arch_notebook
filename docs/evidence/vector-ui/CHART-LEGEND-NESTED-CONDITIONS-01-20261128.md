# CHART-LEGEND-NESTED-CONDITIONS-01 · 2026-11-28

- **Task / owner:** Add bounded nested AND/OR conditional formatting to chart legends; Codex single-session implementation.
- **Repository:** `/home/sysadmin/github/arch_notebook`, branch `codex/ui-00-chart-baseline`.
- **Baseline:** Remote clean source commit `cbad7eca7ffacd3d3f3a2d785abfdd9423278a95`; feature branch base `d4257d48eb3a7d2ed4bb060b989dc08796d17709`. The baseline Rust blob is `6f731733243a96c7f6335bb004ad10500f7e1b17` (2,375,707 bytes).
- **Scope:** `src/main.rs`, `docs/chart-legends.md`, `docs/analyst-notebook-parity.md`, `docs/political-email-qa-sources.md`, `README.md`, and this evidence.

## Change

Legend conditions now optionally store a recursive expression tree with `Condition`, `All`, and `Any` nodes. The editor lets an analyst opt in, select each group's AND/OR operator, add/remove conditions and subgroups, and return to flat-rule editing. Expressions are bounded to four group levels and eight conditions; import validation rejects empty groups, blank attribute names, oversize text, excess depth, and excessive leaf count.

Legacy case JSON without the optional expression remains unchanged in behavior. The pre-existing primary/additional rule fields are retained. Older builds ignore the new expression and evaluate legacy fields; saving a case after editing grouped rules in an older build can lose grouped semantics. IBM documentation reviewed here establishes ordered conditional rules and lookup conditions, but does not establish nested mixed groups. This is an Arch Notebook feature, not an IBM-parity claim.

## Acceptance

| Gate | Result | Evidence |
|---|---|---|
| Mixed nested AND/OR truth table | PASS | `grouped_chart_legend_conditions_match_mixed_boolean_expression`: true and false branches, case-insensitive attribute lookup, numeric comparisons, and serde round-trip. |
| Bounds and invalid structures | PASS | `grouped_chart_legend_conditions_enforce_depth_and_leaf_limits`: depth over four, empty group, matcher fail-closed behavior, and leaf-limit count. Import validation enforces the eight-leaf cap. |
| Legacy saved-case compatibility | PASS | `grouped_chart_legend_conditions_preserve_legacy_saved_cases`: old JSON without expression deserializes and retains the legacy match. |
| Focused offline tests | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline grouped_chart_legend_conditions -- --nocapture`, exit 0; 3 passed. |
| Full offline suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`, exit 0; 78 passed, 0 failed, 3 ignored. |
| Offline check and debug build | PASS | Both locked offline commands exited 0. |
| Formatting and diff | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Optimized release build | BLOCKED | Two release-build invocations caused the Cargo process to disappear while the command runner kept waiting and produced no exit code. Existing binary is dated Nov. 27 and cannot validate this source. Do not treat as pass. |
| Native grouped-editor visual/input review | NOT TESTED | No native interaction was run for this candidate. |
| IBM ANX/schema/UI compatibility | NOT TESTED | No IBM schema, fixture, or running IBM application was used. |
| Political-email source acquisition/import | NOT DONE | Ten candidate record sets are documented in `docs/political-email-qa-sources.md`; no source records were downloaded or imported. |
| Remote CI | PENDING | Candidate is being published to a separate non-main feature branch; exact-head Actions must still be dispatched and checked. |

## Research source notes

The feature patch was reapplied to the intact remote source commit, after confirming its complete blob SHA/size; no truncation markers remain in the candidate source. On 2026-11-28, IBM's official conditional-formatting documentation was checked at `https://docs.i2group.com/anb/10.0.2/about_conditional_formatting.html`; it describes ordered conditional rules and lookup conditions. It did not substantiate nested mixed boolean groups. The public political-record source manifest is separately documented at `docs/political-email-qa-sources.md`; source status and limitations are included entry by entry.

## Next

Retry the optimized release build in a functioning Cargo runner, inspect the grouped editor in a native desktop session, then push to this non-main branch and collect exact-head Actions results. Source corpus downloads and Qwen/content review remain separate tasks.
