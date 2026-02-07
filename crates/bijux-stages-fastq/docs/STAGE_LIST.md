# STAGE_LIST

## Essentials
- `fastq.validate` — Validate FASTQ formatting.
  - Inputs: FASTQ
  - Outputs: validation report
  - Metrics: read count, base count
- `fastq.trim` — Adapter trimming and quality filtering.
  - Inputs: FASTQ
  - Outputs: trimmed FASTQ
  - Metrics: retention (numerator/denominator), bases kept

## Recommended
- `fastq.merge` — Merge paired reads where applicable.
  - Inputs: paired FASTQ
  - Outputs: merged FASTQ
  - Metrics: merge rate

## Optional
- `fastq.screen` — Screen contaminants.
  - Inputs: FASTQ
  - Outputs: screened FASTQ
  - Metrics: contaminant proportion
