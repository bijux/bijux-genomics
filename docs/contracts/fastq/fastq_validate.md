# fastq.validate contract (v1)

## Inputs
- SE: reads_r1 (FASTQ)
- PE: reads_r1 + reads_r2 (FASTQ)

## Outputs
- None (validation only)

## Mandatory metrics
- reads_total
- reads_valid
- reads_invalid
- mean_q

## Allowed side-effects
- Writes reports/logs only.

## Failure vs warning
- Default mode: diagnostic tools never hard-fail.
- Strict mode: gatekeeper tools (fastqvalidator_official) hard-fail on invalid FASTQ.
- Any FASTQ output is a hard failure.
