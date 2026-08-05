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
- `load_stage_interface_catalog_from_repo(path, repository_root)` performs the same admission against an explicit workspace root for production callers.
- `load_checked_in_benchmark_contracts()` preserves the campaign/portfolio return contract while validating every checked-in planning input.
- `load_checked_in_benchmark_planning_contracts()` returns the campaign registry, portfolio, and stage-interface catalog after validating defined identity consistency.
- `load_stage_report(path)` returns a typed admission envelope binding the exact source bytes and canonical parsed report to separate SHA-256 identities.
- `load_stage_reports_from_dir(path, registry, interfaces)` admits checksummed reports against supplied authorities; raw report values are never returned to aggregation callers.
- `load_checked_in_stage_reports()` admits all checked-in reports against campaign and interface authority.
- `plan_stage_campaign_attempts(registry, repository_commit, campaign_registry_sha256, run_id, campaign_id, attempt_id, observed_at_utc)` deterministically derives a source-locked plan with one validated attempt per declared tool.
- `verify_campaign_plan_source(plan, repository_commit, campaign_registry_sha256)` rejects stale source or campaign declaration identity before execution state is created.
- `ensure_stage_campaign_attempts(root, plan)` validates source locks, then preflights and crash-safely initializes every immutable planned attempt in a campaign cycle.
- `benchmark_attempt_directory(root, identity)` derives the permanent attempt path from validated identity.
- `initialize_attempt_lifecycle(root, record)` delegates publication of one immutable planned attempt to runtime.
- `append_attempt_lifecycle_transition(root, identity, transition)` delegates one no-clobber lifecycle event to runtime.
- `load_attempt_lifecycle(root, identity)` delegates reconstruction and validation of the complete event chain to runtime.
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
- `BenchmarkStageCoverage`
- `BenchmarkStageCoverageSummary`
- `BenchmarkStageCoverageRow`
- `BenchmarkStageDefinitionStatus`
- `BenchmarkStageExecutionStatus`
- `BenchmarkPortfolio`
- `BenchmarkStageInterfaceCatalog`
- `BenchmarkStageInterfaceSpec`
- `BenchmarkPipelineParameterConfig`
- `BenchmarkStageReport`
- `BenchmarkToolReport`
- `BenchmarkRunIdentity`
- `BenchmarkAttemptIdentity`
- `BenchmarkAttemptRecordV2`
- `BenchmarkAttemptTransition`
- `BenchmarkCycleStatusV2`
- `BenchmarkCampaignPlan`
- `BenchmarkExecutionSpec`
- `GatePolicy`
- `GatePolicyOverrides`
- `GateDecision`
- `GateViolation`
- `MetricSummary`
- `SummaryRow`
- `BenchError`
- `BenchRunOptions`
- `benchmark_execution_spec_schema`
- `validate_execution_spec`
- `build_execution_spec`
- `BenchmarkExecutionMaterial`

## Export Policy

- Add stable exports in `src/public_api/stable_surface.rs`.
- Keep `src/lib.rs` thin; do not scatter stable exports across the root.
- Do not expose raw `serde_json::Value` in the public API.
- Do not expose planner, runner, API, or domain execution types as benchmark
  operations.
- Update `docs/COMMANDS.md`, `docs/BENCH_CONTRACT.md`, and tests when adding a
  new public benchmark operation or artifact shape.
