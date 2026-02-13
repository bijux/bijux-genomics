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
- `fastq.prepare_reference`: default `star`.
- `fastq.validate_pre`: default `fastqvalidator`.
- `fastq.length_distribution_pre`: default `seqkit_stats`.
- `fastq.detect_adapters`: default `fastp`.
- `fastq.polyg_tailing`: default `fastp`.
- `fastq.trim`: default `fastp`.
- `fastq.filter`: default `fastp`.
- `fastq.stats_neutral`: default `seqkit_stats`.
- `fastq.rrna`: default `sortmerna`.
- `fastq.qc_post`: default `multiqc`.
- `fastq.merge`: default `pear`.
- `fastq.deduplicate`: default `prinseq`.
- `fastq.low_complexity`: default `bbduk`.
- `fastq.host_depletion`: default `bowtie2`.
- `fastq.contaminant_screen`: default `bbduk`.
- `fastq.correct`: default `rcorrector`.
- `fastq.umi`: default `umi_tools`.
- `fastq.overrepresented_sequences`: default `fastqc`.
- `fastq.screen`: default `kraken2`.
- `fastq.primer_normalization`: default `cutadapt`. rationale: deterministic primer trimming with explicit mismatch/orientation controls.
- `fastq.chimera_detection`: default `vsearch`. rationale: deterministic uchime-based baseline before broader ensemble adoption.
- `fastq.otu_clustering`: default `vsearch`. rationale: stable OTU cluster policy with reproducible identifiers.
- `fastq.asv_inference`: default `vsearch` (placeholder). rationale: ASV engine remains external/experimental; placeholder preserves deterministic stage contract.
- `fastq.abundance_normalization`: default `seqkit_stats` (placeholder). rationale: deterministic normalization reporting baseline for compositional warnings.

single_tool_justification: fastq.prepare_reference
single_tool_justification: fastq.detect_adapters
single_tool_justification: fastq.rrna
single_tool_justification: fastq.umi
single_tool_justification: fastq.primer_normalization
single_tool_justification: fastq.chimera_detection
single_tool_justification: fastq.otu_clustering
single_tool_justification: fastq.asv_inference
single_tool_justification: fastq.abundance_normalization
