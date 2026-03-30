# `fastq.merge_pairs` on `corpus-01`

## Scope

This dossier benchmarks the governed `fastq.merge_pairs` stage on the paired-end subset of `corpus-01`.

- Corpus: `corpus-01`
- Species: human
- Platform: Lunarc Apptainer
- Cohort shape: `5` ancient paired-end samples and `5` modern paired-end samples
- Excluded inputs: all single-end corpus members

## Governed contract

The benchmark must run the full governed benchmark roster for `fastq.merge_pairs`:

- `adapterremoval`
- `bbmerge`
- `flash2`
- `leehom`
- `pear`
- `vsearch`

The comparison contract is pinned to one governed merge policy across every backend:

- `merge_overlap = governed tool default`
- `min_length = governed tool default`
- `unmerged_read_policy = emit_unmerged_pairs`

This stage intentionally leaves overlap threshold and minimum merged length at each backend's governed default because the supported capability surface is not identical across the full roster. The shared fairness contract is instead the fixed unmerged mate policy plus identical paired inputs on the same platform.

## Execution rules

- The run manifest must come from an executed benchmark, not `--dry-run`.
- The published dossier must come from the full paired cohort, not `--sample-limit`.
- Every paired sample must have one `report.json` with rows for the complete governed tool roster.
- Single-end samples must never appear in the merge benchmark report.

## Published artifacts

A complete published dossier for this stage contains:

- `summary.json`
- `sample_results.csv`
- `tool_runtime_summary.csv`
- `cohort_runtime_summary.csv`
- `sample_runtime_outliers.csv`
- `benchmark.md`

## Workflow

```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.merge_pairs
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.merge_pairs
```

The runner resolves the governed paired corpus root through `configs/bench/benchmark.toml`. Change that config or pass `--config` only when you intentionally target a different governed workspace.

## Interpretation guardrails

- Merge-rate comparisons are valid only when every backend sees the same paired inputs and the same governed unmerged mate policy.
- Backend-specific overlap and merged-length defaults must be documented, not overwritten silently, because those defaults are part of the observed scientific behavior.
- Runtime comparisons should be interpreted alongside merge rate and base retention, not in isolation.
- Ancient paired libraries are retained because they stress short-fragment overlap handling that modern libraries may not expose.
