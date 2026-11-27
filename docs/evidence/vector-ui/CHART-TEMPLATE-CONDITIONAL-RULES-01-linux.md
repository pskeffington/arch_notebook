# CHART-TEMPLATE-CONDITIONAL-RULES-01 · reusable chart formatting

- **Task / owner:** Carry conditional attribute formatting with case-local chart templates; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `82df134b114cfab7ffdb992da1a63fddb3f2ddef`.
- **Feature introduction:** `f43da45eed456ffd1aa955c1393e9e553a574502`.
- **Review follow-up fix:** `ada63a467e85b00f9186c049947299275abfec11`.
- **Scope:** Persist, apply, reapply, and remove bounded chart-template conditional rules while preserving legacy case readability and local legends, including exact rules already present in local legends.
- **Release and installed binary SHA-256:** `fe94ef08e1e9e1583551a535f8dec74614931fd78736658edaeddb002d08f4e1`.

## Change

Saving a chart template now captures up to 96 distinct conditional attribute-formatting entries from its chart legends. Applying a template adds all captured rules as visible template-owned legend blocks, including when an identical rule is already present in a local legend; active template rules take precedence over conflicting local legend rules. Reapplying first removes generated template blocks and recreates them, and deleting the template removes its generated blocks from the current and saved chart views. Existing charts and templates without the new optional fields remain readable. Template rules stay case-local and do not alter SQL or import mappings.

The 96-rule cap is explicit in the menu and save status. Applied rules are split into legend-sized blocks of at most 24 entries; the final block carries the truncation marker when the source exceeded the cap.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Conditional rules save and apply | **PASS** | `chart_templates_save_apply_and_replace_conditional_attribute_rules` covers rule-only templates, primary and additional predicates, conditional line width, template-over-local precedence, an exact matching local rule plus a conflicting local rule, persistence, reapplication without duplication, cleanup on delete, and legacy JSON fields. |
| Rule capture/apply bounds | **PASS** | `chart_template_conditional_rule_limit_is_visible_and_bounded` proves capture stops at 96, applies four 24-entry blocks, marks truncation, and does not duplicate blocks on reapplication. |
| Full Rust suite | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 63 passed, 0 failed, 1 opt-in benchmark ignored. |
| Formatting, patch validation, check, debug and release builds | **PASS** | `cargo fmt --all -- --check`, `git diff --check`, and locked offline `cargo check`, debug build, and release build exited 0. Build artifacts are under `/tmp/arch-notebook-target`. |
| Per-user launcher, desktop shortcut, icons and helper | **PASS** | The installed release hash matches the updated template-ownership candidate. Menu and Desktop entries match byte-for-byte and validate; the SVG, PNG and SQL automation helper match repository hashes. See [LOCAL-INSTALL-08](../deployment/LOCAL-INSTALL-08-linux.md). |
| Exact-head cross-platform CI | **NOT RUN for the review follow-up** | Workflow run [45985015963](https://github.com/pskeffington/arch_notebook/actions/runs/45985015963) passed all six jobs on the prior candidate `fa9002a2bd86425ef9b61179ab6ab1747e1e3106`; it predates the ownership fix. |
| Review follow-up | **PASS** | A read-only source reviewer confirmed the exact-match case now retains template ownership and precedence, reapplication replaces generated entries without duplicates, and no ordering issue remains. Review scope was source-only; native placement, runtime multi-tab behavior, tests, and builds were not run by the reviewer. |
| Native visual/input review | **BLOCKED** | This session cannot connect to `DISPLAY=:0`; prior candidate launch attempts report `WaylandError(Connection(NoCompositor))`. The generated legends and template precedence have headless logic coverage but no native pointer/visual acceptance. |
| IBM Analyst's Notebook parity | **NOT TESTED** | No IBM application or external schema was used. This is an Arch Notebook adaptability feature, not an IBM compatibility claim. |

## Limits

Template rules are chart formatting preferences, not analysis findings. A template captures only supported conditional attribute legend entries; the first 96 distinct entries are retained. Templates still do not transform SQL or file-import mappings. No data-source rows or relationship records are changed.
