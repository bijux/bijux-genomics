# FASTQ QC2 Contract v1

Status: frozen (write-only spec). No code changes implied.

## Purpose
Second-pass QC on FASTQ data (e.g., post-trim QC). Produces reports only.

## Inputs
- SE: `R1.fastq.gz` (required)
- PE: `R1.fastq.gz` + `R2.fastq.gz` (optional; tools may analyze both)

## Outputs
- No FASTQ output files are permitted.
- Reports (HTML/JSON/text) under the stage output directory.

## Mandatory metrics
- `reads_total`
- `bases_total`
- `mean_q`
- `runtime_s`, `memory_mb`, `exit_code`

## Allowed side-effects
- Report files in stage output directory.
- Logging to stdout/stderr.

## Forbidden side-effects
- Modifying or rewriting input FASTQ files.
- Emitting new FASTQ outputs.

## Failure vs warning
Failure (hard):
- Tool exits non-zero.
- Any FASTQ output file is created.

Warning (soft, record only):
- Missing optional report sections.

## Determinism
Metrics must be deterministic for the same inputs and parameters.
