# UI-00 installed-release chart CPU benchmark — 2026-11-28

## Handoff

- Task: collect repeatable CPU measurements for chart vector preparation and pointer picking on the installed Arch Notebook release.
- Repository: `/tmp/arch-notebook-remote-candidate`.
- Source commit: `e41fe0e059266865a4ac661f8ddba649a07126ab` (product code matches `c653f31`; see [synchronized-build evidence](../deployment/SYNCED-HEAD-01-current-branch-linux-20261128.md)).
- Branch: `codex/ui-00-chart-nested-conditions`.
- Installed binary SHA-256: `46596680af31ba1d92314426e6fd69feb7179a9d8489038cbcca1ee48ac42c1a`.
- Benchmark driver SHA-256: `6c08d79853641e90d72faeced931eab526014b02671eaf780d9c19d23fc352d3`.
- Fixture generator SHA-256: `72de9dc66ec4cad5d3a6f5927da9c8ae130db85b43038ae0a4cf5467e7a26af6`.
- Host: Linux x86_64, `Linux-7.3.5-zen1-1-zen-x86_64-with-glibc2.44`.

## Command and result

```text
python3 scripts/benchmark-chart-vector-geometry.py --binary /home/sysadmin/.local/bin/arch-notebook --output /tmp/ui00-installed-cpu-benchmark.json --iterations 7 --sizes 100 1000 10000
exit: 0
result: 12 deterministic fictional cases measured
```

Each case uses three warmups and seven measured samples. The table reports medians from the application CLI benchmark, not painted frames. The exact fixture SHA-256 values and all sample distributions are retained in [the raw benchmark JSON](artifacts/UI-00-installed-cpu-benchmark-20261128.json), SHA-256 `e5dff5ef9ba1700f191b07dfa42f4f656f398cd9d32a15af0b5991f6a8670cd4`.

| Fixture | Entities | Relationship records | Geometry preparation median | Pointer candidate scan median | Combined preparation + pick median |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sparse | 100 | 300 | 0.45 ms | 0.26 ms | 0.73 ms |
| Sparse | 1,000 | 3,000 | 1.38 ms | 0.76 ms | 2.15 ms |
| Sparse | 10,000 | 30,000 | 21.25 ms | 8.76 ms | 30.23 ms |
| Hub | 100 | 300 | 0.12 ms | 0.07 ms | 0.19 ms |
| Hub | 1,000 | 3,000 | 7.32 ms | 3.74 ms | 11.56 ms |
| Hub | 10,000 | 30,000 | 22.61 ms | 8.81 ms | 32.26 ms |
| Disconnected | 100 | 200 | 0.16 ms | 0.10 ms | 0.28 ms |
| Disconnected | 1,000 | 2,000 | 1.29 ms | 0.57 ms | 1.88 ms |
| Disconnected | 10,000 | 20,000 | 14.18 ms | 6.26 ms | 20.95 ms |
| Dense | 100 | 1,000 | 0.36 ms | 0.24 ms | 0.60 ms |
| Dense | 1,000 | 10,000 | 10.40 ms | 5.04 ms | 15.62 ms |
| Dense | 10,000 | 100,000 | 72.29 ms | 30.28 ms | 103.06 ms |

## Acceptance

| Gate | Result | Evidence and limits |
| --- | --- | --- |
| Installed binary identity | PASS | Binary digest matches the current-user installed release recorded in `SYNCED-HEAD-01`. |
| Workload determinism and raw sample retention | PASS | Twelve generated fixture hashes, seven timing samples per metric, medians, p95s, and counts are in the raw JSON artifact. |
| Application CPU geometry and pointer-pick measurements | PASS, bounded | Measures `chart_vector_geometry` preparation and the same sorted pointer-cycle candidate scan used by the UI. No paint is invoked. |
| Native UI, rendered frames, navigator cost, and interaction response | BLOCKED / NOT MEASURED | `xrandr --query` could not open `:0`; `wayland-info` failed with `Operation not permitted`. The combined UI probe returned 1. The CLI benchmark does not measure painting, text shaping, GPU/compositor work, navigator submission, FPS, or input-to-paint latency. |
| Universal performance target or before/after optimization claim | NOT CLAIMED | Values are host-specific. The benchmark's “previous pointer path proxy” repeats duplicate geometry work in-process; it is not a historical binary baseline and is excluded from any optimization claim. |

## Next action

Repeat the documented native UI-00 sequence on an input-capable Linux display, capture stable application diagnostics and screenshots, and inspect the apparent responsiveness issue from prior native evidence. Keep UI-00 open until that evidence exists; this CPU report does not authorize advancing its native gate or assert IBM visual parity.
