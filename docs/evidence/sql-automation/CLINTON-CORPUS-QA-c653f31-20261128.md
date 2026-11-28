# Clinton corpus acceptance — source `c653f31`

This is a read-only acceptance rerun against the optimized binary built from the current synchronized feature branch. The primary Clinton case remains the main SQL regression batch.

| Gate | Result | Evidence |
| --- | --- | --- |
| Source commit | PASS | `c653f3141b224a10dee721542965cc1a42b30866` |
| Optimized release build | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline`: exit 0, 2m11s. |
| Release binary identity | PASS | SHA-256 `46596680af31ba1d92314426e6fd69feb7179a9d8489038cbcca1ee48ac42c1a`. |
| Query-only SQL preflight | PASS | 1 saved job passed, 0 failed, exit 0. |
| Saved-run replay | PASS | 6,173 relationships, 0 skipped rows, 0 added, 0 removed, 0 changed, exit 0. |
| Saved case and snapshots | PASS | 2,813 entities, 6,173 relationship records, 2 snapshots, 2 complete batches, 2 passing runs. |
| Default-redacted topology packet | PASS | All 2,813 entities rendered; 643 communities, 945 cross-community contact rows, 291 proxy candidates; 704 SVG rows, maximum same-y row 4, 0 clipped labels. No names, relationship labels, IDs, or evidence references included. Packet files were mode 0600 in a mode 0700 directory. |
| Input immutability | PASS | Harness confirmed source PDF, case, and SQLite hashes were unchanged. |
| Extraction, Qwen, human content review | NOT ESTABLISHED | These are outside this SQL/render acceptance harness. |

Aggregate-only artifact: [`CLINTON-CORPUS-QA-c653f31.json`](artifacts/CLINTON-CORPUS-QA-c653f31.json), SHA-256 `c3df6309b31c2c81577970ec53497fdcc3cff5807350d0c863f1cefbea55c15c`.

Command:

```sh
python3 scripts/test-clinton-corpus-acceptance.py \
  --binary /tmp/arch-notebook-target/release/arch-notebook \
  --pdf /home/sysadmin/Research.Databases/TRANSCRIBE/hillary-clinton-emails-august-31-release.pdf \
  --case /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json \
  --database /home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite \
  --report /tmp/CLINTON-CORPUS-QA-c653f31.json
```
