# FASTQ Merge Contract v1

Status: frozen (write-only spec). No code changes implied.

## Purpose
Merge paired-end reads and report merged/unmerged outputs.

## Inputs
- PE only: `R1.fastq.gz` + `R2.fastq.gz`

## Outputs
- Required: at least one of the following must exist:
  - `merged.fastq.gz`
  - `unmerged_R1.fastq.gz` + `unmerged_R2.fastq.gz`

Naming is enforced by the runner and mapped from tool-specific outputs.

## Mandatory metrics
- `reads_r1`, `reads_r2`
- `reads_merged`
- `reads_unmerged_r1`, `reads_unmerged_r2`
- `runtime_s`, `memory_mb`, `exit_code`

## Allowed side-effects
- Reports/logs within the stage output directory.

## Forbidden side-effects
- Writing outputs outside the stage output directory.

## Failure vs warning
Failure (hard):
- Neither merged nor unmerged outputs exist.
- `reads_merged` > min(`reads_r1`, `reads_r2`).
- Output FASTQ fails to parse.

Warning (soft, record only):
- Very low merge rate.
- Tool emits non-fatal warnings.

## Determinism
Output and metrics must be deterministic for the same inputs and parameters.
