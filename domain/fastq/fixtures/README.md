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
- `fastq.deplete_reference_contaminants`: intent = stage-specific command contract coverage for `fastq.deplete_reference_contaminants`.
- `fastq.correct_errors`: intent = stage-specific command contract coverage for `fastq.correct_errors`.
- `fastq.remove_duplicates`: intent = stage-specific command contract coverage for `fastq.remove_duplicates`.
- `fastq.detect_adapters`: intent = stage-specific command contract coverage for `fastq.detect_adapters`.
- `fastq.filter_reads`: intent = stage-specific command contract coverage for `fastq.filter_reads`.
- `fastq.deplete_host`: intent = stage-specific command contract coverage for `fastq.deplete_host`.
- `fastq.profile_read_lengths`: intent = stage-specific command contract coverage for `fastq.profile_read_lengths`.
- `fastq.filter_low_complexity`: intent = stage-specific command contract coverage for `fastq.filter_low_complexity`.
- `fastq.merge_pairs`: intent = stage-specific command contract coverage for `fastq.merge_pairs`.
- `fastq.otu_clustering`: intent = stage-specific command contract coverage for `fastq.otu_clustering`.
- `fastq.profile_overrepresented_sequences`: intent = stage-specific command contract coverage for `fastq.profile_overrepresented_sequences`.
- `fastq.trim_polyg_tails`: intent = stage-specific command contract coverage for `fastq.trim_polyg_tails`.
- `fastq.prepare_reference`: intent = stage-specific command contract coverage for `fastq.prepare_reference`.
- `fastq.primer_normalization`: intent = stage-specific command contract coverage for `fastq.primer_normalization`.
- `fastq.report_qc`: intent = stage-specific command contract coverage for `fastq.report_qc`.
- `fastq.deplete_rrna`: intent = stage-specific command contract coverage for `fastq.deplete_rrna`.
- `fastq.screen_taxonomy`: intent = stage-specific command contract coverage for `fastq.screen_taxonomy`.
- `fastq.profile_reads`: intent = stage-specific command contract coverage for `fastq.profile_reads`.
- `fastq.trim_reads`: intent = stage-specific command contract coverage for `fastq.trim_reads`.
- `fastq.extract_umis`: intent = stage-specific command contract coverage for `fastq.extract_umis`.
- `fastq.validate_reads`: intent = stage-specific command contract coverage for `fastq.validate_reads`.

- `fastq.trim_terminal_damage`: intent = stage-specific command contract coverage for `fastq.trim_terminal_damage`.
