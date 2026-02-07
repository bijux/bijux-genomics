# STAGE_MAPPING

Stage → tool adapter → artifact contract.

- `bam.align`: `tool_adapters::stages_pre::align` → emits BAM + alignment metrics.
- `bam.validate`: `tool_adapters::stages_pre::validate` → emits validation report + metrics.
- `bam.qc_pre`: `tool_adapters::stages_pre::qc_pre` → emits QC metrics.
- `bam.filter`: `tool_adapters::stages_pre::filter` → emits filtered BAM + retention metrics.
- `bam.markdup`: `tool_adapters::stages_post::markdup` → emits dedup BAM + duplication metrics.
- `bam.complexity`: `tool_adapters::stages_post::complexity` → emits complexity metrics.
- `bam.coverage`: `tool_adapters::stages_post::coverage` → emits coverage metrics.
- `bam.damage`: `tool_adapters::stages_adna::damage` → emits damage metrics.
- `bam.authenticity`: `tool_adapters::stages_adna::authenticity` → emits authenticity metrics.
- `bam.contamination`: `tool_adapters::stages_adna::contamination` → emits contamination metrics.
- `bam.sex`: `tool_adapters::stages_adna::sex` → emits sex metrics.
- `bam.bias_mitigation`: `tool_adapters::stages_downstream::bias_mitigation` → emits bias metrics.
- `bam.recalibration`: `tool_adapters::stages_post::recalibration` → emits recalibration metrics.
- `bam.haplogroups`: `tool_adapters::stages_downstream::haplogroups` → emits haplogroup metrics.
- `bam.genotyping`: `tool_adapters::stages_downstream::genotyping` → emits genotype metrics.
- `bam.kinship`: `tool_adapters::stages_downstream::kinship` → emits kinship metrics.
