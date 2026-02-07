# STAGE_MAPPING

| Stage ID | Tool Adapter | Artifacts | Metrics |
| --- | --- | --- | --- |
| fastq.validate_pre | fastqvalidator | report.json | read_count, base_count |
| fastq.trim | fastp | trimmed FASTQ | retention, bases_kept |
| fastq.merge | bbmerge | merged FASTQ | merge_rate |
| fastq.filter | fastp | filtered FASTQ | filter_counts |
| fastq.screen | fastq_screen | screened FASTQ | contaminant_rate |
| fastq.qc_post | fastqc | qc report | qc_metrics |
| fastq.stats_neutral | seqkit | stats report | read_count, base_count |
| fastq.correct | bayeshammer | corrected FASTQ | correction_rate |
| fastq.umi | umi_tools | umi FASTQ | umi_stats |
| fastq.preprocess | pipeline | staged FASTQ | summary |
