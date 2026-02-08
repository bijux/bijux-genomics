# Operational Contract (BAM)

## What
Defines required artifacts and metrics per stage.

## Why
Makes expected outputs explicit for validation.

## Non-goals
- Scientific interpretation (see METHODOLOGICAL_INTENT.md).

## Contracts
Stage contracts in `crates/bijux-dna-stages-bam`.

## Examples
- bam.markdup -> metrics.json + stage_report.json

## Failure modes
Missing required artifacts fail contract enforcement.
