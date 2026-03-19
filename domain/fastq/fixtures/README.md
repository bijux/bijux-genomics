# FASTQ Fixture Format

Each fixture file under `domain/fastq/fixtures/<stage>/*.txt` must define:
- `tool=<tool_id>`
- `tool_version=<pinned|semver|digest>`
- `stage=<domain.stage>`
- `domain=fastq`
- `fixture_kind=<truth|smoke|negative>`
- `command=<tool invocation entrypoint>`
- `args=<cli args or empty>`
- `expected_outputs=<artifact ids or token>`
- `expected_stdout_patterns=<token list or placeholder>`

## Fixture Directories
- `fastq.abundance_normalization`: intent = stage-specific command contract coverage for `fastq.abundance_normalization`.
- `fastq.asv_inference`: intent = stage-specific command contract coverage for `fastq.asv_inference`.
- `fastq.chimera_detection`: intent = stage-specific command contract coverage for `fastq.chimera_detection`.
- `fastq.contaminant_screen`: intent = stage-specific command contract coverage for `fastq.contaminant_screen`.
- `fastq.correct`: intent = stage-specific command contract coverage for `fastq.correct`.
- `fastq.deduplicate`: intent = stage-specific command contract coverage for `fastq.deduplicate`.
- `fastq.detect_adapters`: intent = stage-specific command contract coverage for `fastq.detect_adapters`.
- `fastq.filter_reads`: intent = stage-specific command contract coverage for `fastq.filter_reads`.
- `fastq.host_depletion`: intent = stage-specific command contract coverage for `fastq.host_depletion`.
- `fastq.profile_read_lengths`: intent = stage-specific command contract coverage for `fastq.profile_read_lengths`.
- `fastq.low_complexity`: intent = stage-specific command contract coverage for `fastq.low_complexity`.
- `fastq.merge`: intent = stage-specific command contract coverage for `fastq.merge`.
- `fastq.otu_clustering`: intent = stage-specific command contract coverage for `fastq.otu_clustering`.
- `fastq.profile_overrepresented_sequences`: intent = stage-specific command contract coverage for `fastq.profile_overrepresented_sequences`.
- `fastq.polyg_tailing`: intent = stage-specific command contract coverage for `fastq.polyg_tailing`.
- `fastq.prepare_reference`: intent = stage-specific command contract coverage for `fastq.prepare_reference`.
- `fastq.primer_normalization`: intent = stage-specific command contract coverage for `fastq.primer_normalization`.
- `fastq.report_qc`: intent = stage-specific command contract coverage for `fastq.report_qc`.
- `fastq.rrna`: intent = stage-specific command contract coverage for `fastq.rrna`.
- `fastq.screen_taxonomy`: intent = stage-specific command contract coverage for `fastq.screen_taxonomy`.
- `fastq.profile_reads`: intent = stage-specific command contract coverage for `fastq.profile_reads`.
- `fastq.trim_reads`: intent = stage-specific command contract coverage for `fastq.trim_reads`.
- `fastq.umi`: intent = stage-specific command contract coverage for `fastq.umi`.
- `fastq.validate_reads`: intent = stage-specific command contract coverage for `fastq.validate_reads`.

- `fastq.damage_aware_pretrim`: intent = stage-specific command contract coverage for `fastq.damage_aware_pretrim`.
