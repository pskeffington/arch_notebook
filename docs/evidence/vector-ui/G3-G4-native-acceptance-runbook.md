# G3/G4 native graph acceptance runbook

- **Task:** SNG-DELIVERY-002, remaining native G3/G4 acceptance
- **Prepared against:** `975ebe1a1344cde7cd70ed9ac309d578c1ec344e` on `codex/ui-00-chart-baseline`
- **Owner:** Native QA operator with an input-capable Linux desktop
- **Scope:** Exercise community-aware Arrange/Restore and inspect the optimized chart on a fictional fixture. This checklist does not claim native performance results.

## Prepare an isolated candidate

Run from the repository root. The generated case is fictional and deterministic; keep all edits under `/tmp`.

```sh
python3 scripts/generate-chart-fixtures.py \
  --family correctness \
  --output /tmp/arch-notebook-g3-g4-baseline.case.json
cp /tmp/arch-notebook-g3-g4-baseline.case.json \
  /tmp/arch-notebook-g3-g4-run.case.json
CARGO_TARGET_DIR=/tmp/arch-notebook-target \
  cargo build --release --locked --offline
/tmp/arch-notebook-target/release/arch-notebook \
  /tmp/arch-notebook-g3-g4-run.case.json
```

Record the source commit, generated fixture SHA-256, release-binary SHA-256, display size, and desktop/compositor. With the generator at this runbook's prepared baseline, the generated fixture SHA-256 is `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`. The correctness fixture has eight fictional entities, including the pinned `Device 17`, an isolated entity, parallel/reversed records, a self-loop, a long relationship label, and an offscreen relationship that crosses the viewport at 2× zoom.

## G3 — Arrange and Restore

1. Switch to the **Chart** workspace. Choose **View… → Fit all entities**, capture the full chart, and record the fixture title. This only changes the view. Do not change entity positions before the first Arrange action.
2. Open **View… → Chart layout**, choose **Community-aware force**, leave **Force iterations** at 50, and click **Arrange chart**.
3. Wait for the status to change from `COMMUNITY-AWARE LAYOUT RUNNING · CHART REMAINS RESPONSIVE` to `LAYOUT ARRANGED BY COMMUNITY-AWARE FORCE · 1 POSITIONS PINNED`.
4. Choose **View… → Fit all entities** and capture the arranged chart. Confirm all eight entities remain present, the pinned entity stays fixed, and the result is not collapsed into an unreadable pile. Record any significant node or label overlap as a failure; do not judge social distance from positions.
5. Open **View…** and click **Restore positions from before force layout**. Confirm the status `PRE-FORCE LAYOUT POSITIONS RESTORED` and capture the restored chart.
6. Click the **Save case** button in the top application bar to save the temporary run case. Compare entity coordinates and pin flags with the untouched baseline:

```sh
python3 - /tmp/arch-notebook-g3-g4-baseline.case.json \
  /tmp/arch-notebook-g3-g4-run.case.json <<'PY'
import json
import sys

def positions(path):
    with open(path, encoding="utf-8") as stream:
        case = json.load(stream)
    return {
        entity["id"]: (entity["x"], entity["y"], entity["position_locked"])
        for entity in case["entities"]
    }

before, after = map(positions, sys.argv[1:])
if before != after:
    raise SystemExit("FAIL: restored coordinates or pin flags differ from baseline")
print(f"PASS: exact positions and pin flags restored for {len(before)} entities")
PY
```

**G3 passes** only with the observed running/success/restore status, the pinned entity unchanged, a legible arranged chart, and exact saved-coordinate restoration. Attach before/arranged/restored screenshots and the comparison output to the acceptance record. If the background operation fails, is discarded, or Restore is unavailable, record FAIL with the status text.

## G4 — Route, label, zoom, and pan review

Use a fresh launch of the untouched baseline case and the same candidate binary. Switch to the **Chart** workspace. The correctness fixture starts at 2× zoom and zero pan; retain that view for the initial crossing-route check.

1. Capture the 2× zoom, zero-pan screenshot. Confirm the long relationship from offscreen endpoints crosses the viewport and its label remains rendered. Then pan the chart and confirm visible routes and labels update without leaving gaps.
2. Inspect the parallel/reversed relationship records, self-loop, direction markers, and long labels. Confirm individual relationship picking remains possible and that culling does not remove a route merely because its endpoints are outside the viewport.
3. Pan and zoom back to a neutral fit, capture screenshots, and record any missing route, label, marker, loop, or selectable record as FAIL with the view and steps that reproduce it.

### Dense overview labels

Run a second fictional chart to check the Auto/All behavior that was added after the `VECTOR UI SPARSE 30` screen review:

```sh
python3 scripts/generate-chart-fixtures.py \
  --family sparse --nodes 30 \
  --output /tmp/arch-notebook-g3-g4-sparse-30.case.json
/tmp/arch-notebook-target/release/arch-notebook \
  /tmp/arch-notebook-g3-g4-sparse-30.case.json
```

At the neutral overview with more than 36 visible routes, Auto should hide labels and attribute callouts for unselected/unfocused links. Label mode does not change route rendering or picking; fully offscreen routes remain culled. Click a visible route: the exact relationship record should become selected and its label should appear. Open **View… → Show all relationship labels**; all visible route labels and attribute callouts should appear. Uncheck it or restart the app to restore the default Auto mode. Pan a large chart until 36 or fewer routes are visible; Auto should show labels for that visible subset again. Record screenshots and the selected record's label, endpoints, connection ID, and source reference from the Connections workspace.

The current matched culling A/B establishes CPU frame-build and CPU tessellation reductions on synthetic panned views only. This runbook's screenshots and pointer actions establish visual/input correctness, not GPU time, compositor work, frame rate, presentation latency, or native performance improvement. Leave those measurements **NOT MEASURED** unless a separate instrumented native comparison is provided.

## Evidence handoff

Record the operator, date, source commit, fixture and binary hashes, display/compositor, exact actions, visible status strings, coordinate-comparison output, screenshot paths/hashes, and each gate result as PASS or FAIL. Use **BLOCKED** only when the input/display path prevents the steps from being performed, and retain the specific error. Attach evidence to the next dated G3/G4 native acceptance report; this runbook itself is not acceptance evidence.
