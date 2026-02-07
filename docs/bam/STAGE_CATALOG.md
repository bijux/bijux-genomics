# BAM Stage Catalog

| Stage | Criticality | Artifact expectations | Metrics expectations |
| --- | --- | --- | --- |
| bam.align | Essential | reads → align.bam/bai | alignment rate, MAPQ |
| bam.validate | Essential | bam → validation report | format/flag checks |
| bam.qc_pre | Essential | bam → metrics_json | pre‑QC summaries |
| bam.filter | Essential | bam → filtered.bam/bai | filtering reasons |
| bam.markdup | Essential | bam → markdup.bam/bai | duplicate rate |
| bam.complexity | Recommended | bam → complexity metrics | library complexity |
| bam.coverage | Recommended | bam → coverage summary | depth, breadth |
| bam.damage | Recommended | bam → damage metrics | misincorporation patterns |
| bam.authenticity | Recommended | bam → authenticity metrics | aDNA authenticity |
| bam.contamination | Recommended | bam → contamination metrics | contamination estimates |
| bam.sex | Optional | bam → sex metrics | sex inference |
| bam.bias_mitigation | Optional | bam → bias metrics | GC/length bias |
| bam.recalibration | Optional | bam → recal.bam/bai | BQSR metrics |
| bam.haplogroups | Optional | bam → haplogroup metrics | lineage calls |
| bam.genotyping | Optional | bam → genotyping metrics | variant summaries |
| bam.kinship | Optional | bam → kinship metrics | relatedness |
