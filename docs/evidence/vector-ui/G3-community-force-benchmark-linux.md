# G3 community-aware force layout — Linux evidence

- **Task/owner:** G3 layout performance follow-up / Codex implementation
- **Source baseline:** `95da004b26dd2d067a05c808a5bba60cd3ef1651` on `codex/ui-00-chart-baseline`
- **Candidate:** working tree including the single-community spread regression and revised forces.
- **Fixture source:** `scripts/generate-chart-fixtures.py`; deterministic fictional sparse, hub, disconnected, and dense graphs.
- **Corrected raw data:** [25-sample JSON](artifacts/G3-community-force-25samples-corrected.json), SHA-256 `87b0ccbad6d09ebe3f379a1f77d2522064ffc08bf2f3e13533facb8d3d78c02e`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `e42240f90edfa058d78304ab549e45f8237d426dcba7e25ae8995bb6c7124e1a`.
- **Host:** Linux x86_64, kernel `7.3.5-zen1-1-zen`, glibc 2.44.

## Automated checks

| Check | Result |
| --- | --- |
| `cargo fmt --all` | **PASS**, exit 0 |
| `git diff --check` | **PASS**, exit 0 |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, 46 passed, 0 failed |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | **PASS**, exit 0 |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | **PASS**, exit 0 |
| `python3 scripts/benchmark-community-layout.py --binary /tmp/arch-notebook-target/release/arch-notebook --output /tmp/arch-notebook-g3-layout-benchmark-v2.json --iterations 20 --samples 25` | **PASS**, 12 cases, exit 0 |

The corrected timings below are medians and p95s over 25 samples after one warmup. Community detection is a single sample; it is not included in repeated layout timings.

| Fixture | Nodes | Relationship records | Communities | Community detection | Layout median | Layout p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Sparse | 1,000 | 3,000 | 61 | 3.36 ms | 12.74 ms | 17.33 ms |
| Sparse | 5,000 | 15,000 | 310 | 16.88 ms | 56.16 ms | 67.31 ms |
| Sparse | 10,000 | 30,000 | 635 | 72.42 ms | 133.46 ms | 141.50 ms |
| Hub | 1,000 | 3,000 | 393 | 11.82 ms | 14.53 ms | 88.26 ms |
| Hub | 5,000 | 15,000 | 2,087 | 71.31 ms | 79.70 ms | 89.53 ms |
| Hub | 10,000 | 30,000 | 4,259 | 228.07 ms | 162.18 ms | 170.15 ms |
| Disconnected | 1,000 | 2,000 | 237 | 2.47 ms | 12.90 ms | 56.11 ms |
| Disconnected | 5,000 | 10,000 | 1,192 | 8.00 ms | 68.04 ms | 75.52 ms |
| Disconnected | 10,000 | 20,000 | 2,372 | 32.93 ms | 142.95 ms | 154.78 ms |
| Dense | 1,000 | 10,000 | 444 | 1.66 ms | 18.09 ms | 26.24 ms |
| Dense | 5,000 | 50,000 | 2,230 | 19.45 ms | 91.25 ms | 95.33 ms |
| Dense | 10,000 | 100,000 | 4,462 | 19.95 ms | 178.34 ms | 184.67 ms |

The revision prevents the single-community collapse found in native review by removing member-to-centroid attraction, strengthening node repulsion, and adding community-level translation repulsion. A deterministic 30-node source regression requires spread in both dimensions. The old broken arrangement is preserved in [overlap-before-fix.png](artifacts/G3-community-force-overlap-before-fix.png), SHA-256 `24d7843b7b83c19f614ba70806995c2b94f9097a39f35ec17212dfb9a22ea59e`. A native capture of the corrected build before arrangement is [native-before-retest.png](artifacts/G3-community-force-native-before-retest.png); it does not demonstrate corrected arrange/restore behavior.

## Acceptance and limits

- Corrected algorithm source regression, full Rust suite, offline compile, release build, and synthetic CPU timings: **PASS**.
- Native input and corrected arrange/restore visual acceptance: **BLOCKED**. The app window remained visible to `xwininfo`, but `xdotool getwindowfocus` failed with “Failed creating new xdo instance”; `import -window` failed to open X server `:0`; Spectacle active-window capture aborted. Do not treat the earlier broken-layout capture as acceptance of the fix.
- The earlier native review did exercise the old layout and exposed a severe overlap defect. The repair has not yet been exercised through the UI.
- Direct comparison against the G0 layout baseline: **NOT RUN**. Timings are machine-specific and measure coordinate calculation plus one detection sample only; they do not measure rendering or input-to-display latency.
- Restore is transient within the open case session and is not preserved across app restart. No independent code review has been performed on this follow-up.
- Spatial arrangement is exploratory display geometry and does not encode social distance, causality, or importance.
