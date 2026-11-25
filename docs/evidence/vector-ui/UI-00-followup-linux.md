# UI-00 follow-up evidence

- **Task / owner:** UI-00 evidence follow-up and CI repair; Codex implementation and local verification.
- **Starting source commit:** `086e6268786a6519026a5510dc29b024e2bd0ed8`.
- **Verified CI candidate:** `4e1e9758adffc9c709ecb780d5b76c60e115e01b` on `codex/ui-00-chart-baseline`.
- **CI run:** [GitHub Actions run 45702380830](https://github.com/pskeffington/arch_notebook/actions/runs/45702380830), manually dispatched against the feature branch.
- **Changed paths for the follow-up:** `.github/workflows/build.yml`, `docs/evidence/vector-ui/UI-00-followup-linux.md`.

## Findings and repairs

The first manually dispatched run (45701956331, candidate `086e626`) exposed two CI defects. The Arch packaging container did not install the PKGBUILD runtime dependency `shared-mime-info`. The dense-fixture cap expression indexed a Python file handle before calling `json.load`, raising `TypeError` after the Rust unit tests passed. The workflow setup now installs `shared-mime-info`, and the assertion loads JSON before indexing its `links` array.

An intermediate run on `fa4ee17` was superseded while the workflow was corrected. The final run below was executed on the exact latest candidate after both fixes.

## Acceptance results

| Gate | Result | Evidence |
| --- | --- | --- |
| Linux tests and chart fixture determinism | PASS | Run 45702380830, `Linux tests and chart fixture determinism`; formatting, 11 Rust tests, CLI build, SQL automation acceptance, correctness fixture repeat/hash, and 100,000-link dense cap all passed. |
| Linux x86_64 | PASS | Run 45702380830, `Linux x86_64`. |
| Linux ARM64 | PASS | Run 45702380830, `Linux ARM64`. |
| macOS Apple Silicon | PASS | Run 45702380830, `macOS Apple Silicon`. |
| macOS Intel | PASS | Run 45702380830, `macOS Intel`. |
| Arch Linux package | PASS | Run 45702380830, `Arch Linux package`; package build passed after adding its declared runtime dependency to CI setup. |
| Remote workflow | PASS | Run conclusion `success`, exact head SHA `4e1e9758adffc9c709ecb780d5b76c60e115e01b`. |
| Native Linux UI baseline | BLOCKED | At this follow-up, `xrandr --query` and `xwininfo -root -tree` returned `Can't open display :0` / `unable to open display`; `wayland-info` returned `failed to create display: Operation not permitted`. A subsequent release-binary recheck launched the app on Wayland and collected two chart-preparation and two navigator CPU samples, but no interaction trace or test-window screenshot; see the [Wayland runtime evidence](UI-00-linux-wayland-a84a938.md). |
| Local locked release build | BLOCKED | `CARGO_TARGET_DIR=/tmp/arch-notebook-release-target cargo build --release --locked --offline` emitted `Compiling arch-notebook` but the command session returned no exit code; subsequent process inspection found no cargo/rustc process and no release binary. No success is claimed. |

## Handoff

UI-00 has a passing CI matrix and SQL automation acceptance on the candidate. The UI-01 precondition from PR #3—at least one complete native UI-00 baseline—remains unmet. The later Wayland smoke run proves that the release binary starts and records limited in-app CPU spans, but the documented interaction sequence and visual review remain pending; see the [Wayland runtime evidence](UI-00-linux-wayland-a84a938.md). Collect those results before beginning UI-01. IBM Analyst's Notebook XSD/application interoperability also remains pending; these runs validate Arch Notebook, not IBM compatibility.
