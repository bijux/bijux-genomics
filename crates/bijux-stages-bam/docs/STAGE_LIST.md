# STAGE_LIST

One-line intent per BAM stage with required artifacts/metrics.

- `bam.align`: Align reads to reference; emits BAM + alignment metrics.
- `bam.validate`: Validate BAM format/content; emits validation metrics.
- `bam.qc_pre`: Pre-QC reporting; emits QC metrics.
- `bam.filter`: Filter alignments by quality; emits filtered BAM + retention metrics.
- `bam.markdup`: Mark duplicates; emits deduplicated BAM + duplication metrics.
- `bam.complexity`: Library complexity analysis; emits complexity metrics.
- `bam.coverage`: Coverage summary; emits coverage metrics.
- `bam.damage`: Damage profiling; emits damage metrics.
- `bam.authenticity`: Authenticity assessment; emits authenticity metrics.
- `bam.contamination`: Contamination estimation; emits contamination metrics.
- `bam.sex`: Sex inference; emits sex metrics.
- `bam.bias_mitigation`: Bias mitigation; emits bias metrics.
- `bam.recalibration`: Base quality recalibration; emits recalibrated BAM + metrics.
- `bam.haplogroups`: Haplogroup inference; emits haplogroup metrics.
- `bam.genotyping`: Genotyping; emits genotype metrics.
- `bam.kinship`: Kinship estimation; emits kinship metrics.
