# BAM Stage Scientific Assumptions

This document maps BAM stage-level scientific assumptions used in the pre-HPC scope.
Source of truth remains `domain/bam/stages/*.yaml` (`assumptions` field).

## Stage assumptions
- `bam.align`: read-to-reference mapping is meaningful under selected aligner parameters.
- `bam.validate`: structural BAM validity is a prerequisite for scientific interpretation.
- `bam.filter`: filtering criteria preserve authentic signal while reducing noise.
- `bam.damage`: deamination/misincorporation signatures are interpretable for authenticity context.
- `bam.authenticity`: authenticity proxies (e.g., PMD/damage) reflect endogenous molecule behavior.
- `bam.contamination`: contamination model inputs (mt/panel/reference) are appropriate.
- `bam.coverage`: depth/breadth statistics reflect biological and technical sampling limits.
- `bam.sex`: sex inference assumptions require sufficient chrX/chrY informative coverage.
- `bam.kinship`: kinship inference assumes adequate marker overlap and compatible panel assumptions.

## Planned-stage note
Stages marked `planned` in domain are excluded from active generated pre-HPC configs.
