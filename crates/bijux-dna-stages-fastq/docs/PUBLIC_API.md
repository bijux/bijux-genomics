# Public API

The crate root exports a narrow FASTQ stage surface from `src/lib.rs`. Internal
plugin and runtime modules remain private.

## Public Modules

- `contracts`
- `metrics`
- `observer`
- `stage_specs`

## Stable Root Exports

- `FastqStagePlugin`
- `StagePlanJson`
- `RuntimeInterpretationLevel`
- `contract_stage_ids`
- `closed_execution_stage_ids`
- `implemented_stages`
- `observer_specialized_stage_ids`
- `observer_stage_ids`
- `observer_stage_tool_bindings`
- `runtime_interpretation_for_stage`
- `runtime_interpretation_for_stage_tool`
- `runtime_interpretation_stage_ids`

## Observer Parser Exports

The `observer` module re-exports the governed FASTQ parser surface used by
stage plugins and command/API crates:

- backend metrics: `parse_adapterremoval_metrics`, `parse_fastp_metrics`,
  `parse_fastqc_summary_metrics`, `parse_multiqc_general_stats_metrics`,
  `parse_samtools_flagstat_metrics`, `parse_seqkit_tool_metrics`
- read statistics: `parse_fastqvalidator_count`, `parse_seqkit_stats`,
  `parse_length_histogram`
- governed FASTQ reports: `parse_trim_reads_report`,
  `parse_filter_reads_report`, `parse_filter_low_complexity_report`,
  `parse_merge_pairs_report`, `parse_deduplicate_report`,
  `parse_remove_duplicates_report`, `parse_report_qc_report`,
  `parse_screen_taxonomy_report`, and related stage report parsers

## Compatibility Rules

- Removing or renaming a public export is breaking.
- Changing observer JSON, stage contract snapshots, metrics envelopes, or
  runtime-interpretation classification is breaking unless versioned explicitly.
- Adding a parser, metric field, or stage contract requires matching fixture,
  snapshot, registry, and docs coverage.
- New callable operations must be listed in `docs/COMMANDS.md`.

## Plugin Preconditions

- `FastqStagePlugin::handles_stage` and `FastqStagePlugin::materialize` accept
  only registered FASTQ stage IDs.
- `FastqStagePlugin::materialize` requires the planner-provided command template
  to contain at least one nonblank argument and rejects blank arguments.
- `FastqStagePlugin::materialize` does not choose tools, construct shell
  commands, resolve environments, or execute processes.
- `FastqStagePlugin::parse_outputs` parses existing artifacts into the output
  envelope and preserves observer artifact writes as an explicit observer
  operation.

## Internal Modules

- `plugin` validates FASTQ stage support, materializes planned invocations, and
  builds plugin output envelopes from existing artifacts.
- `runtime` classifies stage and stage-tool interpretation levels.
- `surface` keeps crate-root registry functions out of `lib.rs`.
