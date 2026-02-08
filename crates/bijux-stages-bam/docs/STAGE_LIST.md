# STAGE_LIST

| Stage | Phase | Class | Inputs | Outputs | Metrics |
| --- | --- | --- | --- | --- | --- |
| bam.align | pre | Essential | FASTQ | BAM | alignment_rate |
| bam.sort | core | Essential | BAM | sorted BAM | sort_time |
| bam.index | core | Essential | BAM | BAI | index_stats |
| bam.markdup | core | Recommended | BAM | dedup BAM | dup_rate |
| bam.damage | downstream | Optional | BAM | report.json | damage_profile |
| bam.contamination | downstream | Optional | BAM | report.json | contamination_rate |
| bam.authenticity | downstream | Optional | BAM | report.json | authenticity_score |
