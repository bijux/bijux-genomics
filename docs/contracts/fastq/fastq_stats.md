# fastq.stats contract (v1)

## Inputs
- SE: reads_r1 (FASTQ)
- PE: reads_r1 + reads_r2 (FASTQ)

## Outputs
- stats_json (JSON) containing neutral stats

## Mandatory metrics
- reads_total
- bases_total
- mean_q
- gc_percent
- length_histogram

## Allowed side-effects
- Writes stats JSON only.

## Failure vs warning
- Fail: missing stats output, parse errors, invalid metrics.
- Warn: none.
