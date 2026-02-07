# BAM Tools Roster

## What
Supported tools for each BAM stage.

## Why
Clarifies tool coverage and rationale.

## Non-goals
- Exhaustive tool survey.

## Contracts
- Tools listed must correspond to stage contracts.

## Examples
- bwa for alignment; picard for markdup.

## Failure modes
- Unlisted tools in stage plans violate policy.

| Stage | Supported tools | Rationale |
| --- | --- | --- |
| bam.align | bwa, bowtie2 | Standard aligners |
| bam.validate | samtools | BAM integrity checks |
| bam.markdup | picard | Duplicate marking |
| bam.recalibration | GATK | BQSR |
| bam.coverage | mosdepth | Coverage summaries |
| bam.damage | mapDamage2, pyDamage | Damage profiling |
| bam.complexity | preseq | Complexity estimation |
| bam.authenticity | authenticCT | Authenticity metrics |
| bam.contamination | ANGSD | Contamination estimates |
| bam.sex | RXY | Sex inference |
| bam.haplogroups | Yleaf | Haplogroup inference |
| bam.kinship | KING | Relatedness |
