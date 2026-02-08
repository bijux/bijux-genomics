# STAGE_LIST

Legend:
- Essential = required for default pipelines.
- Recommended = optional but commonly used.
- Optional = only when requested.

See `STAGE_CONTRACTS.md` for contract details.

| Stage | Class | Inputs | Outputs | Metrics |
| --- | --- | --- | --- | --- |
| fastq.validate_pre | Essential | FASTQ | report.json | read_count, base_count |
| fastq.trim | Essential | FASTQ | trimmed FASTQ | retention, bases_kept |
| fastq.merge | Recommended | paired FASTQ | merged FASTQ | merge_rate |
| fastq.filter | Recommended | FASTQ | filtered FASTQ | filter_counts |
| fastq.screen | Optional | FASTQ | screened FASTQ | contaminant_rate |
| fastq.qc_post | Optional | FASTQ | qc report | qc_metrics |
| fastq.stats_neutral | Optional | FASTQ | stats report | read_count, base_count |
| fastq.correct | Optional | FASTQ | corrected FASTQ | correction_rate |
| fastq.umi | Optional | FASTQ | umi FASTQ | umi_stats |
