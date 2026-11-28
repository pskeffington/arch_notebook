# Local Qwen/Ollama processing of public records — 2026-11-28

## Result

**Qwen document processing: PASS, bounded. Fresh Ollama rerun: BLOCKED.** The existing batch contains completed Qwen 3.5 9B outputs for ten public NARA case-letter documents. Its ten case IDs and source PDF SHA-256 values match the ten entries in the private staged-source manifest exactly. This is the ten-letter public-record QA set, not the 7,000-email Clinton corpus.

| Gate | Result | Evidence |
| --- | --- | --- |
| Source-to-result identity | PASS | Ten validation records match all ten staged NARA case IDs and PDF hashes. |
| Model outputs completed | PASS, bounded | 10/10 records have `done=true`; model tag is `qwen3.5:9b`. |
| JSON and source quote checks | PARTIAL PASS | 9/10 outputs parse as JSON; 5/10 candidates pass exact or normalized quote-string matching; 4 valid outputs have missing source quotes; 1 output is truncated and invalid JSON. |
| Candidate review state | PASS | All outputs remain `review_required=true`. Only the five quote-traceable candidate outputs are included in the presentation candidate file. |
| Arch Notebook relationship import | PASS, bounded | The associated notebook handoff contains ten isolated document entities and zero relationships; model-suggested links were not imported. |
| Semantic or legal review | NOT ESTABLISHED | Quote matching verifies string traceability only. It does not establish entailment, accuracy, attribution, or legal conclusions. Human review remains required. |
| Ollama invocation provenance | LIMITED | The completed artifacts record the Qwen model tag but not the original endpoint request, Ollama server version, or invocation ID. The prior QA narrative records a later Ollama rerun as unavailable. Treat the completed Qwen batch as established; exact first-run Ollama transport is not independently reproducible from the committed artifacts. |
| Fresh local Ollama rerun | BLOCKED | `ollama list` cannot connect to `127.0.0.1:11434`; starting `ollama serve` exits 1 with `listen tcp 127.0.0.1:11434: socket: operation not permitted`. No new model request was made in this turn. |

Per-document quote-string gate (matched / missing strings; these are not semantic scores):

| NARA case ID | Response | Quote strings | Gate |
| --- | --- | ---: | --- |
| UD-2020-0001 | Valid JSON | 5 / 6 | FAIL · one source quote missing |
| UD-2022-0055 | Valid JSON | 4 / 6 | FAIL · two source quotes missing |
| UD-2023-0052 | Valid JSON | 6 / 6 | PASS · candidate only |
| UD-2024-0056 | Valid JSON | 6 / 6 | PASS · candidate only |
| UD-2024-0075 | Valid JSON | 6 / 6 | PASS · candidate only |
| UD-2024-0079 | Invalid JSON | 0 / 0 | FAIL · truncated response |
| UD-2025-0019 | Valid JSON | 6 / 6 | PASS · candidate only |
| UD-2025-0039 | Valid JSON | 5 / 6 | FAIL · one source quote missing |
| UD-2026-0001 | Valid JSON | 6 / 6 | PASS · candidate only |
| UD-2026-0007 | Valid JSON | 5 / 6 | FAIL · one source quote missing |

## Repository evidence

- [Political-records QA report](political-records-qa-20261128.md): source and pipeline context, Arch Notebook handoff, and limitations.
- [Per-document Qwen validation](political-records-qa/artifacts/qwen-validation.json), SHA-256 `31c39a92c96edcc9737fb3743385b7b575110ffb4db386d520b3c1c3754a62f7`.
- [Five quote-traceable review-required candidate records](political-records-qa/artifacts/qwen-reviewed-candidates.jsonl), SHA-256 `f67bc3c4aaa06c205f8fe3f5e0db01bb3ac8cbea41bc66501d4ff22cdd6bec9f`.
- [Redacted Arch Notebook finding](political-records-qa/artifacts/notebook-finding.md) and [network rendering](political-records-qa/artifacts/notebook-network.svg).
- [Private-source acquisition and hashes](political-email-messaging-case-acquisition-20261128.md); full PDFs and extracted text are not included in Git.

The validation and candidate artifact digests were checked locally before recording. The exact original inference command and Ollama server log were not retained, so this note does not claim those missing details or claim the later rerun succeeded.
