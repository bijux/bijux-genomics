# EXAMPLES

Canonical examples index.

## Example IDs
- `template`: `examples/_template/README.md`
- `fastq_qc_pre_bench`: `examples/fastq/qc-pre-bench/README.md`
- `fastq_validate_reads_bench`: `examples/fastq/validate-reads-bench/README.md`
- `fastq_edna_mini`: `examples/fastq/edna-mini/README.md`
- `vcf_imputation_mini`: `examples/vcf/imputation-mini/README.md`
- `vcf_downstream_demography_mini`: `examples/vcf/downstream-demography-mini/README.md`
- `corpus_01_mini`: `examples/data/corpus-01-mini/README.md`
- `corpus_01`: `examples/data/corpus-01/README.md`

## Recipe-Only Benchmark Docs
- `examples/fastq/index-reference-bench/README.md`
- `examples/fastq/normalize-abundance-bench/README.md`

## Root Example Guide
- `examples/README.md`
- `examples/POLICY.md`
- `examples/RECIPE_ONLY.txt`

## Purpose
Define the navigation contract for runnable examples, recipe-only benchmark docs, and example-linked corpora.

## Scope
Applies to the `examples/` tree, the generated example index, and the docs that explain how example classes differ.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- `examples/index.yaml` is the SSOT for runnable example IDs only.
- Recipe-only benchmark docs are intentionally excluded from `examples/index.yaml` until they grow an executable example contract.
- `examples/data/` holds corpora inputs and can appear in navigation docs without being treated as runnable examples.
