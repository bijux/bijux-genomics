# Chimera Semantics

## Purpose
Define chimera detection semantics for ecological FASTQ analysis.

## Scope
`fastq.remove_chimeras` in de-novo and reference-assisted modes.

## Non-goals
- Claiming complete chimera removal.
- Replacing manual ecological interpretation.

## Contracts
- Chimera-stage inputs, outputs, invariants, and mode assumptions live in
  [domain/fastq/stages/remove_chimeras.yaml](../../../domain/fastq/stages/remove_chimeras.yaml).
- The pinned default backend and baseline invocation policy live in
  [domain/fastq/docs/DEFAULT_SETTINGS.md](../../../domain/fastq/docs/DEFAULT_SETTINGS.md).
- Reference-backed database and provenance expectations live in
  [REFERENCE_GOVERNANCE.md](REFERENCE_GOVERNANCE.md).

## Examples
- De-novo mode for unsupported markers.
- Reference mode when curated marker databases are available.

## Failure modes
- Low coverage can overcall chimeras.
- Incomplete reference catalogs can undercall chimeras.
- Unlocked or floating reference databases cause irreproducible chimera calls.
