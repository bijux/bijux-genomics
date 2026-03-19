# STAGE_MAPPING

Authority for stage catalog lives in `src/tool_adapters/stages/catalog.rs`.

| Stage ID | Tool Adapter(s) | Artifacts Emitted | Metrics Emitted |
| --- | --- | --- | --- |
| fastq.validate_reads | fastqvalidator | report.json | read_count, base_count |
| fastq.trim_reads | fastp | trimmed FASTQ | retention, bases_kept |
| fastq.merge | bbmerge | merged FASTQ | merge_rate |
| fastq.filter_reads | fastp | filtered FASTQ | filter_counts |
| fastq.screen_taxonomy | fastq_screen | screened FASTQ | contaminant_rate |
| fastq.report_qc | fastqc | qc report | qc_metrics |
| fastq.profile_reads | seqkit | stats report | read_count, base_count |
| fastq.correct | bayeshammer | corrected FASTQ | correction_rate |
| fastq.umi | umi_tools | umi FASTQ | umi_stats |
