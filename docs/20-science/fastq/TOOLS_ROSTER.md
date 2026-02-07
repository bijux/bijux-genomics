# FASTQ Tools Roster

## What
Supported tools for each FASTQ stage.

## Why
Clarifies tool coverage and rationale.

## Non-goals
- Exhaustive tool survey.

## Contracts
- Tools listed must correspond to stage contracts.

## Examples
- fastp is used for adapter detection and trimming.

## Failure modes
- Unlisted tools in stage plans violate policy.

| Stage | Supported tools | Rationale |
| --- | --- | --- |
| fastq.validate_pre | fastqvalidator, seqkit | Format validation + counts |
| fastq.detect_adapters | fastp | Integrated adapter detection |
| fastq.trim | fastp, cutadapt, trimmomatic | Standard trimming tools |
| fastq.filter | seqkit, prinseq, fastp | Quality/length filtering |
| fastq.stats_neutral | seqkit_stats | Fast statistics |
| fastq.merge | pear, flash2, bbmerge, vsearch | Merge alternatives |
| fastq.correct | rcorrector, spades/bayeshammer, lighter, musket | Error correction options |
| fastq.umi | umi_tools | UMI handling |
| fastq.qc_post | multiqc | QC aggregation |
| fastq.screen | kraken2, centrifuge | Contamination screening |
