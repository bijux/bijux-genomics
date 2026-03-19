# STAGE_MAPPING

Authority for stage catalog lives in `src/tool_adapters/stages/catalog.rs`.

| Stage ID | Tool Adapter(s) | Artifacts Emitted | Metrics Emitted |
| --- | --- | --- | --- |
| fastq.validate_reads | fastqvalidator, fqtools, seqtk | validation.json | reads_total, reads_invalid, mean_q |
| fastq.trim_reads | fastp | trimmed FASTQ | retention, bases_kept |
| fastq.merge_pairs | bbmerge | merged FASTQ | merge_rate |
| fastq.filter_reads | fastp | filtered FASTQ | filter_counts |
| fastq.screen_taxonomy | fastq_screen | screened FASTQ | contaminant_rate |
| fastq.report_qc | fastqc | qc report | qc_metrics |
| fastq.profile_reads | seqkit | stats report | read_count, base_count |
| fastq.correct_errors | bayeshammer | corrected FASTQ | correction_rate |
| fastq.extract_umis | umi_tools | umi FASTQ | umi_stats |
