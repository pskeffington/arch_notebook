# TMP-ARCHIVE-01 — move inactive Arch Notebook artifacts to the Research share

- **Task / owner:** Clear excess Arch Notebook build/test files from `/tmp` and archive them on the mounted Research share; implementation session.
- **Repository:** `/home/sysadmin/github/arch_notebook`.
- **Destination:** `smb://archive.local/Research/EVIDENCE/arch_notebook/tmp-archive-2026-11-27/` (mounted at `/home/sysadmin/Research.Databases/EVIDENCE/arch_notebook/tmp-archive-2026-11-27/`).
- **Active build target:** `/tmp/arch-notebook-target` was retained as required by the repository instructions.

## Archived entries

- `arch-notebook-current-target`
- `arch-notebook-release-verify`
- `arch-notebook-packet-community-release`
- `arch-notebook-vector-bench-4mfsu3bo`
- `arch-notebook-ui00-fixtures-20261126-01`
- `arch-notebook-sql-auto-30b8020-20261127`
- `arch-notebook-sql-scale-30b8020-20261127`
- `arch-notebook-sql-sched-30b8020-20261127`
- `arch-notebook-linux-x86_64`
- `clinton-corpus-qa-30b8020.json` and `sql-scale-30b8020.json`

## Verification

| Gate | Result | Evidence |
| --- | --- | --- |
| Copy to SMB archive | **PASS** | `rsync -rlt --info=progress2` transferred 3,195,663,365 bytes (5,713 file transfers across 8,420 filesystem entries) to the destination above. |
| Content comparison | **PASS** | `rsync -rc --dry-run --itemize-changes` completed without itemized differences. SHA-256 of the archived Clinton and scale reports matches the corresponding Git evidence artifacts. |
| Local source cleanup | **PASS** | After the content comparison, only the listed `/tmp` sources were removed. Checks confirmed those paths are absent; the active `/tmp/arch-notebook-target` remains. |
| Local capacity | **PASS** | `/tmp` went from 417 MB available (98% used) to 3.0 GB available (82% used). |
| Scope boundary | **PASS** | Only Arch Notebook artifacts listed above were moved; other projects' temp files were not touched. |

The SMB folder also contains a `README.txt` manifest with the archive contents and transfer summary. SQL scale data is fictional. The Clinton report is aggregate-only; no raw email rows or message text were archived by this QA report.
