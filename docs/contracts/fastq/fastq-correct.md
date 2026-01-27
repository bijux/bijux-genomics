# FASTQ Correct Contract v1

Status: frozen (write-only spec). No code changes implied.

## Purpose
Error-correct FASTQ reads (typically k-mer based) without changing read identifiers.

## Inputs
- SE: `R1.fastq.gz` (required)
- PE: `R1.fastq.gz` + `R2.fastq.gz` (optional; tool may handle pairs)

## Outputs
- SE: one corrected FASTQ `R1.correct.fastq.gz`
- PE: two corrected FASTQ files `R1.correct.fastq.gz`, `R2.correct.fastq.gz`

Naming is enforced by the runner and mapped from tool-specific outputs.

## Mandatory metrics
- `reads_in`, `reads_out`
- `bases_in`, `bases_out`
- `mean_q_before`, `mean_q_after`
- `runtime_s`, `memory_mb`, `exit_code`

## Allowed side-effects
- Tool logs/reports inside the stage output directory.

## Forbidden side-effects
- Dropping reads unless explicitly reported in metrics.
- Changing read identifiers.

## Failure vs warning
Failure (hard):
- Missing output FASTQ.
- Output FASTQ fails to parse.
- `reads_out` > `reads_in` or `bases_out` > `bases_in`.

Warning (soft, record only):
- Tool emits non-fatal warnings.

## Determinism
Output must be deterministic for the same inputs and parameters.
