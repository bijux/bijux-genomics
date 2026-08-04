# Public API

`bijux-dna-bench` exposes a curated stable surface from the crate root:

```rust
pub use public_api::*;
```

The `public_api` module itself is private. Stable exports are owned by
`src/public_api/stable_surface.rs` and re-exported through `src/public_api/mod.rs`.

## Stable Operations

- `load_suite(path)` loads and validates a benchmark suite TOML file.
- `load_stage_campaign_registry(path)` loads and validates a benchmark stage-campaign registry TOML file.
- `load_benchmark_portfolio(path)` loads and validates a benchmark portfolio TOML file.
- `load_stage_interface_catalog(path)` loads and validates the stage-interface catalog, parameter configuration, and referenced repository contracts.
- `load_checked_in_benchmark_contracts()` preserves the campaign/portfolio return contract while validating every checked-in planning input.
- `load_checked_in_benchmark_planning_contracts()` returns the campaign registry, portfolio, and stage-interface catalog after validating defined identity consistency.
- `load_stage_report(path)` loads and structurally admits a typed stage report.
- `load_checked_in_stage_reports()` admits all checked-in reports against campaign and interface authority.
- `summarize(suite, observations, options)` builds a deterministic
  `BenchmarkSummary`.
- `compare(summary_a, summary_b)` compares completed summaries.
- `gate(policy, summary)` evaluates summary rows against a `GatePolicy`.
- `bench_data_dir()` resolves the crate-owned benchmark data directory.
- `bench_suites_dir()` resolves the crate-owned suite catalog directory.
- `bench_contracts_dir()` resolves the repository-owned benchmark contract directory.
- `bench_stage_campaigns_path()` resolves the repository-owned stage-campaign registry path.
- `bench_portfolio_path()` resolves the repository-owned benchmark portfolio path.
- `bench_stage_interfaces_path()` resolves the repository-owned stage-interface catalog path.
- `bench_reports_dir()` resolves the repository-owned benchmark report directory.
- `bench_local_config_dir()` resolves the repository-owned local benchmark config directory.
- `bench_fastq_local_stage_matrix_path()` resolves the governed FASTQ local stage matrix path.
- `bench_bam_local_stage_matrix_path()` resolves the governed BAM local stage matrix path.

`docs/COMMANDS.md` is the SSOT for this callable operation list.

## Stable Types

The public API re-exports benchmark model contracts from `bijux-dna-bench-model`,
including:

- `BenchmarkSuiteSpec`
- `BenchmarkObservation`
- `BenchmarkSummary`
- `BenchmarkDecision`
- `BenchmarkStageCampaignRegistry`
- `BenchmarkStageCampaignSpec`
- `BenchmarkPortfolio`
- `BenchmarkStageInterfaceCatalog`
- `BenchmarkStageInterfaceSpec`
- `BenchmarkPipelineParameterConfig`
- `BenchmarkStageReport`
- `BenchmarkToolReport`
- `GatePolicy`
- `GatePolicyOverrides`
- `GateDecision`
- `GateViolation`
- `MetricSummary`
- `SummaryRow`
- `BenchError`
- `BenchRunOptions`

## Export Policy

- Add stable exports in `src/public_api/stable_surface.rs`.
- Keep `src/lib.rs` thin; do not scatter stable exports across the root.
- Do not expose raw `serde_json::Value` in the public API.
- Do not expose planner, runner, API, or domain execution types as benchmark
  operations.
- Update `docs/COMMANDS.md`, `docs/BENCH_CONTRACT.md`, and tests when adding a
  new public benchmark operation or artifact shape.
