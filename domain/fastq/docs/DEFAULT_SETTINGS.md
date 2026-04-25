# FASTQ Default Settings (Pre-HPC)

Purpose: define deterministic defaults for every FASTQ stage contract.

## Profile Semantics
- `fastq-to-fastq__default__v1` is the generic FASTQ baseline. It does not require `fastq.trim_terminal_damage` and does not carry terminal-damage tool or parameter defaults.
- `fastq-to-fastq__minimal__v1` is the reduced generic baseline. It keeps only validation, adapter detection, trimming, filtering, and QC aggregation as required stages.
- aDNA-oriented FASTQ profiles opt in to `fastq.trim_terminal_damage` explicitly because terminal damage trimming is a library-specific decision, not an unknown-assay default.
- UMI extraction is ordered immediately after read validation when requested so inline UMIs are captured before adapter, polyG, quality, or length trimming can alter the barcode-bearing sequence.

## Inputs
- FASTQ read pairs or single-end reads, plus optional reference/decoy indexes by stage.

## Outputs
- stage-specific FASTQ/BAM/JSON artifacts declared in stage contracts.

## Key Parameters
- read layout (SE/PE), quality thresholds, adapter/polyg settings, classifier presets.

## Validity Limits
- only pinned tool versions are valid
- contract-required inputs/outputs must be preserved
- stage/tool combinations must remain in index compatibility map

## Stage Coverage
- `fastq.index_reference`: default `bowtie2_build`. rationale: default FASTQ reference preparation must emit the mapper index consumed by governed host and reference-contaminant depletion.
- `fastq.validate_reads`: default `fastqvalidator`.
- `fastq.profile_read_lengths`: default `seqkit_stats`.
- `fastq.detect_adapters`: default `fastqc`.
- `fastq.trim_polyg_tails`: default `fastp`.
- `fastq.trim_reads`: default `fastp`.
- `fastq.filter_reads`: default `fastp`.
- `fastq.profile_reads`: default `seqkit_stats`.
- `fastq.deplete_rrna`: default `sortmerna`.
- `fastq.report_qc`: default `multiqc`.
- `fastq.merge_pairs`: default `pear`.
- `fastq.remove_duplicates`: default `clumpify`. rationale: the governed default must remain runnable for both single-end and paired-end inputs while still supporting explicit optical-aware duplicate policy.
- `fastq.filter_low_complexity`: default `bbduk`.
- `fastq.deplete_host`: default `bowtie2`.
- `fastq.deplete_reference_contaminants`: default `bowtie2`. rationale: reference-guided decoy depletion stays aligned with the current stage contract.
- `fastq.correct_errors`: default `rcorrector`.
- `fastq.extract_umis`: default `umi_tools`.
- `fastq.profile_overrepresented_sequences`: default `fastqc`.
- `fastq.screen_taxonomy`: default `kraken2`.
- `fastq.trim_terminal_damage`: default `cutadapt`. rationale: deterministic terminal mask/trim policy for aDNA damage-aware pretrim.
- `fastq.normalize_primers`: default `cutadapt`. rationale: deterministic primer trimming with explicit mismatch/orientation controls.
- `fastq.remove_chimeras`: default `vsearch`. rationale: deterministic uchime-based baseline before broader ensemble adoption.
- `fastq.cluster_otus`: default `vsearch`. rationale: stable OTU cluster policy with reproducible identifiers.
- `fastq.infer_asvs`: default `dada2`. rationale: governed ASV inference now uses the admitted containerized dada2 backend and publishes the canonical infer-asvs report plus taxonomy-ready representative sequences.
- `fastq.normalize_abundance`: default `seqkit`. rationale: abundance-table normalization stays within the currently admitted amplicon table tooling.

validation_benchmark_policy: fastq.validate_reads
- default benchmark backend is `fastqvalidator`
- `fastqc`, `fastq_scan`, `fqtools`, and `seqtk` are comparison backends for governed-report agreement studies
- `fastq.report_qc` and `fastq.profile_reads` are downstream complements, not substitutes for structural validation

single_tool_justification: fastq.index_reference
single_tool_justification: fastq.detect_adapters
single_tool_justification: fastq.deplete_rrna
single_tool_justification: fastq.extract_umis
single_tool_justification: fastq.normalize_primers
single_tool_justification: fastq.remove_chimeras
single_tool_justification: fastq.cluster_otus
single_tool_justification: fastq.normalize_abundance

single_tool_justification: fastq.trim_terminal_damage
