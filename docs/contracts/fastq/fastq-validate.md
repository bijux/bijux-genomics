# FASTQ Validate Contract v1

Status: frozen (write-only spec). No code changes implied.

## Purpose
Validate FASTQ structural integrity and report basic summary statistics. Validation does **not** modify the input data.

## Inputs
- SE: `R1.fastq.gz` (required)
- PE: `R1.fastq.gz` + `R2.fastq.gz` (optional; tools may validate each independently)

## Outputs
- No FASTQ output files are permitted.
- Tool may emit reports (text/HTML/JSON) under the stage output directory.

## Mandatory metrics
- `reads_total`
- `reads_invalid`
- `bases_total`
- `mean_q`
- `runtime_s`, `memory_mb`, `exit_code`

## Allowed side-effects
- Creation of report files in the stage output directory.
- Logging to stdout/stderr.

## Forbidden side-effects
- Modifying or rewriting input FASTQ files.
- Emitting trimmed/filtered FASTQ outputs.

## Failure vs warning
Failure (hard):
- Tool exits non-zero.
- Any FASTQ output file is created.
- `reads_invalid` < 0 or `reads_invalid` > `reads_total`.

Warning (soft, record only):
- Non-critical tool warnings in stdout/stderr.
- Reports missing optional sections.

## Determinism
For the same inputs and parameters, metrics must be deterministic within floating point precision.
