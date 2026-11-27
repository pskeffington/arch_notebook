# Origin-centered group contact map acceptance

- **Task / owner:** Add a visual, bounded group-level overview to origin-centered contact analysis; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `01a651890bd5d38cfe03ac017edb4a3e610b37b5`.
- **Product source commit:** `26fe459dee702ab6c41da483f7f29d267c60fed7`.
- **Changed source path:** `src/main.rs` (map rendering, deterministic bounded sampler and regression test).
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `b6955b38b8af4596e0d88a9667d7d60637ffdd7536c63a806d2d8334e648172f`.

## Behavior

The Analysis pane now draws origin-centered contact groups as nodes in hop-layer columns and cross-group contacts as lines. Line width indicates relationship-record count; direction counts and exact records remain in the complete interaction list. Nodes show group and internal record counts. The origin has a distinct fill, and a gold outline marks groups containing a ranked cross-group proxy. Clicking a node focuses its original member entities on the chart; it does not create or persist synthetic entities.

To bound painting for large cases, the overview shows at most 120 groups across at most 40 sampled hop layers and reports when it omits groups or layers. The group and interaction lists retain the full analysis and exact record references. This is a navigation overview in the Analysis pane; the primary chart still renders the original entities and relationship records.

## Acceptance results

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Locked offline compile | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0 with no warnings after the final source adjustment. |
| Rust unit suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` exited 0; 51 passed, 0 failed. Proximity analysis tests cover hop assignment, same-layer contact groups, exact parallel records, direction counts, distinct endpoint pairs, proxy ranking and deterministic map sampling. |
| Optimized release build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0 in 2m42s. Artifact hash recorded above. |
| Current-user binary, launcher and icons | PASS | Installed release binary matches the build hash; desktop shortcut and menu entry compare byte-for-byte and pass `desktop-file-validate`; installed SVG and PNG hashes match repository assets. Installed binary reports `Arch Notebook 0.1.0`. |
| GitHub workflow matrix | PENDING | Final manual run [45914257016](https://github.com/pskeffington/arch_notebook/actions/runs/45914257016) targets exact source commit `26fe459dee702ab6c41da483f7f29d267c60fed7`; it was queued. The immediately preceding candidate's matrix passed run [45912700353](https://github.com/pskeffington/arch_notebook/actions/runs/45912700353), but that run does not cover the final sampler regression test. |
| Native visual and pointer acceptance | BLOCKED | `wayland-info` returned `Operation not permitted`; no screenshot or pointer interaction is claimed for this candidate. |
| IBM application/XSD compatibility | NOT TESTED | No IBM Analyst's Notebook installation or IBM XSD validation was available. |
| Independent code review | NOT PERFORMED | This is implementation-owner acceptance evidence. |

## Commands and observed outcomes

```text
cargo fmt --all -- --check
exit 0

git diff --check
exit 0

CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline
exit 0

CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline
exit 0 (51 passed, 0 failed)

CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline
exit 0 (optimized profile finished in 2m42s)

scripts/install-linux-user.sh /tmp/arch-notebook-map-install-final/arch-notebook-linux-x86_64
exit 0

cmp /home/sysadmin/.local/share/applications/arch-notebook.desktop '/home/sysadmin/Desktop/Arch Notebook.desktop'
exit 0

desktop-file-validate /home/sysadmin/.local/share/applications/arch-notebook.desktop '/home/sysadmin/Desktop/Arch Notebook.desktop'
exit 0

wayland-info
exit 1 (failed to create display: Operation not permitted)
```

The checked-in proximity fixture tests the analysis model, not the visual map, and no native UI test is claimed. The map's runtime appearance and pointer behavior still require review from an input-capable desktop session.
