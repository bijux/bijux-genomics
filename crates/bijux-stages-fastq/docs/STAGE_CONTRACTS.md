# STAGE_CONTRACTS

| Stage | Inputs | Outputs | Metrics |
| --- | --- | --- | --- |
| fastq.validate | FASTQ | report.json | read_count, base_count |
| fastq.trim | FASTQ | trimmed FASTQ | retention, bases_kept |
| fastq.merge | paired FASTQ | merged FASTQ | merge_rate |
| fastq.screen | FASTQ | screened FASTQ | contaminant_rate |
