# Generated Evidence Snapshot

`science/generated/current/evidence/` holds the row-level science outputs for
the current authored workspace state.

## Traceability Ledgers

- [binding_resolution.tsv](binding_resolution.tsv) records which compiled
  bindings resolved to which targets and enforcement levels
- [claim_evidence_map.tsv](claim_evidence_map.tsv) records the claim-to-evidence
  traceability rows
- [decision_reasoning_map.tsv](decision_reasoning_map.tsv) records the
  decision-to-reasoning traceability rows

## Source Accounting

- [source_inventory.tsv](source_inventory.tsv) inventories governed sources
  consumed by the compiled science slice
- [source_archive_gaps.tsv](source_archive_gaps.tsv) records governed archive
  gaps that would block a closed source surface
- [unresolved_refs.json](unresolved_refs.json) records reference-resolution
  failures that must stay empty for a clean compiled state

## FASTQ Runtime and Closure Ledgers

- [fastq_stage_tool_environment_matrix.tsv](fastq_stage_tool_environment_matrix.tsv)
  inventories the governed FASTQ stage-tool runtime surface and the claims,
  decisions, and bindings that justify each row
- [fastq_container_reference_matrix.tsv](fastq_container_reference_matrix.tsv)
  inventories the governed FASTQ container references, version pins, and runtime
  artifacts behind each tool
- [fastq_download_backlog.tsv](fastq_download_backlog.tsv) tracks the current
  source-packet acquisition backlog for reviewed FASTQ tools
- [fastq_paper_archive_matrix.tsv](fastq_paper_archive_matrix.tsv) tracks the
  current FASTQ paper archive coverage for reviewed tool families
- [fastq_closure_gate.tsv](fastq_closure_gate.tsv) records closure status and
  blocker or warning reasons per FASTQ stage-tool binding
- [fastq_missing_closure_prerequisites.tsv](fastq_missing_closure_prerequisites.tsv)
  expands each closure blocker into one row per missing prerequisite
- [fastq_default_binding_risk_ledger.tsv](fastq_default_binding_risk_ledger.tsv)
  records the rolled-up risk class for each default FASTQ binding
- [fastq_truth_delta.tsv](fastq_truth_delta.tsv) records where observed compiled
  closure state still diverges from the expected governed truth surface

## Adjacent Surfaces

- [../README.md](../README.md) explains the current generated snapshot boundary
- [../../indexes/README.md](../../indexes/README.md) explains the rolled-up JSON
  index entrypoints built from these row files
