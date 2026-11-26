# UI-ANALYSIS-CARD-01: Active-chart method card

- **Result:** PASS for deterministic counts, privacy-scoped copy text, offline Rust checks/builds, and local install synchronization. **BLOCKED** for native card interaction, screenshot review, and independently documented review.
- **Task / owner:** SNG-DELIVERY-001 Analysis Method Card; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `008537813d60dd76db0732b06f87b6266a02603a`.
- **Product source commit:** `a5208a0e3aa8c0fe3b3a1e3f02986e45e97b919a` (`feat: add analysis method card`).
- **Changed paths:** `src/main.rs`, `docs/notebook-workflows.md`, `docs/analyst-notebook-parity.md`.
- **Build target directory:** `/tmp/arch-notebook-target`.
- **Debug executable SHA-256:** `dfab5049a21a08063a3bf244bcd16d164d51aed516d01d7f18e3034d1697c38d`.
- **Release executable SHA-256:** `765f8fa2e6384a2df4747890a9b954920a1668dc35557068f749f89afcea1fc7`.

The card is opt-in, transient, and read-only. It reports entity/isolate counts; directed and undirected relationship-record counts; unordered endpoint pairs, repeated records, and records with missing endpoints; plus timeline-recognized, blank, and unrecognized date counts. The method text describes the current components, articulation points, geodesic layers, and shortest-path controls. Structural changes and relationship date/direction edits clear the cached summary. Copy text contains aggregate counts and fixed method descriptions only.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0 | Rust formatting is clean. |
| `git diff --check` | PASS, exit 0 | No whitespace errors. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` | PASS, exit 0 | 25 passed, 0 failed, 0 ignored. `analysis_method_card_counts_records_pairs_isolates_and_date_quality` covers isolates, direction, self/parallel pairs, missing endpoints, date categories, redaction, and unchanged case serialization. `network_analysis_invalidation_discards_a_cached_method_card` covers cache invalidation. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | PASS, exit 0 | Offline integration check completed. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` | PASS, exit 0 | Debug build completed. |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | PASS, exit 0 | Optimized release build completed. |
| Local user install | PASS, exit 0 | `scripts/install-linux-user.sh` installed the release build to `/home/sysadmin/.local/bin/arch-notebook`; its SHA-256 matches the release artifact. The menu entry and desktop shortcut compare byte-for-byte and both pass `desktop-file-validate`. |
| Launcher/runtime metadata | PASS | `--version --json` returned version `0.1.0`; MIME query returns `arch-notebook.desktop`. The installed automation installer and SVG/PNG icons match repository hashes. |
| Native app window | PASS, launch-only | Launching the installed executable with the fictional correctness fixture stayed live and the X window tree showed `Arch Notebook · Link Analysis`. No image capture of the new build was obtained. |
| Method-card click, refresh, and clipboard UI | BLOCKED | Input injection is unavailable. `spectacle` aborted on both capture attempts; ImageMagick could not open the X server. No visual review of the new card is claimed. The earlier [UI-00 render screenshot](UI-00-native-render-20261126.md) predates this feature and is not evidence for the card. |
| IBM application/XSD comparison | BLOCKED / not run | No IBM Analyst's Notebook application or vendor schema was used; no parity or external interoperability claim is made. |
| Independent code/security review | NOT VERIFIED in this run | The roadmap contains earlier reviewer-pass statements, but no separate handoff/report artifact is checked in. This implementation session performed only code-path self-review; it is not independent approval. |

The unit fixture is synthetic. No operational case or SQLite source was opened or changed. Building/copying the summary does not modify the in-memory case in the regression test; the UI clipboard path was not interactively exercised.

## Handoff

G1 implementation and G2 build/documentation gates pass. The product change is committed on a non-main branch and the local desktop launcher now points to the matching release build. G3 remains blocked on input-driven card review and independently verifiable review evidence. Continue UI-00 native interaction acceptance when a usable input path is available; separately validate IBM formats in the IBM application before making compatibility claims.
