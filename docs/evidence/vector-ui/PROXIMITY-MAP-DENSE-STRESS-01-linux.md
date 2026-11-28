# Dense topology map production UI stress — Linux evidence

- **Task / owner:** Validate production topology rendering and pre-sampling culling near the 48-group map limit; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `2feca0710b564e28be72bc5f129e8223c2c8adf6`.
- **Changed paths:** `src/main.rs`, `docs/build-acceptance.md`, this report.

## Fictional stress fixture

The test builds 48 entities as one origin, 24 independent groups at hop 1, and 23 independent groups at hop 2. The origin contacts all hop-1 groups (24 ties); each hop-1 group contacts all hop-2 groups (552 ties). Because same-layer entities have no direct ties, the analysis produces 48 distinct groups and 576 distinct cross-group interactions, all represented in the bounded map.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Production Analysis panel at dense scale | PASS, headless | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline dense_proximity_map_culls_before_sampling_without_losing_visible_routes -- --nocapture`, exit 0. The actual UI rendered 48 groups / 576 pairs at 1440×1600; emitted route shapes confirmed a 291×193-point map clip, which the exhaustive culling model asserts against. |
| Hull culling completeness | PASS | Exhaustive check over all 576 route samples found no route whose expanded sampled bounds intersected the viewport but whose endpoint/control-point hull was culled. Non-finite input remains conservative in the focused helper regression. |
| Paint count | PASS | 156 routes passed the viewport hull check; 420 were skipped before generating 13 samples; the production map emitted exactly 1,872 line segments (156×12). |
| Production interactions | PASS | The focused full-suite topology regression continues to exercise route selection and node-click priority; synthetic stress does not replace that pointer test. |
| Retained Clinton real-corpus UI | PASS, headless | The retained 2,813-entity / 6,173-record Clinton production UI regression passed on the same route-culling implementation; 48 groups / 47 sampled pairs / 1,315 represented records. See [CLINTON-UI-01](PROXIMITY-MAP-CLINTON-UI-01-linux.md) and [viewport culling run](PROXIMITY-MAP-VIEWPORT-CULL-01-linux.md). |
| Full suite / compile / build / format | PASS | 73 passed, 0 failed, 3 ignored; locked offline `cargo check` and debug build, `cargo fmt --all -- --check`, and `git diff --check` pass. |
| Independent topology source review | PASS | Reviewer confirmed the quadratic hull conservatively includes crossings, margins cover arrows and route-pick tolerance, and node clicks retain priority. |
| Native visual / GPU / exact-head CI | NOT RUN / PENDING | The stress test is headless. Build workflow does not run on this non-main branch push. |
