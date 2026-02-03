# FASTQ Screen Contract v1

Status: frozen (write-only spec). No code changes implied.

## Purpose
Screen FASTQ reads against reference databases and report contamination or origin composition.

## Inputs
- SE: `R1.fastq.gz` (required)
- PE: `R1.fastq.gz` + `R2.fastq.gz` (optional)

## Outputs
- No FASTQ output files are permitted.
- Screening reports under the stage output directory.

## Mandatory metrics
- `reads_total`
- `bases_total`
- `contamination_rate`
- `runtime_s`, `memory_mb`, `exit_code`

## Allowed side-effects
- Report files in stage output directory.
- Logging to stdout/stderr.

## Forbidden side-effects
- Modifying or rewriting input FASTQ files.
- Emitting FASTQ outputs.

## Failure vs warning
Failure (hard):
- Tool exits non-zero.
- Any FASTQ output file is created.

Warning (soft, record only):
- Missing optional report sections.
- Unavailable reference database (if tool emits partial results).

## Determinism
Metrics must be deterministic for the same inputs and parameters.
