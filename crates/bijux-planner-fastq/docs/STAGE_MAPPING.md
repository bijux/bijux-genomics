# STAGE_MAPPING

Stage → tool adapter → artifact contract.

- `fastq.validate_pre`: `tool_adapters::stages::pre::validate_pre` → emits validation report (ReportJson).
- `fastq.detect_adapters`: `tool_adapters::stages::pre::detect_adapters` → emits adapter detection metrics.
- `fastq.trim`: `tool_adapters::stages::transform::trim` → emits trimmed reads + retention metrics.
- `fastq.filter`: `tool_adapters::stages::transform::filter` → emits filtered reads + retention metrics.
- `fastq.merge`: `tool_adapters::stages::transform::merge` → emits merged/unmerged reads + merge metrics.
- `fastq.correct`: `tool_adapters::stages::transform::correct` → emits corrected reads + correction metrics.
- `fastq.umi`: `tool_adapters::stages::transform::umi` → emits UMI-processed reads + UMI metrics.
- `fastq.stats_neutral`: `tool_adapters::stages::qc::stats_neutral` → emits stats metrics.
- `fastq.qc_post`: `tool_adapters::stages::qc::qc_post` → emits QC report bundle.
- `fastq.screen`: `tool_adapters::stages::qc::screen` → emits screen report + metrics.
- `fastq.preprocess`: `tool_adapters::stages::pre::preprocess` → emits preprocessing artifacts.
