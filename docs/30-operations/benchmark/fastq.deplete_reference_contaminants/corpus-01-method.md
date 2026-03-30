# `fastq.deplete_reference_contaminants` corpus-01 method

## Scope
- Stage: `fastq.deplete_reference_contaminants`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `contaminant_depletion_fairness`

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.deplete_reference_contaminants --kind benchmark`.
- The current governed fairness cohort is:
  - `bowtie2`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Hold the contaminant-reference lineage constant across the full corpus:
  - identical input hashes
  - identical contaminant bundle digest
  - identical reference-index provenance
  - identical contaminant-depletion policy hash
- Preserve retained and removed-read evidence in every successful sample report.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and depletion summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or most aggressive samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.deplete_reference_contaminants
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.deplete_reference_contaminants
```

The runner resolves the governed contaminant reference lineage through `configs/bench/benchmark.toml` unless you intentionally override `REFERENCE_INDEX` or `--reference-index` for a non-governed audit.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that omits the governed contaminant bundle lineage or index provenance.
- Reject any dossier that omits a tool row for any sample.
