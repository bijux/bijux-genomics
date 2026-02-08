# STAGE_MAPPING

Authority for stage catalog lives in `src/tool_adapters/stages/catalog.rs`.

| Stage ID | Tool Adapter(s) | Artifacts Emitted | Metrics Emitted |
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
| fastq.preprocess | pipeline composition | staged FASTQ + summary | summary |
