# PROXIMITY-MAP-HOVER-02 — restore topology contact-line hover preview

- **Task / owner:** Clean up the sampled graph topology interaction; implementation session with topology audit and code review.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `44a6900305d4e969e1bfb473a249feab5bc53fe6`.
- **Candidate source:** `30b8020` (`Restore topology line hover preview`), pushed to `origin/codex/ui-00-chart-baseline`.
- **Changed product path:** `src/main.rs`.

## Change

Hovering a sampled contact route again highlights the route and previews both group numbers, relationship-record count, distinct endpoint-pair count, and directed/undirected totals. The preview stays in a reserved one-line row below the scrollable topology map; truncation retains a tooltip with the full summary. Group hover takes priority over route hover, and clicking a route still selects it for exact-record inspection.

## Acceptance

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Focused headless pointer interaction | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline headless_ui_drives_geodesic_and_detected_community_actions -- --test-threads=1` exited 0. Pointer movement over a rendered route shows its summary without selecting the route; the same UI regression continues to verify route-click selection and node actions. |
| Full offline Rust suite | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 73 passed, 0 failed, 3 ignored (the opt-in benchmark and two Clinton-local tests). |
| Retained Clinton topology QA | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target ARCH_NOTEBOOK_CLINTON_CASE_PATH=/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json cargo test --locked --offline clinton_case_topology -- --ignored --nocapture --test-threads=1` exited 0: 2 passed. The production UI exercised the 2,813-entity / 6,173-record case and its 48-group, 47-pair sample (1,315 represented records); deterministic layout sampling reduced crossings from 23 to 3. Frame timing was noisy and is not presented as a performance improvement. |
| Exact-source optimized build | **PASS** | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0 for the candidate source. Release SHA-256: `330bc4fe2446bf8c7bff685ac4e56ddc5ffabc7b82cb59199b2cf537bd0d09b0`. |
| Topology code review | **PASS** | A separate topology code review found node hover priority, viewport culling, route tolerance, and the fixed 16 px preview row consistent; no files changed and no tests were run by the reviewer. |
| Current-user deployment icon and launcher sync | **PASS** | The release executable installed to `/home/sysadmin/.local/bin/arch-notebook` matches the build SHA. The application-menu file and `/home/sysadmin/Desktop/Arch Notebook.desktop` compare byte-for-byte and both pass `desktop-file-validate`. SVG and PNG icon hashes match repository assets; `--version --json` reports `0.1.0`. Isolated installer acceptance report: [LOCAL-INSTALL-08 JSON](../deployment/artifacts/LOCAL-INSTALL-08-linux.json). |
| Native visual and pointer acceptance | **BLOCKED** | The installed program was invoked with the retained case. This session could not produce a screenshot: `spectacle` aborted, and ImageMagick `import` could not open the display. The running UI's actual layout, contrast, and pointer feel therefore remain unreviewed. |
| Cross-platform CI | **PENDING** | The workflow runs on `main` pushes, pull requests, and manual dispatch, not feature-branch pushes. No PR or manual dispatch was created. |

No corpus names, message content, or row-level records are included in this evidence. The tests validate structural topology rendering and interaction, not email completeness, semantic interpretation, IBM ANX interoperability, or Qwen analysis.
