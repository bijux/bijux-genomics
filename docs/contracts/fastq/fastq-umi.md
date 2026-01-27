# FASTQ UMI Contract v1

Status: frozen (write-only spec). No code changes implied.

## Purpose
Extract or annotate UMI sequences and emit updated FASTQ (and optional tag reports).

## Inputs
- SE: `R1.fastq.gz` (required)
- PE: `R1.fastq.gz` + `R2.fastq.gz` (optional; tool may use one or both reads)

## Outputs
- SE: one UMI-annotated FASTQ `R1.umi.fastq.gz`
- PE: two UMI-annotated FASTQ files `R1.umi.fastq.gz`, `R2.umi.fastq.gz`
- Optional: UMI summary report files

Naming is enforced by the runner and mapped from tool-specific outputs.

## Mandatory metrics
- `reads_in`, `reads_out`
- `bases_in`, `bases_out`
- `runtime_s`, `memory_mb`, `exit_code`

## Allowed side-effects
- Report files in stage output directory.

## Forbidden side-effects
- Dropping reads unless explicitly reported in metrics.
- Changing read identifiers beyond UMI annotation rules.

## Failure vs warning
Failure (hard):
- Missing output FASTQ.
- Output FASTQ fails to parse.
- `reads_out` > `reads_in` or `bases_out` > `bases_in`.

Warning (soft, record only):
- Tool emits non-fatal warnings.

## Determinism
Output must be deterministic for the same inputs and parameters.
