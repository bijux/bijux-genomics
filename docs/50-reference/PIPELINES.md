# Pipelines

## What
Pipeline profiles define scientific intent and defaults from
[SCIENTIFIC_DEFAULTS.md](../20-science/SCIENTIFIC_DEFAULTS.md).

## Why
Keeps selection and execution reproducible.

## Non-goals
- Tool selection logic.

## Contracts
- Profile IDs are sourced from
  [crates/bijux-dna-core/src/id_catalog/pipeline/](../../crates/bijux-dna-core/src/id_catalog/pipeline/).
- Compatibility expectations are published in [COMPATIBILITY_MATRIX.md](COMPATIBILITY_MATRIX.md).

## Examples
- fastq-to-fastq__default__v1.

## Failure modes
- Invalid pipeline IDs fail validation.
