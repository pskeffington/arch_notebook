# CHART-TEMPLATE-CONDITIONAL-RULES-01 · reusable chart formatting

- **Task / owner:** Carry conditional attribute formatting with case-local chart templates; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `82df134b114cfab7ffdb992da1a63fddb3f2ddef`.
- **Source candidate:** pending commit.
- **Scope:** Persist, apply, reapply, and remove bounded chart-template conditional rules while preserving legacy case readability and local legends.
- **Release and installed binary SHA-256:** `7f789d8fa53fb2fac444e2189767ffd266e3f49c537b2058d6175356fdf176c8`.

## Change

Saving a chart template now captures up to 96 distinct conditional attribute-formatting entries from its chart legends. Applying a template adds missing rules as visible template-owned legend blocks; active template rules take precedence over conflicting local legend rules. Reapplying first removes generated template blocks and recreates them, and deleting the template removes its generated blocks from the current and saved chart views. Existing charts and templates without the new optional fields remain readable. Template rules stay case-local and do not alter SQL or import mappings.

The 96-rule cap is explicit in the menu and save status. Applied rules are split into legend-sized blocks of at most 24 entries; the final block carries the truncation marker when the source exceeded the cap.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Conditional rules save and apply | **PASS** | `chart_templates_save_apply_and_replace_conditional_attribute_rules` covers rule-only templates, primary and additional predicates, conditional line width, template-over-local precedence, persistence, reapplication without duplication, cleanup on delete, and legacy JSON fields. |
| Rule capture/apply bounds | **PASS** | `chart_template_conditional_rule_limit_is_visible_and_bounded` proves capture stops at 96, applies four 24-entry blocks, marks truncation, and does not duplicate blocks on reapplication. |
| Full Rust suite | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 63 passed, 0 failed, 1 opt-in benchmark ignored. |
| Formatting, patch validation, check, debug and release builds | **PASS** | `cargo fmt --all -- --check`, `git diff --check`, and locked offline `cargo check`, debug build, and release build exited 0. Build artifacts are under `/tmp/arch-notebook-target`. |
| Per-user launcher, desktop shortcut, icons and helper | **PASS** | The installed release hash matches the candidate. Menu and Desktop entries match byte-for-byte and validate; the SVG, PNG and SQL automation helper match repository hashes. See [LOCAL-INSTALL-08](../deployment/LOCAL-INSTALL-08-linux.md). |
| Independent code review | **NOT RUN** | No independent reviewer was invoked for this implementation. |
| Native visual/input review | **BLOCKED** | This session cannot connect to `DISPLAY=:0`; prior candidate launch attempts report `WaylandError(Connection(NoCompositor))`. The generated legends and template precedence have headless logic coverage but no native pointer/visual acceptance. |
| IBM Analyst's Notebook parity | **NOT TESTED** | No IBM application or external schema was used. This is an Arch Notebook adaptability feature, not an IBM compatibility claim. |

## Limits

Template rules are chart formatting preferences, not analysis findings. A template captures only supported conditional attribute legend entries; the first 96 distinct entries are retained. Templates still do not transform SQL or file-import mappings. No data-source rows or relationship records are changed.
