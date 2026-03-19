# STAGE_LIST

Legend:
- Essential = required for default pipelines.
- Recommended = optional but commonly used.
- Optional = only when requested.

See `STAGE_CONTRACTS.md` for contract details.

| Stage | Class | Inputs | Outputs | Metrics |
| --- | --- | --- | --- | --- |
| fastq.validate_reads | Essential | FASTQ | report.json | read_count, base_count |
| fastq.trim_reads | Essential | FASTQ | trimmed FASTQ | retention, bases_kept |
| fastq.merge | Recommended | paired FASTQ | merged FASTQ | merge_rate |
| fastq.filter_reads | Recommended | FASTQ | filtered FASTQ | filter_counts |
| fastq.screen_taxonomy | Optional | FASTQ | screened FASTQ | contaminant_rate |
| fastq.report_qc | Optional | FASTQ | qc report | qc_metrics |
| fastq.profile_reads | Optional | FASTQ | stats report | read_count, base_count |
| fastq.correct | Optional | FASTQ | corrected FASTQ | correction_rate |
| fastq.umi | Optional | FASTQ | umi FASTQ | umi_stats |
