# Clinton corpus Qwen structural review

- **Task / owner:** Bounded local Qwen review of aggregate extraction QA metrics / orchestrator.
- **Repository:** `pskeffington/arch_notebook`, feature branch `codex/ui-00-chart-baseline`.
- **Baseline:** local source `c310ec3886449c20b5736c6bab2f11c65c54449f`; repository worktree contained unrelated existing commits and was not modified by this review.
- **Data source:** `/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-email-analysis.sqlite`, opened read-only. SHA-256: `fe9dcdf26f3810a0df3905ddf217cfd45704fd412f1509e1e379d2bb1483e071`.
- **Case file:** `/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json`, SHA-256: `6143c16d3f66964991f80184d6e2a4d165553bf9443ac3cb6bfbeea930fa255b`.
- **Model:** local Qwen 3.5 9B GGUF through `llama-completion` (`llama.cpp` b12618); weights were already present on this host. No remote inference was used.
- **Input privacy:** the prompt contained aggregate metrics only. No names, addresses, entity IDs, subjects, email text, or relationship rows were supplied. Prompt and raw model output are private `/tmp` artifacts and are not committed. Prompt SHA-256: `3c2684dc7df24e68a68b286cb3118542d56dbf13602e73bb19fd2aa62839ee46`; output SHA-256: `c0ad4b3e5b673fe63ae36d00a070ea721eb32b5c5e3e95f8abf34b4437606733`.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Read-only aggregate extraction | PASS | 4,767 messages; 6,173 relationship rows; 3,331 distinct directed pairs; 2,813 distinct endpoint IDs; 7,121 page candidates; 133 blank message dates; 163 blank relationship dates. |
| Qwen execution | PASS, bounded | Local inference exited 0; response was 606 bytes and within the 180-token cap. |
| Numeric claim review | PASS, with one unsupported cause | Qwen repeated provided totals/date counts correctly. It attributed review flags to “low confidence or unreviewed status”; only `requires_human_review=1` was provided, so the reason for the flag is unsupported. Do not reuse that causal claim. |
| Suggestion review | PASS, limited | Date-gap verification and checking whether represented pages cover the full PDF are valid QA prompts, not findings. The actual PDF completeness remains unresolved. |
| Semantic/content review | NOT ESTABLISHED | Aggregate-only prompting cannot assess message meaning, extraction accuracy, allegations, intent, or correctness of the underlying records. |

Verified source counts: all 4,767 message records and 6,173 relationship rows have `requires_human_review=1`; all 7,121 page candidates have the same flag. Every message has `header_parse_status='header-candidate-unreviewed'`. There are 3,051 distinct pages represented by at least one message, which does not establish how many pages exist in the source PDF.

An earlier full prompt produced a truncated response and included unsupported assertions about pair totals, self-loops, and a confidence threshold. That attempt is excluded from usable conclusions. This bounded run does not repair or supersede the earlier failure for semantic analysis.

**Commands:** read-only SQLite aggregate queries and local `llama-completion` invocation; exact model command is recorded in the session transcript. Input and output remained under `/tmp/clinton-qwen-review` with mode 0600 inside a mode 0700 directory. No corpus files were changed.

**Next action:** route the 11 officially sourced public-record PDFs through the authorized TRANSCRIBE/Eagle-Eye ingestion workflow, preserve a local source manifest and hashes, then validate extracted records with a human reviewer. Do not infer wrongdoing from this structural review.
