# FASTQ Stage Catalog

## What
Canonical FASTQ stage definitions for preprocessing, screening, and amplicon-specific transforms.

## Why
Keeps stage names tied to stable inputs, outputs, and mutations so tool support does not drift across review cycles.

## Non-goals
- Exhaustive benchmark guidance.
- BAM-level post-alignment QC contracts.

## Contracts
- A stage name should describe one stable transformation boundary.
- Report-only stages must not be described as mutating reads.
- Tool lists here must match the FASTQ domain manifests.

### fastq.validate_reads {#fastq-validate-pre}
- Purpose: validate FASTQ structure and parser-level integrity before any mutating stage.
- Inputs/Outputs: reads -> validation_report.
- Metrics: read counts, format errors, parser pass/fail.
- Tools: fastqvalidator, fqtools, seqtk.

### fastq.detect_adapters {#fastq-detect-adapters}
- Purpose: report adapter evidence without changing reads.
- Inputs/Outputs: reads -> adapter_report.
- Metrics: adapter-content summary.
- Tools: fastqc.

### fastq.trim_polyg_tails {#fastq-trim-polyg}
- Purpose: trim polyG/polyX sequencer tail artifacts.
- Inputs/Outputs: reads -> trimmed_reads, report_json.
- Metrics: trimmed-read counts, tail prevalence before/after.
- Tools: fastp, bbduk.

### fastq.trim_reads {#fastq-trim}
- Purpose: remove adapters and low-quality terminal sequence.
- Inputs/Outputs: reads -> trimmed_reads.
- Metrics: reads retained, bases retained, trimming counts.
- Tools: fastp, cutadapt, atropos, bbduk, adapterremoval, trimmomatic, trim_galore.

### fastq.filter_reads {#fastq-filter}
- Purpose: remove reads failing quality, length, or simple content thresholds.
- Inputs/Outputs: reads -> filtered_reads.
- Metrics: reads removed by reason, retention.
- Tools: fastp, seqkit, prinseq, bbduk.

### fastq.filter_low_complexity {#fastq-low-complexity}
- Purpose: remove low-complexity reads with dedicated complexity-aware filters.
- Inputs/Outputs: reads -> filtered_fastq, filter_report_json.
- Metrics: reads_removed_low_complexity.
- Tools: prinseq, bbduk, fastp.

### fastq.profile_read_lengths {#fastq-read-lengths}
- Purpose: compute neutral read-length summaries without mutating reads.
- Inputs/Outputs: reads -> length_distribution_tsv, length_distribution_json.
- Metrics: length distributions, read counts.
- Tools: seqkit_stats, prinseq, fastp.

### fastq.profile_reads {#fastq-stats-neutral}
- Purpose: compute neutral read-level summary statistics.
- Inputs/Outputs: reads -> qc_json, qc_tsv, qc_plots_dir.
- Metrics: read totals, base totals, quality summaries.
- Tools: seqkit_stats.

### fastq.profile_overrepresented_sequences {#fastq-overrepresented}
- Purpose: report overrepresented sequences and recurring contaminants.
- Inputs/Outputs: reads -> overrepresented_sequences_tsv, overrepresented_sequences_json.
- Metrics: overrepresented sequence counts and flags.
- Tools: fastqc, seqkit.

### fastq.merge_pairs {#fastq-merge}
- Purpose: merge overlapping paired-end reads.
- Inputs/Outputs: paired reads -> merged_reads, report_json.
- Metrics: merged pairs, unmerged pairs, merge rate.
- Tools: pear, vsearch, bbmerge, flash2, leehom.

### fastq.remove_duplicates {#fastq-deduplicate}
- Purpose: remove duplicate reads in raw FASTQ space.
- Inputs/Outputs: reads -> dedup_reads_r1/dedup_reads_r2, report_json.
- Metrics: duplicate counts, dedup_rate.
- Tools: fastuniq, clumpify.

### fastq.deplete_host {#fastq-host-depletion}
- Purpose: remove reads matching an explicit host reference by mapping.
- Inputs/Outputs: reads + reference_index -> host_depleted_reads_r1/host_depleted_reads_r2, host_depletion_report_json.
- Metrics: reads_out, bases_out, host_fraction_removed.
- Tools: bowtie2.

### fastq.deplete_reference_contaminants {#fastq-reference-contaminants}
- Purpose: remove reads matching configured decoy or contaminant references.
- Inputs/Outputs: reads + contaminant reference -> contaminant_screened_reads, contaminant_screen_report_json.
- Metrics: contaminant-screened retention.
- Tools: bowtie2.

### fastq.deplete_rrna {#fastq-rrna}
- Purpose: remove rRNA-derived reads from raw FASTQ.
- Inputs/Outputs: reads -> rrna_filtered_reads_r1/rrna_filtered_reads_r2, rrna_report_tsv/rrna_report_json.
- Metrics: rRNA hits, rRNA fraction, retained reads.
- Tools: sortmerna.

### fastq.correct_errors {#fastq-correct}
- Purpose: correct sequencing errors while preserving read pairs.
- Inputs/Outputs: paired reads -> corrected_reads_r1/corrected_reads_r2.
- Metrics: corrected reads, corrected bases, quality shift.
- Tools: rcorrector, musket, lighter, bayeshammer.

### fastq.extract_umis {#fastq-umi}
- Purpose: extract UMIs and propagate them into read identifiers without dropping reads.
- Inputs/Outputs: paired reads -> umi_reads_r1/umi_reads_r2.
- Metrics: reads_with_umi, reads_in, reads_out.
- Tools: umi_tools.

### fastq.screen_taxonomy {#fastq-screen}
- Purpose: classify reads for taxonomic screening and contamination assessment.
- Inputs/Outputs: reads -> screen_report_tsv, classification_report_json.
- Metrics: classified/unclassified reads, contamination summaries.
- Tools: kraken2, krakenuniq, centrifuge, metaphlan, kaiju, fastq_screen.

### fastq.report_qc {#fastq-qc-post}
- Purpose: aggregate QC outputs after read-level preprocessing.
- Inputs/Outputs: stage reports -> multiqc_report/multiqc_data.
- Metrics: QC module summaries, warnings, failures.
- Tools: multiqc.

### fastq.trim_terminal_damage {#fastq-damage-pretrim}
- Purpose: trim or mask terminal damage signatures in aDNA-like libraries.
- Inputs/Outputs: reads -> trimmed_reads, report_json.
- Metrics: terminal asymmetry before/after trimming.
- Tools: cutadapt, seqkit.

### fastq.normalize_primers {#fastq-primer-normalization}
- Purpose: remove or normalize primer sequence with explicit orientation control.
- Inputs/Outputs: reads -> normalized_reads_r1/normalized_reads_r2, primer_orientation_report, primer_stats_json.
- Metrics: primer-trimmed reads, retained reads.
- Tools: cutadapt, seqkit.

### fastq.remove_chimeras {#fastq-remove-chimeras}
- Purpose: remove chimeric sequences in amplicon-oriented workflows.
- Inputs/Outputs: reads -> chimera_filtered_reads_r1/chimera_filtered_reads_r2, chimera_metrics_json, chimeras_fasta.
- Metrics: chimera counts, retained reads.
- Tools: vsearch.

### fastq.cluster_otus {#fastq-cluster-otus}
- Purpose: cluster reads into operational taxonomic units for amplicon workflows.
- Inputs/Outputs: reads -> otu_table, otu_representatives, taxonomy_ready_fasta, taxonomy_ready_fastq.
- Metrics: cluster counts, representative counts.
- Tools: vsearch.

### fastq.infer_asvs {#fastq-infer-asvs}
- Purpose: infer amplicon sequence variants.
- Inputs/Outputs: reads -> asv_table_tsv, asv_sequences_fasta, taxonomy_ready_fasta, taxonomy_ready_fastq.
- Metrics: inferred variant counts.
- Tools: no admitted backend yet.

### fastq.normalize_abundance {#fastq-normalize-abundance}
- Purpose: normalize abundance summaries after amplicon inference.
- Inputs/Outputs: feature table -> normalized_abundance_tsv.
- Metrics: normalized abundance summaries.
- Tools: seqkit.
