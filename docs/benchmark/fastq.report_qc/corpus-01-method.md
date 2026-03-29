# `fastq.report_qc` corpus-01 method

## Scope
- Stage: `fastq.report_qc`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `qc_aggregation_fairness`

## Governed tool cohort
- The benchmark runner resolves the tool roster from `bijux-dna registry list-tools --stage fastq.report_qc --kind benchmark`.
- The current governed fairness cohort is:
  - `multiqc`

## Governed contributor surface
- `fastq.report_qc` is benchmarked as a report-only aggregation stage over a fixed governed QC manifest.
- The governed contributor set for `corpus-01` is:
  - `fastq.validate_reads` via `fastqvalidator`
  - `fastq.detect_adapters` via `fastqc`
  - `fastq.profile_reads` via `seqkit_stats`
  - `fastq.profile_read_lengths` via `seqkit_stats`
- Each sample-level aggregation run must consume the governed contributor artifacts written by those upstream stages rather than rebuilding an ad hoc QC bundle.

## Execution contract
- Use normalized FASTQ inputs from `corpus_01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Pin the governed aggregation contract across the whole corpus:
  - `aggregation_engine = "multiqc"`
  - `aggregation_scope = "governed_qc_artifacts"`
  - `report_only = true`
  - `mutates_fastq = false`
  - `may_change_read_count = false`
- Require a governed sample manifest that records:
  - the exact contributor artifact paths
  - the lineage hash for the governed QC bundle
  - the raw FastQC directory routed into MultiQC

## Why this stage is benchmarked differently
- `fastq.report_qc` does not modify reads, pairs, or bases.
- The benchmark therefore measures governed manifest handling, aggregation runtime, and publication integrity rather than retention deltas.
- This dossier is only valid when every sample publishes a consistent MultiQC bundle over the same governed upstream evidence surface.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and aggregation summary.
- `cohort_runtime_summary.csv`: era/layout and size-band runtime breakdowns.
- `sample_runtime_outliers.csv`: slowest aggregation samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.report_qc
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.report_qc
```

The default corpus root is loaded from `configs/bench/benchmark.toml`. Update that config or pass `--config` when rerendering against a different governed workspace.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any published report produced from `--dry-run`.
- Reject any published report that omits a tool row for any sample.
- Reject any published report whose governed QC input count does not match the expected contributor contract.
- Reject any published report that lacks the governed manifest, raw FastQC directory, MultiQC report, or MultiQC data directory for successful rows.
