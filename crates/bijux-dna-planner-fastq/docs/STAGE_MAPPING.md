# STAGE_MAPPING

Authority for planner-stage bindings lives in `src/tool_adapters/fastq.rs` plus the FASTQ domain manifests.

| Stage ID | Tool Adapter(s) | Artifacts Emitted | Metrics Emitted |
| --- | --- | --- | --- |
| fastq.validate_reads | fastqvalidator, fqtools, seqtk | validation report | reads_total, reads_invalid, mean_q |
| fastq.profile_read_lengths | seqkit_stats, prinseq, fastp | length report | length histogram |
| fastq.detect_adapters | fastqc | adapter evidence report | candidate_adapter_count |
| fastq.profile_overrepresented_sequences | fastqc, seqkit | overrepresented-sequence report | flagged sequence counts |
| fastq.index_reference | bowtie2_build, star | reference index directory | runtime_s, index_file_count, index_bytes |
| fastq.trim_polyg_tails | fastp, bbduk | trimmed FASTQ, trim report | polyG/polyX trimming counts |
| fastq.trim_terminal_damage | cutadapt, seqkit | terminal-damage-trimmed FASTQ, trim report | reads_out |
| fastq.trim_reads | fastp, cutadapt, atropos, bbduk, adapterremoval, trimmomatic, trim_galore, prinseq, seqkit, skewer, leehom, alientrimmer, fastx_clipper | trimmed FASTQ | retention, bases_kept |
| fastq.filter_reads | fastp, seqkit, prinseq, bbduk | filtered FASTQ | filter counts |
| fastq.deplete_reference_contaminants | bowtie2 | contaminant-screened FASTQ, contaminant screen report | reads_removed_contaminant_kmer |
| fastq.filter_low_complexity | prinseq, bbduk | filtered FASTQ, low-complexity report | reads_removed_low_complexity |
| fastq.merge_pairs | pear, vsearch, bbmerge, flash2, leehom | merged FASTQ, unmerged mates, merge report | merge_rate |
| fastq.remove_duplicates | fastuniq, clumpify | deduplicated FASTQ | dedup_rate |
| fastq.deplete_host | bowtie2 | host-depleted FASTQ, host depletion report | host_fraction_removed |
| fastq.deplete_rrna | sortmerna | rRNA-filtered FASTQ, rRNA report | rrna_fraction |
| fastq.correct_errors | rcorrector, musket, lighter, bayeshammer | corrected FASTQ | correction_rate |
| fastq.extract_umis | umi_tools | UMI-tagged FASTQ | umi_stats |
| fastq.screen_taxonomy | kraken2, krakenuniq, centrifuge, kaiju | screen report, classification report | contamination_rate |
| fastq.profile_reads | seqkit_stats | stats report | read_count, base_count |
| fastq.normalize_primers | cutadapt, seqkit | primer-normalized FASTQ | primer_trimmed_fraction |
| fastq.remove_chimeras | vsearch | chimera-filtered FASTQ, chimera report | chimera_fraction |
| fastq.cluster_otus | vsearch | OTU table, representative FASTA, taxonomy-ready FASTA/FASTQ | otu_count |
| fastq.normalize_abundance | seqkit | normalized abundance table | table_rows |
| fastq.report_qc | multiqc | qc report | qc summary |
