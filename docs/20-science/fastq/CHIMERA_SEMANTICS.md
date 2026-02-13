# Chimera Semantics

## Purpose
Define chimera detection semantics for ecological FASTQ analysis.

## Scope
`fastq.chimera_detection` in de-novo and reference-assisted modes.

## Non-goals
- Claiming complete chimera removal.
- Replacing manual ecological interpretation.

## Contracts
- Mode (de-novo/reference) is explicit in run metadata.
- Chimera-filtered and flagged outputs are distinguishable.
- Threshold changes are reflected in report artifacts.

## Examples
- De-novo mode for unsupported markers.
- Reference mode when curated marker databases are available.

## Failure modes
- Low coverage can overcall chimeras.
- Incomplete reference catalogs can undercall chimeras.
