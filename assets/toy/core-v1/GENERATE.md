# GENERATE

## Command(s)
Generated via `cargo run -p bijux-dna-dev -- assets run refresh-toy`.

## Tool versions
- `bijux-dna-dev`, `cargo`, and `rustc` versions are recorded in `artifacts/assets-refresh/toy/report.json`.

## Input origins
- Synthetic deterministic toy records authored in `bijux-dna-dev` assets control-plane commands.

## Expected outputs
- `fastq/reads_1.fastq`
- `fastq/reads_2.fastq`
- `bam/toy.sam`
- `bam/qc_pre_core_metrics.sam`
- `bam/mapping_summary_partial_mapping.sam`
- `bam/filter_mixed_constraints.sam`
- `bam/mapq_threshold_ladder.sam`
- `bam/length_threshold_ladder.sam`
- `bam/validation_reference.fasta`
- `bam/validation_pass.bam`
- `bam/validation_pass.bam.bai`
- `bam/validation_malformed.bam`
- `tables/otu_abundance_small.tsv`
- `vcf/toy.vcf`
- `CHECKSUMS.sha256`
