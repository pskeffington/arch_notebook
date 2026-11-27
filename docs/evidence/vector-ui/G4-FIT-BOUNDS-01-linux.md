# G4-FIT-BOUNDS-01 · Fit all entity-card bounds

- **Task / owner:** SNG-DELIVERY-002, G4 viewport usability acceptance; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `03fc79fcf33dc4dcaf221ab7690697037de6f656`.
- **Source candidate:** `7713bc5a13e7f603eadfe76c9e619df1e5f2ff70`.
- **Scope:** Extend the existing `chart_fit_transform` regression in `src/main.rs` so it checks that each selected entity's rendered card rectangle remains inside the chart viewport after applying the returned fit zoom and pan.
- **Independent review:** Read-only review of the candidate returned **PASS**. The reviewer did not run tests or edit files.

## Coverage

The regression retains the empty-selection and missing-ID checks, and adds fictional entities at the normalized chart corners. It evaluates 1000×700 and 560×420 viewports with nonzero screen origins, applies the computed transform, and asserts every selected entity card fits inside the viewport. The representative cases cover a wide and a narrower chart area; they do not establish native rendering, the View-menu click path, card text overflow, or behavior at every possible window size.

## Verification

Each successful command exited 0:

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | **PASS** |
| `git diff --check` | **PASS** |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline chart_fit_ignores_missing_ids_and_contains_selected_entity_cards -- --nocapture` | **PASS**, 1 passed |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, 61 passed, 0 failed, 1 opt-in benchmark ignored |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | **PASS** |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | **PASS** |
| `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | **PASS** on retry after freeing this repository's regenerable Rust incremental cache; the initial attempt was stopped while `/tmp` had only 100 KiB free |

Release binary and installed current-user launcher SHA-256 both equal `604e95d768748e8543a91325fdc8917b7aab1dd68770b0a8b0e83ccf58f419fa`.

## Remaining acceptance

The live `VECTOR UI SPARSE 30` screen capture shows a zoomed/panned subset with stacked labels and cards outside the viewport. It does not prove a Fit all defect, identify the running process binary, or test the new Auto/All label mode. Native review still needs to identify the current candidate, invoke **Fit all**, inspect card bounds and label density, and exercise G3 Arrange/Restore. GPU, compositor, and presentation performance remain unmeasured. See the [native acceptance runbook](G3-G4-native-acceptance-runbook.md) and [existing-window capture limits](G4-DENSE-LABEL-AUTO-01-linux.md#existing-window-follow-up).
