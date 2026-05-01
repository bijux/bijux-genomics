# Scope

`bijux-dna-db-ref` owns species/build reference resolution contracts for VCF
planning and runtime provenance.

## In Scope

- Species alias and default-build resolution.
- Species authority, contig map, sex chromosome, and coverage profile lookup.
- Reference bank, bundle, provenance, organellar policy, and default
  reference-set lookup.
- Panel and genetic map catalog lookup with lock validation.
- Tool compatibility checks for panel/map pairs.

## Out of Scope

- Pipeline planning.
- Stage execution.
- Runtime backend orchestration.
- Network access, downloads, and filesystem writes.
- CLI command parsing and product API handlers.

## Policy Reference

- Workspace style and boundary policy:
  `README.md`, `README.md`, and
  repository `docs/40-policies/STYLE.md`.
