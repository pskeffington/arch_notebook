# UI-01 coincident-position route and picking evidence

- **Task / owner:** Separate relationship geometry and hit targets when distinct entities share chart positions; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline / candidate:** `32124542094c8ba08010b5fd67297e6d2385236b` → `fae574dc74af78ae19a950641ba2b14e652901c2`.
- **Changed path:** `src/main.rs`.
- **Date / host:** 2026-11-26, Linux x86_64, kernel `7.3.5-zen1-1-zen`; `rustc 1.100.0`, `cargo 1.100.0`.
- **Fixture:** `fixtures/chart/correctness.case.json`, SHA-256 `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`.
- **Lockfile:** `Cargo.lock`, SHA-256 `942e3102cd836863c5565bb70793038a187eda8a504dc2444a1b70be6696031c`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `63d008c9d5431390663da1f58f803534908e3e5d4904d67054895b3391093088`.

## Source checks

| Check | Result | Evidence |
| --- | --- | --- |
| Coincident-position route and selection regression | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`, exit 0; 28 passed, 0 failed. The fixture includes parallel and reversed relationships, self-links, and separate entity pairs at identical positions; all nine sampled routes are distinct, and a route sample selects its exact relationship. |
| Formatting | **PASS** | `cargo fmt --all -- --check`, exit 0. |
| Compile check | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, exit 0. |
| Optimized release build | **PASS** | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline`, exit 0. Binary digest above. |
| Independent code review | **PASS** | [Review handoff](UI-01-coincident-routes-review-linux.md) records review of route allocation, render/hit-test sharing, and regression coverage. Security review was not performed. |
| Whitespace check | **PASS** | `git diff --check`, exit 0. |
| CI for candidate | **PENDING** | After push, `GET /repos/pskeffington/arch_notebook/actions/runs?head_sha=fae574dc74af78ae19a950641ba2b14e652901c2` returned `total_count: 0` on 2026-11-26. Local tests do not substitute for a candidate workflow run. |

The renderer now derives route offsets from endpoint positions for links that otherwise share the same geometric path. Direction is normalized for distinct coincident endpoints; self-links at a shared position receive separate loop routes. The chart's nearest-link picking uses the same computed offsets as drawing. This preserves separate relationship records and their identities.

## Native observation and limits

| Check | Result | Evidence |
| --- | --- | --- |
| Current release process smoke | **PASS, bounded only** | `timeout 10s env DISPLAY= WAYLAND_DISPLAY=wayland-1 XDG_RUNTIME_DIR=/run/user/1000 XDG_DATA_HOME=/tmp/arch-notebook-ui01-coincident-xdg /tmp/arch-notebook-target/release/arch-notebook --open-case fixtures/chart/correctness.case.json` exited 124 after the 10-second timeout with no application error output. This establishes only that the process remained alive for the interval. |
| Window capture and visual review | **BLOCKED** | `xrandr --query` could not open display `:0`; no screenshot was obtained for this candidate. The bounded process smoke is not visual-render evidence. |
| Pointer/keyboard interaction and UI-00 native baseline | **BLOCKED** | `xdotool search --onlyvisible --name 'Arch Notebook'` failed with `Failed creating new xdo instance`; no input was delivered. UI-00 task flow, repeated traces, picking latency and performance samples remain open. |
| IBM visual or schema/application interoperability | **NOT TESTED** | No IBM application or schema was used. |

This is source-level route and picking evidence plus an optimized build. It does not close UI-00 native acceptance, prove visible output on this candidate, or establish IBM parity. Continue with an input-capable Linux desktop for the UI-00 task sequence; validate IBM interoperability separately.
