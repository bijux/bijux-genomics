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
| fastq.validate_reads | fastqvalidator, fqtools, seqtk | Structural validation + parser cross-checks before mutating stages |
| fastq.detect_adapters | fastp | Integrated adapter detection |
| fastq.trim_reads | fastp, cutadapt, trimmomatic | Standard trimming tools |
| fastq.filter_reads | seqkit, prinseq, fastp | Quality/length filtering |
| fastq.profile_reads | seqkit_stats | Fast statistics |
| fastq.merge_pairs | pear, flash2, bbmerge, vsearch | Merge alternatives |
| fastq.correct_errors | rcorrector, spades/bayeshammer, lighter, musket | Error correction options |
| fastq.extract_umis | umi_tools | UMI handling |
| fastq.report_qc | multiqc | QC aggregation |
| fastq.screen_taxonomy | kraken2, centrifuge | Contamination screening |
