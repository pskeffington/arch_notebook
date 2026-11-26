# Arch Notebook GUI QA review checklist

Use the bundled fictional correctness chart only. The package launches with isolated XDG data/config/cache directories and, by default, a bubblewrap network namespace. `--allow-app-network` uses host networking and does not provide network isolation. The run report records package checks separately from native UI review.

## Native review

- [ ] Window opens and remains responsive for the review session.
- [ ] Correctness chart shows all 8 entities and 8 relationship records.
- [ ] Self-loop and parallel/reversed links are visible as separate records.
- [ ] Selecting each coincident-position relationship resolves to the intended record.
- [ ] Analysis Method Card updates when switching charts or changing relevant graph data.
- [ ] Copy action copies only the documented summary/method metadata.
- [ ] Quick open finds **Fit chart**; Ctrl/Command+0 frames all eight entities.
- [ ] Resize, zoom, pan, and Fit all remain usable; no critical controls are obscured.
- [ ] No crash, unresponsive window, or unexpected data write occurred.

Record each result in `manual-review.json` (copy `manual-review.template.json`) using only `PASS`, `FAIL`, `BLOCKED`, or `PENDING`; include no real case content or free-form notes. Attach screenshots only after checking that the image contains no personal or sensitive data.

## Interpretation

Package hash, binary digest, fixture counts, and startup status are automated evidence. They do not prove UI usability. Qwen feedback is advisory; it does not change a `BLOCKED` or `PENDING_HUMAN_REVIEW` status to `PASS`.
