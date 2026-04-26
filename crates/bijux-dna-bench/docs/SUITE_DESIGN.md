# Suite Design

Checked-in benchmark suites live under `crates/bijux-dna-bench/bench/suites/`.

## Suite Rules

- Use `schema_version = "bijux.bench.suite.v1"`.
- Use canonical stage ids, not legacy aliases.
- Use governed tool ids admitted by the relevant domain contract.
- Use structured `param_bindings` for stage-scoped and tool-scoped parameter
  coverage.
- Include stratification metadata required by the benchmark model.
- Cover governed benchmark stages and important multi-tool cohorts.
- Keep suite files as TOML data; explanatory Markdown belongs in `docs/`.

## Current Suite Families

- BAM stage suites: `bam_stage*.toml`.
- FASTQ surface suites: `fastq_*_surface.toml`.
- FASTQ cohort suites for trimming, validation, duplicates, host/reference
  depletion, taxonomy, amplicon, and terminal trimming behavior.
- HPC-focused FASTQ suites: `fastq_hpc_*.toml`.

## Adding A Suite

1. Add a TOML file under `crates/bijux-dna-bench/bench/suites/` with a durable suite id.
2. Validate it through `BenchmarkSuiteSpec`.
3. Ensure stage ids, tool ids, and parameter bindings are governed.
4. Add fixtures or snapshots when the suite introduces a new artifact shape or
   comparison behavior.
5. Run the contract suite.

## Guardrails

`tests/contracts/benching/suite_catalog.rs` validates checked-in suites,
canonical stage ids, admitted tool coverage, structured parameter bindings,
multi-tool validation coverage, and governed FASTQ benchmark stage coverage.
