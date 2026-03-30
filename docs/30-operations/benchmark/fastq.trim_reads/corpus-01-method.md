# `fastq.trim_reads` corpus-01 method

## Scope
- Stage: `fastq.trim_reads`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `trim_fairness`

## Governed tool cohort
- The benchmark runner resolves the tool roster from `bijux-dna registry list-tools --stage fastq.trim_reads --kind benchmark`.
- The current governed fairness cohort is:
  - `adapterremoval`
  - `alientrimmer`
  - `atropos`
  - `bbduk`
  - `cutadapt`
  - `fastp`
  - `fastx_clipper`
  - `leehom`
  - `prinseq`
  - `seqkit`
  - `skewer`
  - `trim_galore`
  - `trimmomatic`
- `seqpurge` is intentionally excluded from the governed cohort until its Lunarc image exposes a real trimming runtime instead of the current compatibility wrapper.

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Pin one comparable trim policy bundle across the whole cohort:
  - `min_length = governed tool default`
  - `quality_cutoff = null`
  - `n_policy = retain`
  - `adapter_policy = none`
  - `polyx_policy = none`
  - `contaminant_policy = none`
- Keep bank presets unset for this benchmark contract.

## Why the trim policy is bank-free
- The full governed trim fairness cohort does not implement bank-driven adapter trimming, polyX trimming, or contaminant handoff uniformly.
- Enabling those policies would silently benchmark only a subset of the governed cohort.
- The benchmark therefore measures shared trim behavior first, not profile-specific enrichment behavior.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and retention summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or lowest-retention samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.trim_reads
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.trim_reads
```

The runner and dossier command resolve the governed Lunarc corpus root and run root from `configs/bench/benchmark.toml`; change that config or pass `--config` only when you intentionally target a different governed workspace.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any published report whose rows drift from the run manifest policy bundle.
- Reject any published report that omits a tool row for any sample.
- Preserve backend-native report provenance through `raw_backend_report_format`.
