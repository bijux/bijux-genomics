# BAM Default Settings (Pre-HPC)

This document defines aDNA-sane baseline defaults by stage.

- `bam.align`: default `bwa` for broad compatibility.
- `bam.validate`: default `samtools` for deterministic health checks.
- `bam.filter`: default `samtools` for stable filtering behavior.
- `bam.coverage`: default `mosdepth` for efficient coverage summaries.
- `bam.damage`: default `mapdamage2` for historical comparability.
- `bam.authenticity`: default `authenticct` for authenticity summary baseline.
- `bam.contamination`: default `schmutzi` for mtDNA contamination baseline.
- `bam.sex`: default `rxy` for lightweight sex inference.
- `bam.kinship`: default `king` for pairwise kinship baseline.
