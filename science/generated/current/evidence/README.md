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

## Adjacent Surfaces

- [../README.md](../README.md) explains the current generated snapshot boundary
- [../../indexes/README.md](../../indexes/README.md) explains the rolled-up JSON
  index entrypoints built from these row files
