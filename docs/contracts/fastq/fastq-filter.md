# FASTQ Filter Contract v1

Status: frozen (write-only spec). No code changes implied.

## Purpose
Filter reads based on quality/length or other criteria, producing a subset of the input reads.

## Inputs
- SE: `R1.fastq.gz` (required)
- PE: `R1.fastq.gz` + `R2.fastq.gz` (optional; tool may handle pairs)

## Outputs
- SE: one filtered FASTQ `R1.filter.fastq.gz`
- PE: two filtered FASTQ files `R1.filter.fastq.gz`, `R2.filter.fastq.gz`

Naming is enforced by the runner and mapped from tool-specific outputs.

## Mandatory metrics
- `reads_in`, `reads_out`
- `bases_in`, `bases_out`
- `mean_q_before`, `mean_q_after`
- `runtime_s`, `memory_mb`, `exit_code`

## Allowed side-effects
- Tool logs/reports inside the stage output directory.

## Forbidden side-effects
- Reordering reads unless documented by the tool.
- Outputting more reads than input.

## Failure vs warning
Failure (hard):
- Missing output FASTQ.
- Output FASTQ fails to parse.
- `reads_out` > `reads_in` or `bases_out` > `bases_in`.

Warning (soft, record only):
- Tool emits non-fatal warnings.

## Determinism
Output must be deterministic for the same inputs and parameters.
