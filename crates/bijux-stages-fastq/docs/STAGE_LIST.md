# STAGE_LIST

One-line intent per FASTQ stage with required artifacts/metrics.

- `fastq.validate_pre`: Validate raw FASTQ input; requires input reads and emits validation metrics.
- `fastq.detect_adapters`: Detect adapter content; consumes reads and emits adapter detection metrics.
- `fastq.trim`: Trim adapters/low-quality bases; consumes reads and produces trimmed reads with retention metrics.
- `fastq.filter`: Filter reads by quality/complexity; consumes reads and produces filtered reads with retention metrics.
- `fastq.merge`: Merge paired reads; consumes paired reads and emits merged/unmerged reads with merge metrics.
- `fastq.correct`: Error-correct reads; consumes reads and emits corrected reads with correction metrics.
- `fastq.preprocess`: Preprocess reads for downstream steps; consumes reads and emits normalized reads + preprocessing metrics.
- `fastq.qc_post`: Post-QC reporting; consumes reads and emits QC metrics/report.
- `fastq.rrna`: rRNA screening; consumes reads and emits rRNA screening metrics.
- `fastq.screen`: Contaminant screening; consumes reads and emits screen metrics/report.
- `fastq.stats_neutral`: Neutral stats summary; consumes reads and emits stats metrics.
- `fastq.umi`: UMI handling; consumes reads and emits UMI-processed reads + metrics.
