# FASTQ Trim Contract v1

Status: frozen (write-only spec). No code changes implied.

## Purpose
Remove adapters and low-quality bases/reads from FASTQ while preserving read order as much as the tool allows.

## Inputs
- SE: `R1.fastq.gz` (required)
- PE: `R1.fastq.gz` + `R2.fastq.gz` (optional; tool may handle pairs)

## Outputs
- SE: one trimmed FASTQ file `R1.trim.fastq.gz`
- PE: two trimmed FASTQ files `R1.trim.fastq.gz`, `R2.trim.fastq.gz`

Naming is enforced by the runner and mapped from tool-specific outputs.

## Mandatory metrics
- `reads_in`, `reads_out`
- `bases_in`, `bases_out`
- `mean_q_before`, `mean_q_after`
- `runtime_s`, `memory_mb`, `exit_code`

## Allowed side-effects
- Creation of tool logs and optional reports in the stage output directory.

## Forbidden side-effects
- Writing output outside the stage output directory.
- Re-encoding FASTQ with lossy transformations.

## Failure vs warning
Failure (hard):
- Missing output FASTQ.
- `reads_out` > `reads_in` or `bases_out` > `bases_in`.
- Output FASTQ fails to parse.

Warning (soft, record only):
- Minor changes in read ordering.
- Tool-specific warnings that do not affect output validity.

## Determinism
Output must be deterministic for the same input and parameters.
