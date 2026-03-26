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

The comparison contract is pinned to one merge configuration for every backend:

- `merge_overlap = 11`
- `min_length = 20`
- `unmerged_read_policy = emit_unmerged_pairs`

These settings follow the scientific defaults recorded in [SCIENTIFIC_DEFAULTS.md](/Users/bijan/bijux/bijux-dna/docs/20-science/SCIENTIFIC_DEFAULTS.md).

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
- `lunarc.md`

## Interpretation guardrails

- Merge-rate comparisons are valid only when overlap threshold, minimum merged length, and unmerged mate policy are fixed.
- Runtime comparisons should be interpreted alongside merge rate and base retention, not in isolation.
- Ancient paired libraries are retained because they stress short-fragment overlap handling that modern libraries may not expose.
