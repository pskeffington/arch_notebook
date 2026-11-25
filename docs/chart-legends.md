# Chart legends

**Task:** ANB-CHART-LEGEND-01
**Owner:** Codex, single-session implementation
**Baseline:** `7677248ebbc8732a38aa8c8187f2fe61c1799c7a` (current worktree also contains unrelated user changes)
**Scope:** `src/main.rs`, this document, and the Analyst's Notebook compatibility map.

## Implementation

Add a standalone chart legend that follows the existing chart-label lifecycle and records common entity and relationship conventions. The legend must be useful as a chart annotation, remain separate from entity and relationship records, and persist locally.

Create a chart legend with **＋ Legend** on the chart toolbar. It is placed in a low-occupancy in-bounds area on the active chart, selected, and populated with up to 24 distinct entity types and relationship labels. If the chart has more categories, the legend marks that some were omitted. The Analysis pane edits the title and each entry's description, symbol, and color. **Populate from chart** rebuilds entries from the current chart; **＋ Entry** adds a manual marker entry, and **×** removes an entry. Drag the legend on the canvas to move it; delete it with the regular chart-item delete action.

Legend entries annotate the chart only. Editing a legend does not change entity or relationship records. Legends use the existing chart label's local case persistence, chart switching, duplication, and Arch Notebook GraphML graph metadata path. GraphML imports reject legends with more than 24 entries or entry descriptions over 4,096 bytes.

## Acceptance criteria

- An analyst can create a legend populated from up to 24 active-chart entity types and relationship labels; omitted categories are identified.
- An analyst can change the title, edit descriptions, choose entry colors, add a manual entry, and remove an entry.
- The legend renders on the chart, can be selected, repositioned, duplicated with its chart item, and deleted without changing analytical data.
- Case save/load, chart switching, chart duplication, and GraphML's Arch Notebook chart metadata preserve legends. Existing label-only cases and GraphML remain readable.
- Input validation bounds imported legend entry counts and text sizes and rejects invalid serialized colors or coordinates without partially applying the import.

## Design record

Represent a legend as an optional collection of typed entries on the existing `ChartLabel` canvas item. Empty entries preserve current label behavior and serialized compatibility; an explicit defaulted legend marker selects legend rendering and editing. This keeps chart placement, selection, dragging, copy, chart switching, and GraphML metadata on the established path. A legend entry carries its description, symbol class (entity, relationship, or generic marker), and RGB color. Chart population uses one entry per distinct entity type and relationship label, with the first matching entity's saved color and the current link color as defaults. Manual edits affect the legend only and never update source entities or links.

## Attribute class symbols

**Task:** ANB-CHART-ATTRIBUTE-SYMBOLS-01
**Owner:** Codex, single-session implementation
**Baseline:** `7677248ebbc8732a38aa8c8187f2fe61c1799c7a`
**Scope:** `src/main.rs`, this document, and the Analyst's Notebook compatibility map.

IBM defines an attribute class with a symbol and value; chart items can display the symbol and value, and chart properties can assign a symbol per attribute class ([attribute definitions](https://docs.i2group.com/anb/10.0.2/defining_chart_property_attributes.html), [setting display icons and symbols](https://docs.i2group.com/anb/10.1.2/setting_display_icon.html)). Arch Notebook applies this to entity and relationship attributes through the chart legend:

- **Populate from chart** now includes distinct attribute classes from entities and relationships alongside entity types and relationship labels.
- Select an attribute circle, square, triangle, or diamond; set its color; enter the matching attribute class; and enable **Show on chart**.
- Matching values appear below their chart items with the class symbol and color. At most two values display per item, followed by a count when more configured classes match.
- Enable **Color matching items** to apply the entry color to entities and relationships using an ordered attribute condition: any value, set/absent, equal/not equal, greater than, at least, less than, at most, before date, on or before date, after date, on or after date, contains/does not contain, starts with, or ends with. Add up to seven conditions for other attribute classes, then choose **Match all (AND)** or **Match any (OR)** for those additional rules. The primary rule always must match; with AND every additional rule must match, and with OR at least one must match; an empty additional attribute class never matches. Numeric rules require both the attribute value and threshold to parse as finite numbers; other values do not match. Date rules require a valid ISO calendar date in `YYYY-MM-DD` form on both sides; timestamps and other formats do not match. New text comparisons are case-sensitive; legacy exact-value entries retain their prior case-insensitive comparison. The first matching legend entry in order applies, so place specific rules before broader rules.
- The **Override matching link width** option applies a 0.5–8 pt width to relationships matched by that entry, including arrowheads, while retaining the link confidence pattern; entity rendering is unaffected. The legend entry carries the attribute-class key, marker style, display setting, and optional conditional color and link-width rules. Case JSON, chart switching and duplication, and the existing Arch Notebook GraphML chart metadata preserve these settings. Older entries default to non-attribute markers with attribute display and conditional formatting off.
- Attribute display and conditional colors are chart formatting; they do not modify entity attributes, relationship attributes, or SQL snapshots. Rules apply only to the chart containing the legend. Selection, path, source, group, and time-focus styles retain higher display priority.
- GraphML preserves the conditional rule in Arch Notebook chart metadata while its standard node colors continue to represent saved entity colors. Other GraphML tools may ignore the Arch-specific rule metadata.

Acceptance for this increment is that an analyst can define and enable a class symbol, see that colored symbol and value on matching entities and relationships, optionally apply a condition-based item color, edit formatting independently from underlying item data, and retain it across case/GraphML/chart operations. Imported GraphML still bounds legend count and rule text sizes and rejects conditional link widths outside 0.5–8 pt.

This is not complete IBM conditional formatting or attribute parity. Rules currently support one primary condition, one additional AND or OR group of up to seven conditions, a color action, and an optional matching-link width override. Nested or mixed AND/OR groups, lookup-table, type, visibility, badge, dash-style, and gradual-scale styles; attribute data types and value formatting; semantic types; custom image symbols; fonts; time zones; page-relative print positioning; and IBM-native ANX legend styling remain future work. The ANX importer/exporter preserves Arch Notebook legend settings as custom chart-label attributes; it does not recreate native ANX legend formatting, and its XML has not been validated against IBM's XSD. IBM describes ordered conditional rules and lookup conditions including equality, text matching, and attribute presence ([conditional formatting](https://docs.i2group.com/anb/10.0.2/about_conditional_formatting.html), [lookup-table styling](https://docs.i2group.com/anb/10.1.1/setting_formatting_style_lookup_table.html)).

## Evidence

IBM describes legends as chart keys that can be populated from chart items or built manually, with entries for entity icons, lines, links, fonts, time zones, and other conventions: [Work with legends](https://docs.i2group.com/anb/10.0.2/about_legends.html). The local baseline already persists standalone chart labels per chart and embeds them in GraphML graph metadata.

## Verification and remaining work

The current source passed `cargo fmt --all -- --check`, `git diff --check`, `cargo check --locked --offline`, `cargo build --locked --offline`, and `cargo build --release --locked --offline`. A prior local Linux/X11 visual review with a synthetic saved case confirmed the original legend title, type-specific entity symbols, and relationship lines render within the chart at 1440×900 with readable spacing. The attribute controls and conditional colors have not yet been visually reviewed. No tests were run. New serialization fields are optional/defaulted, and the legacy exact-value flag remains readable. ANX compatibility requires IBM schema/fixtures and is outside this slice.
