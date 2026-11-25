# Bounded document evidence handoff

Arch Notebook does not read PDF or OCR payloads. Eagle-Eye contains a `LocalOcrVisualBackend` that renders PDFs with `pypdfium2`, uses Tesseract, and emits bounded visual observations; its configured page limit defaults to 10 and it does not return full OCR text. Eagle-Eye's broader `docs/pdf_ingestion.md` describes a wider ingestion pipeline. Neither the backend nor the broader specification proves that a particular file completed a verified Eagle-Eye workflow. For an upstream-processed source, inspect its packet and release state before using it. The native JSONL importer accepts only an approved, downstream-ready handoff; candidate, held, review-required, or blocked packets are not eligible.

## Authority and limits

- Eagle-Eye owns source, page, and region evidence; `trans` owns transcription; `trans-release` owns validation and release status.
- Arch Notebook checks the approved handoff's bounded shape, explicit approval reference, workflow state, and evidence boundary. It does not perform Eagle-Eye visual review, `trans` processing, or `trans-release` validation, and it does not assign release approval or confidence.
- Import preserves the approval and workflow identifiers as attributes. An approved handoff remains source-attributed material for analyst review; import does not itself establish an investigative fact or create relationships.
- Use bounded identifiers, field values, confidence, field-origin maps, and page/region references when present. Never include a PDF, page image, OCR dump, or unrestricted transcript in the CSV or case file.
- The upstream handoff's `target_layer` remains `trans-downstream`. Importing it into a local Arch Notebook case does not change the upstream target or establish Arch Notebook as an approved `trans-release` destination. The upstream handoff shape is not yet governed by a versioned Arch Notebook consumer contract.

## Attach a local transcription candidate

For a locally processed, still-unreviewed transcript candidate, use `arch-notebook --attach-transcription-candidates <case.json> <manifest.json> [--json]`. The command accepts the bounded `arch-notebook-transcription-candidate/v1` local adapter manifest. It verifies the candidate text digest, the source PDF filename, byte count and SHA-256 against exactly one existing source entity, sequential per-page render/text digests, and the review-required flag before saving metadata. It records paths, digests, page count, OCR/normalizer provenance, review status, and a constrained Qwen analysis state on the matching source entity. The candidate text and page images remain outside the case; no analytical entities, relationships, or release state are created or inferred. This local adapter is not an upstream TRANS producer contract and does not mark the transcript corrected, verified, or approved.

## Import approved bounded handoffs

Use **Chart → Import… → Import approved handoffs** with an approved downstream-handoff JSONL file in the Chart workspace's **Import path** field, or run `arch-notebook --import-document-packets <case.json> <approved-handoffs.jsonl> [--json]`. The importer requires `validation_status=schema_valid`, `release_status=released`, `downstream_status=handoff_ready`, `target_layer=trans-downstream`, and a matching `release-approval-ref/v1` marked approved. If `review_status` is present, it must be `qa_approved`. It also requires explicit false evidence-boundary flags for source documents, OCR dumps, and page images, and accepts only the `json_export` and `sheets_reporting` destination labels. The complete file is checked before changing the case. Accepted handoffs create or update one isolated entity per `release_packet_id` in the separate **Document evidence · trans-release** chart; no relationships are inferred. This is a bounded local import of the currently documented upstream shape, not a versioned or producer-approved Arch Notebook integration.

The importer requires the source, packet, visual, return, release, and downstream-handoff IDs; validation, release, and downstream states; and the matching approval reference. `review_status` is retained when supplied but is optional in the downstream handoff contract. It retains the bounded approval identifiers, evidence boundary, destinations, selected reporting fields, and available field/evidence references. Optional metadata values are each limited to 8 KiB. It rejects positive raw-document/OCR/image flags and forbidden raw-evidence fields, and limits input to 10 MiB, 500 handoffs, 256 KiB per JSONL line, and 32 KiB per projected handoff. Duplicate release packet and downstream-handoff IDs are rejected. This is an Arch Notebook consumer boundary check; upstream Eagle-Eye, `trans`, and `trans-release` remain responsible for their own packet validation and release decisions. Candidate, rejected, blocked, held, and review-required records are rejected.

For a manual CSV workflow or a custom field mapping, use the import design below instead.

## CSV import design

For a manual CSV workflow, convert one already-approved handoff to one isolated entity row. In the import design, map `record_type` to record type, `source` to entity name, `source_type` to entity type, `source_external_ids_json` to external IDs, and each `source_attribute:<key>` column to an entity attribute. Select **Import into new chart** before importing so handoff records remain separate from the active analysis chart. Do not use this route to bypass upstream release approval.

For example, the approved handoff in `trans-downstream/fixtures/sanitized_public_notices/downstream_handoffs.jsonl` can map these fields:

| CSV column | Value from packet | Purpose |
| --- | --- | --- |
| `record_type` | `entity` | Creates an isolated record, not a relationship. |
| `source` | `Document packet · <source_id>` | Neutral chart label that does not promote the issuer as a verified entity. |
| `source_type` | `Released document packet` | Identifies the record as a bounded handoff. |
| `source_external_ids_json` | `[ ["trans-release", "<release_packet_id>"] ]` | Stable release-packet identity for reimport. |
| `source_attribute:packet_id` | `packet_id` | Workflow join key. |
| `source_attribute:visual_packet_id` | `visual_packet_id` | Eagle-Eye evidence link. |
| `source_attribute:return_packet_id` | `return_packet_id` | `trans` return link. |
| `source_attribute:release_packet_id` | `release_packet_id` | `trans-release` packet link. |
| `source_attribute:downstream_handoff_id` | `downstream_handoff_id` | Stable downstream-handoff identity. |
| `source_attribute:validation_status` | `validation_status` | Retains the source-owned validation result. |
| `source_attribute:release_status` | `release_status` | Retains the released state exactly. |
| `source_attribute:release_id` | `release_approval_ref.release_id` | Retains the approval correlation ID. |
| `source_attribute:approval_status` | `release_approval_ref.approval_status` | Retains the explicit approval state. |
| `source_attribute:target_layer` / `source_attribute:downstream_status` | `target_layer` / `downstream_status` | Retains the original routing and readiness states. |
| `source_attribute:overall_confidence` | `confidence.overall`, when supplied | Retains an optional bounded score; it is not a truth claim. |
| `source_attribute:bounded_packet_metadata_json` | Projected workflow, approval, evidence-boundary, and provenance fields | Keeps supported bounded metadata together for audit review. |
| `source_attribute:field_origin_json` | JSON from `field_origin`, when supplied | Retains optional field-level origin metadata. |
| `source_attribute:page_refs_json` / `source_attribute:region_refs_json` | Bounded references, when supplied | Retains page and region pointers without copying source content. |
| `source_attribute:evidence_boundary_json` | JSON from `evidence_boundary` | Retains the source packet's boundary flags. |
| `source_attribute:reporting_fields_json` | JSON from `reporting_fields` | Retains allowlisted bounded reporting values for analyst review. |

If the packet does not contain a page, region, or field-evidence reference, leave that mapped value empty; do not infer one from a filename or summary. Store each normalized reporting field as its own attribute instead of the JSON aggregate when analysts need to filter it individually. Keep the aggregate only when round-trip audit context is useful.

Do not create relationship rows from the presence of multiple fields in one packet. Create chart relationships only after an analyst reviews the record and identifies supported endpoints and evidence.
