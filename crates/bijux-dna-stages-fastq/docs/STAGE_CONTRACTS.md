# STAGE_CONTRACTS

## Symmetry

Symmetry is enforced at the contract level (observable inputs/outputs), not file naming.

## Coverage surfaces

`contract_stage_ids()` covers the published FASTQ stage contracts.
`implemented_stages()` covers the closed execution subset implemented for governed FASTQ stages in
`bijux-dna-stages-fastq`.
`closed_execution_stage_ids()` exposes the broader closed execution subset owned by the FASTQ
domain.
`observer_specialized_stage_ids()` is the narrower fully observer-specialized subset documented in
`OBSERVERS.md`.
`observer_stage_ids()` remains a compatibility alias for that observer-specialized subset.

## Registry completeness

`tests/contracts/registry_completeness.rs` ensures every domain stage appears in the stage registry.
When adding a stage, update the registry and this document.

## Stage Registry

Legend:

- Essential: required for default pipelines.
- Recommended: optional but commonly used.
- Optional: only when explicitly requested or dictated by sample type.

| Stage | Class | Inputs | Outputs | Metrics |
| --- | --- | --- | --- | --- |
| `fastq.validate_reads` | Essential | FASTQ | validation report | read count, base count, format errors |
| `fastq.detect_adapters` | Recommended | FASTQ | adapter evidence report | evidence-only adapter inspection summary |
| `fastq.trim_polyg_tails` | Recommended | FASTQ | trimmed FASTQ | governed polyG-tail trimming semantics and backend report provenance |
| `fastq.trim_reads` | Essential | FASTQ | trimmed FASTQ | retention and bases kept |
| `fastq.filter_reads` | Recommended | FASTQ | filtered FASTQ | filter counts |
| `fastq.filter_low_complexity` | Optional | FASTQ | filtered FASTQ | reads removed for low complexity |
| `fastq.merge_pairs` | Optional | paired FASTQ | merged FASTQ | merge rate |
| `fastq.remove_duplicates` | Optional | FASTQ | deduplicated FASTQ and duplicate evidence | dedup rate, duplicate classes, backend lineage |
| `fastq.deplete_host` | Optional | FASTQ and host reference index | host-depleted FASTQ and canonical host depletion report | host fraction removed, reads removed, reference lineage |
| `fastq.deplete_rrna` | Optional | FASTQ | rRNA-filtered FASTQ | rRNA fraction |
| `fastq.correct_errors` | Optional | single-end or paired FASTQ | corrected FASTQ and governed correction report | corrected reads, kmer fix rate, correction engine, executable parameter lineage |
| `fastq.extract_umis` | Optional | paired FASTQ | UMI-tagged FASTQ | UMI statistics |
| `fastq.screen_taxonomy` | Optional | FASTQ | governed taxonomy report and raw screen summary | contamination rate, classified fraction, top taxa, database lineage |
| `fastq.profile_reads` | Optional | FASTQ | stats report | read count, base count |
| `fastq.profile_read_lengths` | Optional | FASTQ | length report | length histogram |
| `fastq.profile_overrepresented_sequences` | Optional | FASTQ | sequence report | flagged sequence counts |
| `fastq.report_qc` | Optional | upstream QC reports | governed aggregation report and MultiQC bundle | contributor lineage, aggregation scope, QC summary |

## Declared Contract Stages

The FASTQ domain also publishes declared contract stages that may be
planner-facing, preparation-facing, or generic-envelope only. They remain part
of `contract_stage_ids()` even when this crate has narrower observer
specialization for a subset.

- `fastq.build_contaminant_db`
- `fastq.build_rrna_db`
- `fastq.build_taxonomy_db`
- `fastq.capture_provenance_snapshot`
- `fastq.classify_layout`
- `fastq.cluster_otus`
- `fastq.concatenate_lanes`
- `fastq.deplete_reference_contaminants`
- `fastq.deinterleave_reads`
- `fastq.demultiplex_reads`
- `fastq.detect_duplicates_premerge`
- `fastq.detect_instrument_artifacts`
- `fastq.estimate_library_complexity_prealign`
- `fastq.index_reference`
- `fastq.infer_asvs`
- `fastq.interleave_reads`
- `fastq.materialize_qc_manifest`
- `fastq.normalize_abundance`
- `fastq.normalize_primers`
- `fastq.normalize_read_names`
- `fastq.prepare_adapter_bank`
- `fastq.prepare_host_reference_bundle`
- `fastq.prepare_primer_bank`
- `fastq.repair_pairs`
- `fastq.remove_chimeras`
- `fastq.subsample_reads`
- `fastq.trim_terminal_damage`
- `fastq.verify_assets`

## Observer Coverage

- `fastq.validate_reads`
- `fastq.profile_read_lengths`
- `fastq.detect_adapters`
- `fastq.profile_overrepresented_sequences`
- `fastq.profile_reads`
- `fastq.report_qc`

Parser outputs must serialize to canonical JSON deterministically. Unknown
fields in supported tool outputs are ignored unless the domain parser documents
strict handling for that format. Missing required fields must produce a parser
error that identifies the missing tool output or field closely enough for
fixture debugging.

## Metrics Contract

Schema: `bijux.metrics.summary.v1`

Required governed metrics:

- `fastq.trim_reads`: trim retention and quality deltas.
- `fastq.filter_reads`: filter removals and retention.
- `fastq.remove_duplicates`: governed dedup summary with duplicate classes,
  provenance JSON, dedup mode, and keep-order semantics.
- `fastq.report_qc`: governed QC aggregation summary with contributor lineage,
  aggregation scope, MultiQC sample/module counts, and backend report paths.

Tool metrics schemas covered by parsers:

- `bijux.fastp.metrics.v1`
- `bijux.adapterremoval.metrics.v1`
- `bijux.seqkit.metrics.v1`
- `bijux.samtools.flagstat.v1`
- `bijux.fastqc.metrics.v1`
- `bijux.multiqc.metrics.v1`

Every metrics envelope must include metric provenance with the stage parameter
hash, executed tool identity, and all available input artifact hashes.

## Fixture Inventory

- `tests/fixtures/fastqvalidator/default/*`: FASTQ validation fixtures.
- `tests/fixtures/deduplicate/default/*`: duplicate-removal parser fixtures.
- `tests/fixtures/low_complexity/default/*`: low-complexity filter parser fixtures.
- `tests/fixtures/screen/default/*`: taxonomy screen parser fixtures.
- `tests/fixtures/seqkit/default/*`: seqkit text fixtures.
- `tests/fixtures/seqkit_stats/default/*`: canonical seqkit stats snapshots.
- `tests/fixtures/stage_contracts/default/*`: stage contract snapshots.
- `tests/fixtures/stage_output_bank/default/*`: representative stage output bank.
- `tests/fixtures/tool_metrics/default/*`: tool metrics parser fixtures.
- `tests/snapshots/*`: schema shape snapshots.

## Tool References

- fastp 0.23.x
- fastq_screen 0.14+
- seqkit 2.x
- AdapterRemoval
- bbmerge
- fastqvalidator

## External Truth

Planner stage mapping should point readers to the domain execution-support
truth file: `domain/fastq/execution_support.yaml`.
