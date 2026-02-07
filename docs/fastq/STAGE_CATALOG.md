# FASTQ Stage Catalog

| Stage | Criticality | Artifact expectations | Metrics expectations |
| --- | --- | --- | --- |
| fastq.validate_pre | Essential | reads → validation report | read counts, format checks |
| fastq.detect_adapters | Essential | reads → adapter summary | adapter composition, read loss |
| fastq.trim | Essential | reads → trimmed_reads | base trimming stats, retention |
| fastq.filter | Essential | reads → filtered_reads | read loss reasons, retention |
| fastq.stats_neutral | Essential | reads → metrics_json | base/read length summary |
| fastq.merge | Recommended | paired → merged | merge rate, overlap stats |
| fastq.correct | Recommended | reads → corrected | correction rate, error profile |
| fastq.umi | Optional | reads → umi_reads | umi grouping, consensus stats |
| fastq.qc_post | Optional | reads → report_html | QC summary artifacts |
| fastq.screen | Optional | reads → screen report | contamination classification |
| fastq.preprocess | Optional | composite | pipeline‑level metrics |
