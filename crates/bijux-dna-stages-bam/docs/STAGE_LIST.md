# STAGE_LIST

This list mirrors the `BamStage::all()` registry consumed by this crate. Keep it
in sync with `bijux-dna-domain-bam` when adding, renaming, or removing stages.

| Stage | Phase | Primary tools | Audit focus |
| --- | --- | --- | --- |
| `bam.align` | pre | bwa, bowtie2 | aligned BAM, index, flagstat, idxstats, samtools stats |
| `bam.validate` | pre | samtools | validation report, flagstat |
| `bam.qc_pre` | pre | samtools | flagstat, idxstats, samtools stats |
| `bam.mapping_summary` | pre | samtools | flagstat, idxstats, samtools stats, mapping summary |
| `bam.filter` | core | samtools, bamtools | filtered BAM, index, before/after flagstat and idxstats |
| `bam.mapq_filter` | core | samtools, bamtools | filtered BAM, index, before/after flagstat |
| `bam.length_filter` | core | samtools, picard | filtered BAM, index, length-filter summary |
| `bam.markdup` | core | picard | duplicate-marked BAM, index, before/after metrics |
| `bam.duplication_metrics` | core | samtools, picard | duplication report and histogram |
| `bam.complexity` | core | preseq | complexity report and preseq estimates |
| `bam.coverage` | core | mosdepth | coverage summary |
| `bam.insert_size` | core | picard | insert-size metrics and histogram |
| `bam.gc_bias` | core | picard | GC-bias metrics and plot |
| `bam.endogenous_content` | core | samtools | endogenous-content report |
| `bam.overlap_correction` | core | bamutil | overlap-corrected BAM and index |
| `bam.damage` | downstream | pydamage, mapdamage2 | DNA damage reports |
| `bam.authenticity` | downstream | authenticct | authenticity report |
| `bam.contamination` | downstream | angsd | contamination report |
| `bam.sex` | downstream | rxy | sex inference report |
| `bam.bias_mitigation` | downstream | mapdamage2 | bias report |
| `bam.recalibration` | downstream | gatk | recalibrated BAM, index, recalibration report |
| `bam.haplogroups` | downstream | yleaf | haplogroup report |
| `bam.genotyping` | downstream | angsd, bcftools | genotyping report |
| `bam.kinship` | downstream | king | kinship report |
