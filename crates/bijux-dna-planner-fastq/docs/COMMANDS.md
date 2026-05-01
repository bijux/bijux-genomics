# Commands

`bijux-dna-planner-fastq` does not expose Cargo binary targets, runtime CLI
commands, or process execution. It manages deterministic command specifications
inside planned `StagePlanV1` values and `ExecutionGraph` steps.

## Runtime Commands
None.

## Forbidden Command Surfaces
- No Cargo binary targets or `src/bin` command modules.
- No CLI parser ownership.
- No process spawning or runtime command execution.

## Managed Plan Command Specs
The crate can produce command specs for these FASTQ stage IDs:

- `fastq.index_reference`
- `fastq.validate_reads`
- `fastq.profile_read_lengths`
- `fastq.detect_adapters`
- `fastq.detect_duplicates_premerge`
- `fastq.estimate_library_complexity_prealign`
- `fastq.trim_terminal_damage`
- `fastq.normalize_primers`
- `fastq.trim_polyg_tails`
- `fastq.trim_reads`
- `fastq.filter_reads`
- `fastq.profile_reads`
- `fastq.deplete_rrna`
- `fastq.merge_pairs`
- `fastq.remove_duplicates`
- `fastq.filter_low_complexity`
- `fastq.deplete_host`
- `fastq.deplete_reference_contaminants`
- `fastq.correct_errors`
- `fastq.extract_umis`
- `fastq.profile_overrepresented_sequences`
- `fastq.remove_chimeras`
- `fastq.infer_asvs`
- `fastq.cluster_otus`
- `fastq.normalize_abundance`
- `fastq.screen_taxonomy`
- `fastq.report_qc`

The crate also owns planner-local graph step specs:

- `fastq.preprocess`
- `report.aggregate`
- `benchmark.compare_stage_tools`
- `benchmark.select_stage_tool`

## Source Of Truth
- FASTQ stage IDs come from `bijux_dna_domain_fastq::STAGES`.
- Stage command templates come from `src/tool_adapters/`.
- Synthetic planner step IDs come from `src/surface.rs` and `src/report_stage.rs`.
- This document is the crate-level command inventory for humans and tests.
