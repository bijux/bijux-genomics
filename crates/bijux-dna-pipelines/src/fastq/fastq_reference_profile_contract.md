# FASTQ Reference-Grade aDNA Profile Contract

Profile: `fastq-to-fastq__reference_adna__v1`

Guarantees:
- Required stages: `fastq.validate_pre`, `fastq.detect_adapters`, `fastq.trim`, `fastq.low_complexity`, `fastq.merge`, `fastq.filter`, `fastq.stats_neutral`, `fastq.qc_post`.
- aDNA trimming invariants: `trim.min_len > 0`, `trim.adapter_policy != none`, quality trimming enabled, poly-X trimming enabled.
- Pairing/library declaration: preprocess params must declare paired library mode; paired libraries require merge unless explicitly disabled.
- Optional contamination screen hook: if `fastq.screen` is enabled, `contaminant_db` must be declared.

Metrics expectations:
- `fastq.stats_neutral` includes read-length and GC distributions.
- `fastq.detect_adapters` and `fastq.qc_post` include overrepresented-sequence counts derived from FastQC data.
- `fastq.low_complexity` is used as a pre-alignment complexity/duplication proxy estimate stage.
