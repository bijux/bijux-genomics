# BAM Stage Scientific Assumptions

## What
Stage-level scientific assumptions for the governed BAM stage catalog.

## Why
These assumptions define when BAM-stage outputs are scientifically interpretable
instead of merely syntactically valid.

## Non-goals
- Replacing the lower-level stage manifests under [domain/bam/stages/](../../../domain/bam/stages/).
- Repeating tool-specific failure modes that belong in [TOOLS_ROSTER.md](TOOLS_ROSTER.md).

## Contracts
- Every `status: "supported"` BAM stage must appear exactly once below.
- Planned BAM stages stay out of this list until they are promoted to supported.
- The authoritative source for per-stage assumption payloads remains
  [domain/bam/stages/](../../../domain/bam/stages/) (`assumptions` field).

## Stage assumptions
- `bam.align`: read-to-reference mapping is meaningful under selected aligner parameters.
- `bam.authenticity`: authenticity proxies are interpretable only when damage-sensitive molecule context is preserved.
- `bam.validate`: structural BAM validity is a prerequisite for scientific interpretation.
- `bam.mapping_summary`: mapping summaries are only comparable when contig naming and indexing state are consistent.
- `bam.filter`: filtering criteria preserve authentic signal while reducing noise.
- `bam.mapq_filter`: MAPQ thresholds must match the study design and must not silently erase low-quality authentic reads.
- `bam.length_filter`: minimum-length thresholds must reflect expected fragment-size biology for the library class.
- `bam.duplication_metrics`: duplicate summaries assume coordinate-consistent BAM plus valid duplicate interpretation policy.
- `bam.endogenous_content`: endogenous-content estimates assume host/non-host contig partitions are declared correctly.
- `bam.coverage`: depth and breadth summaries reflect biological and technical sampling limits only when the reference target space is fixed.
- `bam.damage`: deamination/misincorporation signatures are interpretable for authenticity context.
- `bam.contamination`: contamination-model inputs, references, and mixture assumptions must be appropriate for the cohort.
- `bam.sex`: sex inference assumptions require sufficient chrX/chrY informative coverage.
- `bam.kinship`: kinship inference assumes adequate marker overlap and compatible panel or allele-frequency assumptions.

## Stage notes
- Supported BAM stages are the only stages that may currently drive governed
  scientific interpretations in the pre-HPC surface.
- Planned stages such as `bam.qc_pre`, `bam.markdup`, `bam.gc_bias`, `bam.insert_size`,
  `bam.overlap_correction`, `bam.bias_mitigation`, `bam.recalibration`,
  `bam.haplogroups`, `bam.genotyping`, and `bam.complexity` stay documented in
  [STAGE_CATALOG.md](STAGE_CATALOG.md) and [TOOLS_ROSTER.md](TOOLS_ROSTER.md), but they do not belong in this
  supported-stage assumption ledger yet.
