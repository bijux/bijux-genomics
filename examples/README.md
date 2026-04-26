# Examples Index

Canonical example index: `examples/index.yaml` (generated).

## Example Classes

- Runnable examples: carry `example.toml`, golden outputs, and a generated entry in `examples/index.yaml`.
- Recipe-only benchmark docs: live under domain folders, are listed in `examples/RECIPE_ONLY.txt`, and remain `README.md`-only until the CLI contract is ready.
- Data corpora: live under `examples/data/` and provide reproducible inputs for runnable examples.

- `fastq_qc_pre_bench`: `examples/fastq/qc-pre-bench/`
- `fastq_validate_reads_bench`: `examples/fastq/validate-reads-bench/`
- `fastq_edna_mini`: `examples/fastq/edna-mini/`
- `vcf_imputation_mini`: `examples/vcf/imputation-mini/`
- `vcf_downstream_demography_mini`: `examples/vcf/downstream-demography-mini/`
- `vcf_downstream_vcf_full_mini`: `examples/vcf/downstream-vcf-full-mini/`
- `vcf_damage_aware_genotype_mini`: `examples/vcf/damage-aware-genotype-mini/`
- `template`: `examples/_template/`
- `data_corpus_01`: `examples/data/corpus-01/`
- `data_corpus_01_mini`: `examples/data/corpus-01-mini/`

## Recipe-Only Benchmark Docs

- `examples/fastq/index-reference-bench/`
- `examples/fastq/normalize-abundance-bench/`

## Contracts

- Treat `examples/index.yaml` as the SSOT for runnable example IDs.
- Treat `examples/POLICY.md` as the boundary contract for runnable examples, recipe-only benchmark docs, and notebook usage.
