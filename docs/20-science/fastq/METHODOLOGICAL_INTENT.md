# Methodological Intent (FASTQ)

## What
Defines the scientific meaning of FASTQ stages.

## Why
Separates scientific intent from operational contracts.

## Non-goals
- Tool execution details.

## Contracts
- The cross-stage scientific contract lives in [SCIENTIFIC_SPEC.md](SCIENTIFIC_SPEC.md).
- Stage-level purpose and inputs/outputs live in [STAGE_CATALOG.md](STAGE_CATALOG.md).
- Per-stage interpretation limits live in [STAGE_ASSUMPTIONS.md](STAGE_ASSUMPTIONS.md).

## Examples
- validate: ensures input is syntactically correct.
- trim: removes adapters/low-quality tails.
- merge: combines overlapping paired reads.
- screen: identifies contaminant signatures.

## Failure modes
If intent is unclear, update this file before changing stages.
