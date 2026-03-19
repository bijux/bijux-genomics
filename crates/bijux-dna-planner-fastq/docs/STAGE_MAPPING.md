# STAGE_MAPPING

Authority for planner-stage bindings lives in `src/tool_adapters/stages/catalog.rs` plus the FASTQ domain manifests.

| Stage ID | Tool Adapter(s) | Artifacts Emitted | Metrics Emitted |
| --- | --- | --- | --- |
| fastq.validate_reads | fastqvalidator, fqtools, seqtk | validation report | reads_total, reads_invalid, mean_q |
| fastq.detect_adapters | fastqc | adapter report | adapter evidence summary |
| fastq.trim_polyg_tails | fastp, bbduk | trimmed FASTQ, trim report | polyG/polyX trimming counts |
| fastq.trim_reads | fastp, cutadapt, atropos, bbduk, adapterremoval, trimmomatic, trim_galore, seqpurge, prinseq, seqkit, skewer, leehom, alientrimmer, fastx_clipper | trimmed FASTQ | retention, bases_kept |
| fastq.filter_reads | fastp, seqkit, prinseq, bbduk | filtered FASTQ | filter counts |
| fastq.filter_low_complexity | prinseq, bbduk, fastp | filtered FASTQ, low-complexity report | reads_removed_low_complexity |
| fastq.merge_pairs | pear, vsearch, bbmerge, flash2, leehom | merged FASTQ, merge report | merge_rate |
| fastq.remove_duplicates | fastuniq, clumpify | deduplicated FASTQ | dedup_rate |
| fastq.deplete_host | bowtie2 | host-depleted FASTQ, host depletion report | host_fraction_removed |
| fastq.deplete_rrna | sortmerna | rRNA-filtered FASTQ, rRNA report | rrna_fraction |
| fastq.correct_errors | rcorrector, musket, lighter, bayeshammer | corrected FASTQ | correction_rate |
| fastq.extract_umis | umi_tools | UMI-tagged FASTQ | umi_stats |
| fastq.screen_taxonomy | kraken2, krakenuniq, diamond, centrifuge, metaphlan, kaiju, fastq_screen | screening report, classification report | contaminant_rate |
| fastq.profile_reads | seqkit_stats | stats report | read_count, base_count |
| fastq.profile_read_lengths | seqkit_stats, seqfu, prinseq, fastp | length report | length histogram |
| fastq.profile_overrepresented_sequences | fastqc, seqkit | overrepresented-sequence report | flagged sequence counts |
| fastq.report_qc | multiqc | qc report | qc summary |
