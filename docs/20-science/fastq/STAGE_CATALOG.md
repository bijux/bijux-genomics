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
- Defaults: strict validation, header-sync checks only when paired reads are present.
- Tools: fastqvalidator, fqtools, seqtk.
- References: `domain/fastq/stages/validate_reads.yaml`, `domain/fastq/tools/`.

### fastq.detect_adapters {#fastq-detect-adapters}
- Purpose: report adapter evidence without changing reads.
- Inputs/Outputs: reads -> adapter_report.
- Metrics: adapter-content summary.
- Defaults: report-only detection with no read mutation.
- Tools: fastqc.
- References: `domain/fastq/stages/detect_adapters.yaml`, `domain/fastq/tools/`.

### fastq.trim_polyg_tails {#fastq-trim-polyg}
- Purpose: trim polyG/polyX sequencer tail artifacts.
- Inputs/Outputs: reads -> trimmed_reads, report_json.
- Metrics: trimmed-read counts, tail prevalence before/after.
- Defaults: sequencer-tail trimming is enabled only when an admitted backend declares the capability.
- Tools: fastp, bbduk.
- References: `domain/fastq/stages/trim_polyg_tails.yaml`, `domain/fastq/tools/`.

### fastq.trim_reads {#fastq-trim}
- Purpose: remove adapters and low-quality terminal sequence.
- Inputs/Outputs: reads -> trimmed_reads.
- Metrics: reads retained, bases retained, trimming counts.
- Defaults: planner uses governed adapter and quality policies from the FASTQ domain contract.
- Tools: fastp, cutadapt, atropos, bbduk, adapterremoval, trimmomatic, trim_galore.
- References: `domain/fastq/stages/trim_reads.yaml`, `domain/fastq/tools/`.

### fastq.filter_reads {#fastq-filter}
- Purpose: remove reads failing quality, length, or simple content thresholds.
- Inputs/Outputs: reads -> filtered_reads.
- Metrics: reads removed by reason, retention.
- Defaults: neutral pass-through until explicit quality, length, or content thresholds are requested.
- Tools: fastp, seqkit, prinseq, bbduk.
- References: `domain/fastq/stages/filter_reads.yaml`, `domain/fastq/tools/`.

### fastq.filter_low_complexity {#fastq-low-complexity}
- Purpose: remove low-complexity reads with dedicated complexity-aware filters.
- Inputs/Outputs: reads -> filtered_fastq, filter_report_json.
- Metrics: reads_removed_low_complexity.
- Defaults: no complexity filtering is applied until a threshold is bound for the stage.
- Tools: prinseq, bbduk, fastp.
- References: `domain/fastq/stages/filter_low_complexity.yaml`, `domain/fastq/tools/`.

### fastq.profile_read_lengths {#fastq-read-lengths}
- Purpose: compute neutral read-length summaries without mutating reads.
- Inputs/Outputs: reads -> length_distribution_tsv, length_distribution_json.
- Metrics: length distributions, read counts.
- Defaults: histogram-style summaries are emitted without altering read content.
- Tools: seqkit_stats, prinseq, fastp.
- References: `domain/fastq/stages/profile_read_lengths.yaml`, `domain/fastq/tools/`.

### fastq.profile_reads {#fastq-stats-neutral}
- Purpose: compute neutral read-level summary statistics.
- Inputs/Outputs: reads -> qc_json, qc_tsv, qc_plots_dir.
- Metrics: read totals, base totals, quality summaries.
- Defaults: neutral read statistics only; no filtering or trimming side effects.
- Tools: seqkit_stats.
- References: `domain/fastq/stages/profile_reads.yaml`, `domain/fastq/tools/`.

### fastq.profile_overrepresented_sequences {#fastq-overrepresented}
- Purpose: report overrepresented sequences and recurring contaminants.
- Inputs/Outputs: reads -> overrepresented_sequences_tsv, overrepresented_sequences_json.
- Metrics: overrepresented sequence counts and flags.
- Defaults: descriptive reporting only, leaving downstream remediation to later explicit stages.
- Tools: fastqc, seqkit.
- References: `domain/fastq/stages/profile_overrepresented_sequences.yaml`, `domain/fastq/tools/`.

### fastq.merge_pairs {#fastq-merge}
- Purpose: merge overlapping paired-end reads.
- Inputs/Outputs: paired reads -> merged_reads, report_json.
- Metrics: merged pairs, unmerged pairs, merge rate.
- Defaults: unmerged mates remain available unless a backend-specific policy says otherwise.
- Tools: pear, vsearch, bbmerge, flash2, leehom.
- References: `domain/fastq/stages/merge_pairs.yaml`, `domain/fastq/tools/`.

### fastq.remove_duplicates {#fastq-deduplicate}
- Purpose: remove duplicate reads in raw FASTQ space.
- Inputs/Outputs: reads -> dedup_reads_r1/dedup_reads_r2, report_json.
- Metrics: duplicate counts, dedup_rate.
- Defaults: exact duplicate handling with stable output ordering when the backend supports it.
- Tools: fastuniq, clumpify.
- References: `domain/fastq/stages/remove_duplicates.yaml`, `domain/fastq/tools/`.

### fastq.deplete_host {#fastq-host-depletion}
- Purpose: remove reads matching an explicit host reference by mapping.
- Inputs/Outputs: reads + reference_index -> host_depleted_reads_r1/host_depleted_reads_r2, host_depletion_report_json.
- Metrics: reads_out, bases_out, host_fraction_removed.
- Defaults: retain unmapped reads and require an explicit host reference binding.
- Tools: bowtie2.
- References: `domain/fastq/stages/deplete_host.yaml`, `domain/fastq/tools/`.

### fastq.deplete_reference_contaminants {#fastq-reference-contaminants}
- Purpose: remove reads matching configured decoy or contaminant references.
- Inputs/Outputs: reads + contaminant reference -> contaminant_screened_reads, contaminant_screen_report_json.
- Metrics: contaminant-screened retention.
- Defaults: decoy removal stays off until explicit contaminant references are configured.
- Tools: bowtie2.
- References: `domain/fastq/stages/deplete_reference_contaminants.yaml`, `domain/fastq/tools/`.

### fastq.deplete_rrna {#fastq-rrna}
- Purpose: remove rRNA-derived reads from raw FASTQ.
- Inputs/Outputs: reads -> rrna_filtered_reads_r1/rrna_filtered_reads_r2, rrna_report_tsv/rrna_report_json.
- Metrics: rRNA hits, rRNA fraction, retained reads.
- Defaults: conservative rRNA depletion against the configured reference catalog.
- Tools: sortmerna.
- References: `domain/fastq/stages/deplete_rrna.yaml`, `domain/fastq/tools/`.

### fastq.correct_errors {#fastq-correct}
- Purpose: correct sequencing errors while preserving read pairs.
- Inputs/Outputs: paired reads -> corrected_reads_r1/corrected_reads_r2.
- Metrics: corrected reads, corrected bases, quality shift.
- Defaults: phred33 quality interpretation with backend-owned correction heuristics.
- Tools: rcorrector, musket, lighter, bayeshammer.
- References: `domain/fastq/stages/correct_errors.yaml`, `domain/fastq/tools/`.

### fastq.extract_umis {#fastq-umi}
- Purpose: extract UMIs and propagate them into read identifiers without dropping reads.
- Inputs/Outputs: paired reads -> umi_reads_r1/umi_reads_r2.
- Metrics: reads_with_umi, reads_in, reads_out.
- Defaults: no-op until an explicit UMI pattern or extraction contract is bound.
- Ordering: must run after structural FASTQ validation and before trim/filter stages when inline UMIs are requested.
- Tools: umi_tools.
- References: `domain/fastq/stages/extract_umis.yaml`, `domain/fastq/tools/`.

### fastq.screen_taxonomy {#fastq-screen}
- Purpose: classify reads for taxonomic screening and contamination assessment.
- Inputs/Outputs: reads -> screen_report_tsv, classification_report_json.
- Metrics: classified/unclassified reads, contamination summaries.
- Defaults: descriptive classification only; no reads are removed by this stage.
- Tools: kraken2, krakenuniq, centrifuge, kaiju.
- References: `domain/fastq/stages/screen_taxonomy.yaml`, `domain/fastq/tools/`.

### fastq.report_qc {#fastq-qc-post}
- Purpose: aggregate QC outputs after read-level preprocessing.
- Inputs/Outputs: stage reports -> multiqc_report/multiqc_data.
- Metrics: QC module summaries, warnings, failures.
- Defaults: governed QC artifact aggregation using MultiQC-compatible report inputs.
- Tools: multiqc.
- References: `domain/fastq/stages/report_qc.yaml`, `domain/fastq/tools/`.

### fastq.trim_terminal_damage {#fastq-damage-pretrim}
- Purpose: trim or mask terminal damage signatures in aDNA-like libraries.
- Inputs/Outputs: reads -> trimmed_reads, report_json.
- Metrics: terminal asymmetry before/after trimming.
- Defaults: ancient-DNA-oriented terminal trimming with symmetric short-end clipping; generic default and minimal FASTQ profiles do not require this stage.
- Tools: cutadapt, seqkit.
- References: `domain/fastq/stages/trim_terminal_damage.yaml`, `domain/fastq/tools/`.

### fastq.normalize_primers {#fastq-primer-normalization}
- Purpose: remove or normalize primer sequence with explicit orientation control.
- Inputs/Outputs: reads -> normalized_reads_r1/normalized_reads_r2, primer_orientation_report, primer_stats_json.
- Metrics: primer-trimmed reads, retained reads.
- Defaults: forward-primer normalization with strict 5' anchoring and IUPAC-aware matching.
- Tools: cutadapt, seqkit.
- References: `domain/fastq/stages/normalize_primers.yaml`, `domain/fastq/tools/`.

### fastq.remove_chimeras {#fastq-remove-chimeras}
- Purpose: remove chimeric sequences in amplicon-oriented workflows.
- Inputs/Outputs: reads -> chimera_filtered_reads_r1/chimera_filtered_reads_r2, chimera_metrics_json, chimeras_fasta.
- Metrics: chimera counts, retained reads.
- Defaults: backend-owned chimera scoring with explicit filtered-read outputs.
- Tools: vsearch.
- References: `domain/fastq/stages/remove_chimeras.yaml`, `domain/fastq/tools/`.

### fastq.cluster_otus {#fastq-cluster-otus}
- Purpose: cluster reads into operational taxonomic units for amplicon workflows.
- Inputs/Outputs: reads -> otu_table, otu_representatives, taxonomy_ready_fasta, taxonomy_ready_fastq.
- Metrics: cluster counts, representative counts.
- Defaults: domain threshold defaults apply when no explicit OTU identity is bound.
- Tools: vsearch.
- References: `domain/fastq/stages/cluster_otus.yaml`, `domain/fastq/tools/`.

### fastq.infer_asvs {#fastq-infer-asvs}
- Purpose: infer amplicon sequence variants.
- Inputs/Outputs: reads -> asv_table_tsv, asv_sequences_fasta, taxonomy_ready_fasta, taxonomy_ready_fastq.
- Metrics: inferred variant counts.
- Defaults: no production backend admitted yet; stage remains a governed catalog-only entry.
- Tools: no admitted backend yet.
- References: `domain/fastq/stages/infer_asvs.yaml`, `domain/fastq/tools/`.

### fastq.normalize_abundance {#fastq-normalize-abundance}
- Purpose: normalize abundance summaries after amplicon inference.
- Inputs/Outputs: feature table -> normalized_abundance_tsv.
- Metrics: normalized abundance summaries.
- Defaults: relative-abundance normalization unless an explicit alternative method is bound.
- Tools: seqkit.
- References: `domain/fastq/stages/normalize_abundance.yaml`, `domain/fastq/tools/`.
