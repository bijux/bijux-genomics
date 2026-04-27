# Operational Contract (BAM)

## What
Defines required artifacts and metrics per stage.

## Why
Makes expected outputs explicit for validation.

## Non-goals
- Scientific interpretation (see [METHODOLOGICAL_INTENT.md](METHODOLOGICAL_INTENT.md)).

## Contracts
- The governed BAM artifact inventory lives in [../../../domain/bam/artifacts.yaml](../../../domain/bam/artifacts.yaml).
- Stage-level required outputs and defaults live in [STAGE_CATALOG.md](STAGE_CATALOG.md).
- Scientific meaning stays separated in [METHODOLOGICAL_INTENT.md](METHODOLOGICAL_INTENT.md).

## Examples
- `bam.align` emits `align_report_json`.
- `bam.coverage` emits `coverage_report_json`.
- `bam.contamination` emits `contamination_report_json`.

## Failure modes
- Missing required artifacts fail contract enforcement.
- Ad hoc artifact names make downstream validation and report assembly non-comparable across runs.
