# Operational Contract (FASTQ)

## What
Defines required artifacts and metrics per stage.

## Why
Makes expected outputs explicit for validation.

## Non-goals
- Scientific interpretation (see METHODOLOGICAL_INTENT.md).

## Contracts
- Governed artifact IDs live in
  [domain/fastq/artifacts.yaml](../../../domain/fastq/artifacts.yaml).
- Governed metric IDs live in
  [domain/fastq/metrics.yaml](../../../domain/fastq/metrics.yaml).
- Stage-by-stage inputs and outputs are summarized in
  [STAGE_CATALOG.md](STAGE_CATALOG.md).

## Examples
- fastq.trim_reads -> metrics.json + stage_report.json

## Failure modes
Missing required artifacts fail contract enforcement.
