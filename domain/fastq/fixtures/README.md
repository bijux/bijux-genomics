# FASTQ Fixture Format

Each fixture file under domain/fastq/fixtures/STAGE_ID/*.txt must define:
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
- `fastq.build_contaminant_db`: intent = stage-specific command contract coverage for `fastq.build_contaminant_db`.
- `fastq.build_rrna_db`: intent = stage-specific command contract coverage for `fastq.build_rrna_db`.
- `fastq.build_taxonomy_db`: intent = stage-specific command contract coverage for `fastq.build_taxonomy_db`.
- `fastq.normalize_abundance`: intent = stage-specific command contract coverage for `fastq.normalize_abundance`.
- `fastq.infer_asvs`: intent = stage-specific command contract coverage for `fastq.infer_asvs`.
- `fastq.remove_chimeras`: intent = stage-specific command contract coverage for `fastq.remove_chimeras`.
- `fastq.deplete_reference_contaminants`: intent = stage-specific command contract coverage for `fastq.deplete_reference_contaminants`.
- `fastq.correct_errors`: intent = stage-specific command contract coverage for `fastq.correct_errors`.
- `fastq.remove_duplicates`: intent = stage-specific command contract coverage for `fastq.remove_duplicates`.
- `fastq.detect_adapters`: intent = stage-specific command contract coverage for `fastq.detect_adapters`.
- `fastq.filter_reads`: intent = stage-specific command contract coverage for `fastq.filter_reads`.
- `fastq.deplete_host`: intent = stage-specific command contract coverage for `fastq.deplete_host`.
- `fastq.profile_read_lengths`: intent = stage-specific command contract coverage for `fastq.profile_read_lengths`.
- `fastq.filter_low_complexity`: intent = stage-specific command contract coverage for `fastq.filter_low_complexity`.
- `fastq.merge_pairs`: intent = stage-specific command contract coverage for `fastq.merge_pairs`.
- `fastq.cluster_otus`: intent = stage-specific command contract coverage for `fastq.cluster_otus`.
- `fastq.profile_overrepresented_sequences`: intent = stage-specific command contract coverage for `fastq.profile_overrepresented_sequences`.
- `fastq.trim_polyg_tails`: intent = stage-specific command contract coverage for `fastq.trim_polyg_tails`.
- `fastq.index_reference`: intent = stage-specific command contract coverage for `fastq.index_reference`.
- `fastq.normalize_primers`: intent = stage-specific command contract coverage for `fastq.normalize_primers`.
- `fastq.prepare_adapter_bank`: intent = stage-specific command contract coverage for `fastq.prepare_adapter_bank`.
- `fastq.prepare_host_reference_bundle`: intent = stage-specific command contract coverage for `fastq.prepare_host_reference_bundle`.
- `fastq.prepare_primer_bank`: intent = stage-specific command contract coverage for `fastq.prepare_primer_bank`.
- `fastq.report_qc`: intent = stage-specific command contract coverage for `fastq.report_qc`.
- `fastq.deplete_rrna`: intent = stage-specific command contract coverage for `fastq.deplete_rrna`.
- `fastq.screen_taxonomy`: intent = stage-specific command contract coverage for `fastq.screen_taxonomy`.
- `fastq.profile_reads`: intent = stage-specific command contract coverage for `fastq.profile_reads`.
- `fastq.trim_reads`: intent = stage-specific command contract coverage for `fastq.trim_reads`.
- `fastq.extract_umis`: intent = stage-specific command contract coverage for `fastq.extract_umis`.
- `fastq.validate_reads`: intent = stage-specific command contract coverage for `fastq.validate_reads`.
- `fastq.verify_assets`: intent = stage-specific command contract coverage for `fastq.verify_assets`.

- `fastq.trim_terminal_damage`: intent = stage-specific command contract coverage for `fastq.trim_terminal_damage`.
