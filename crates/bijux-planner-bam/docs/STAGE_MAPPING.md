# STAGE_MAPPING

Authority for stage catalog lives in `src/stages/stage_catalog.rs`.

| Stage ID | Phase | Tool Adapter | Artifacts | Metrics |
| --- | --- | --- | --- | --- |
| bam.align | pre | bwa | BAM | alignment_rate |
| bam.sort | core | samtools | sorted BAM | sort_time |
| bam.index | core | samtools | BAI | index_stats |
| bam.markdup | core | picard | dedup BAM | dup_rate |
| bam.damage | downstream | mapDamage2 | report.json | damage_profile |
| bam.contamination | downstream | pydamage | report.json | contamination_rate |
| bam.authenticity | downstream | pydamage | report.json | authenticity_score |
