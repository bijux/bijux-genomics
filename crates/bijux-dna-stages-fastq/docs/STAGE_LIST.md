# STAGE_LIST

Legend:
- Essential = required for default pipelines.
- Recommended = optional but commonly used.
- Optional = only when explicitly requested or dictated by the sample type.

See `STAGE_CONTRACTS.md` for detailed contracts.

| Stage | Class | Inputs | Outputs | Metrics |
| --- | --- | --- | --- | --- |
| fastq.validate_reads | Essential | FASTQ | validation report | read_count, base_count, format errors |
| fastq.detect_adapters | Recommended | FASTQ | adapter evidence report | evidence-only adapter inspection summary |
| fastq.trim_polyg_tails | Recommended | FASTQ | trimmed FASTQ | governed polyG-tail trimming semantics plus backend-native report provenance |
| fastq.trim_reads | Essential | FASTQ | trimmed FASTQ | retention, bases_kept |
| fastq.filter_reads | Recommended | FASTQ | filtered FASTQ | filter counts |
| fastq.filter_low_complexity | Optional | FASTQ | filtered FASTQ | reads_removed_low_complexity |
| fastq.merge_pairs | Optional | paired FASTQ | merged FASTQ | merge_rate |
| fastq.remove_duplicates | Optional | FASTQ | deduplicated FASTQ | dedup_rate |
| fastq.deplete_host | Optional | FASTQ + host reference index | host-depleted FASTQ | host_fraction_removed |
| fastq.deplete_rrna | Optional | FASTQ | rRNA-filtered FASTQ | rrna_fraction |
| fastq.correct_errors | Optional | paired FASTQ | corrected FASTQ | correction_rate; `rcorrector` is the closed execution backend |
| fastq.extract_umis | Optional | paired FASTQ | UMI-tagged FASTQ | umi_stats |
| fastq.screen_taxonomy | Optional | FASTQ | governed taxonomy report + raw screen summary | contamination_rate, classified_fraction, top_taxa, database lineage |
| fastq.profile_reads | Optional | FASTQ | stats report | read_count, base_count |
| fastq.profile_read_lengths | Optional | FASTQ | length report | length histogram |
| fastq.profile_overrepresented_sequences | Optional | FASTQ | sequence report | flagged sequence counts |
| fastq.report_qc | Optional | upstream QC reports | governed aggregation report + MultiQC bundle | contributor lineage, aggregation scope, qc summary |
