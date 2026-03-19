# FASTQ Default Settings (Pre-HPC)

Purpose: define deterministic defaults for every FASTQ stage contract.

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
- `fastq.index_reference`: default `star`.
- `fastq.validate_reads`: default `fastqvalidator`.
- `fastq.profile_read_lengths`: default `seqkit_stats`.
- `fastq.detect_adapters`: default `fastp`.
- `fastq.trim_polyg_tails`: default `fastp`.
- `fastq.trim_reads`: default `fastp`.
- `fastq.filter_reads`: default `fastp`.
- `fastq.profile_reads`: default `seqkit_stats`.
- `fastq.deplete_rrna`: default `sortmerna`.
- `fastq.report_qc`: default `multiqc`.
- `fastq.merge_pairs`: default `pear`.
- `fastq.remove_duplicates`: default `prinseq`.
- `fastq.filter_low_complexity`: default `bbduk`.
- `fastq.deplete_host`: default `bowtie2`.
- `fastq.deplete_reference_contaminants`: default `bbduk`. rationale: deterministic k-mer depletion against configured decoy references.
- `fastq.correct_errors`: default `rcorrector`.
- `fastq.extract_umis`: default `umi_tools`.
- `fastq.profile_overrepresented_sequences`: default `fastqc`.
- `fastq.screen_taxonomy`: default `kraken2`.
- `fastq.trim_terminal_damage`: default `cutadapt`. rationale: deterministic terminal mask/trim policy for aDNA damage-aware pretrim.
- `fastq.normalize_primers`: default `cutadapt`. rationale: deterministic primer trimming with explicit mismatch/orientation controls.
- `fastq.remove_chimeras`: default `vsearch`. rationale: deterministic uchime-based baseline before broader ensemble adoption.
- `fastq.cluster_otus`: default `vsearch`. rationale: stable OTU cluster policy with reproducible identifiers.
- `fastq.infer_asvs`: default `vsearch` (placeholder). rationale: ASV engine remains external/experimental; placeholder preserves deterministic stage contract.
- `fastq.normalize_abundance`: default `seqkit_stats` (placeholder). rationale: deterministic normalization reporting baseline for compositional warnings.

single_tool_justification: fastq.index_reference
single_tool_justification: fastq.detect_adapters
single_tool_justification: fastq.deplete_rrna
single_tool_justification: fastq.extract_umis
single_tool_justification: fastq.normalize_primers
single_tool_justification: fastq.remove_chimeras
single_tool_justification: fastq.cluster_otus
single_tool_justification: fastq.infer_asvs
single_tool_justification: fastq.normalize_abundance

single_tool_justification: fastq.trim_terminal_damage
