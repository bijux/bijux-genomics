# BAM Default Settings (Pre-HPC)

Purpose: define deterministic defaults for every BAM stage contract.

## Inputs
- aligned or partially processed BAM, reference assets, optional metadata inputs by stage.

## Outputs
- BAM transforms plus stage metrics/reports defined in stage contracts.

## Key Parameters
- MAPQ/length thresholds, duplicate policy, contamination/authenticity model toggles.

## Validity Limits
- only pinned tool versions are valid
- required stage inputs/outputs must be preserved
- compatibility map in index.yaml is authoritative

## Stage Coverage
- `bam.align`: default `bwa`.
- `bam.validate`: default `samtools`.
- `bam.qc_pre`: default `samtools`.
- `bam.mapping_summary`: default `samtools`.
- `bam.filter`: default `samtools`.
- `bam.mapq_filter`: default `samtools`.
- `bam.length_filter`: default `samtools`.
- `bam.markdup`: default `samtools`.
- `bam.duplication_metrics`: default `samtools`.
- `bam.complexity`: default `preseq`.
- `bam.coverage`: default `mosdepth`.
- `bam.insert_size`: default `picard`.
- `bam.gc_bias`: default `picard`.
- `bam.endogenous_content`: default `samtools`.
- `bam.overlap_correction`: default `bamutil`.
- `bam.damage`: default `mapdamage2`.
- `bam.authenticity`: default `authenticct`.
- `bam.contamination`: default `schmutzi`.
- `bam.sex`: default `rxy`.
- `bam.bias_mitigation`: default `samtools`.
- `bam.recalibration`: default `gatk`.
- `bam.haplogroups`: default `yleaf`.
- `bam.genotyping`: default `gatk`.
- `bam.kinship`: default `king`.

single_tool_justification: bam.qc_pre
single_tool_justification: bam.mapping_summary
single_tool_justification: bam.complexity
single_tool_justification: bam.insert_size
single_tool_justification: bam.gc_bias
single_tool_justification: bam.endogenous_content
single_tool_justification: bam.overlap_correction
single_tool_justification: bam.bias_mitigation
single_tool_justification: bam.recalibration
single_tool_justification: bam.haplogroups
