# FASTQ Default Settings (Pre-HPC)

This document defines aDNA-sane baseline defaults by stage.

- `fastq.prepare_reference`: prefer deterministic reference indexing (`star`).
- `fastq.detect_adapters`: detect with `fastp` before trimming decisions.
- `fastq.trim`: default `fastp`; keep deterministic ordering and stable report artifacts.
- `fastq.filter`: default `fastp`; preserve stage artifact contract.
- `fastq.validate_pre`: default `fastqvalidator` for strict validation.
- `fastq.stats_neutral`: default `seqkit_stats` for neutral descriptive summaries.
- `fastq.rrna`: default `sortmerna` when rRNA depletion is enabled.
- `fastq.qc_post`: default `multiqc` for operator-facing aggregation.
- `fastq.screen`: default `kraken2` for baseline taxonomic screening.
- `fastq.merge`: default `pear` for overlap merge baseline.
- `fastq.correct`: default `rcorrector` for conservative correction baseline.
- `fastq.umi`: default `umi_tools` for UMI-aware workflows.
