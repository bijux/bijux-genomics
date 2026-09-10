# bijux-dna Tests

## Test Layout
`tests/` should contain executable integration test entrypoints, owned test modules, snapshots, and
shared path helpers. It should not contain README placeholder files or empty taxonomy directories.

```text
tests/
├── bench_readiness_bam.rs
├── bench_readiness_fastq.rs
├── bench_readiness_vcf.rs
├── benchmark_readiness.rs
├── benchmark/
│   ├── bam_readiness/
│   ├── fastq_readiness/
│   ├── vcf_readiness/
│   └── readiness/
├── boundaries.rs
├── boundaries/
├── contracts.rs
├── contracts/
├── guardrails.rs
├── schemas.rs
├── schemas/
├── snapshots.rs
├── snapshots/
└── support/
```

## Coverage Map
- `tests/bench_readiness_bam.rs` and `tests/bench_readiness_vcf.rs`: domain readiness contracts grouped by inventory, adapter, metric and scientific responsibility. Standard-harness cases share one process guard per executable; nextest retains case-level scheduling. The four dedicated VCF parser-failure and missing-input targets remain entrypoints for their native fast lanes.
- `tests/bench_readiness_fastq.rs`: FASTQ stage bindings, execution surfaces, evidence maps and metrics contracts, grouped under `tests/contracts/benchmark/fastq_readiness/`. Cases retain their function names and use one shared process guard; nextest partitions individual cases and retains the full-profile repository mutation group.
- `tests/benchmark_readiness.rs`: active stage/tool scope, adapters, parser fixtures, output declarations/contracts, expected results and report maps. JSON and TSV assertions live in `tests/contracts/benchmark/readiness/`. One executable retains individual case identities under report modules; the slow roster matches the unchanged function-name suffixes. The standard harness shares one process environment guard, while nextest can schedule separate cases.
- `tests/bench_local_vcf_{calling,processing,population,population_truth,admission}.rs`: local VCF command and file contracts under `tests/contracts/benchmark/vcf_smoke/`, compiled by workflow responsibility. The population-truth executable keeps the two existing science-fast cases isolated from broader population operations. Native case identities retain their function suffixes, and the full-profile mutation group remains in force.
- `tests/bench_local_slurm.rs`: stage and all-domain script rendering, body/shell validation, dependency, path and submission contracts under `tests/contracts/benchmark/slurm/`. One executable retains individual cases and feature conditions; the full-profile repository mutation group still applies.
- `tests/boundaries.rs`: layout, docs placement, dependency, public-surface, no-process-spawn, and
  guardrail checks.
- `tests/boundaries/architecture_tree.rs`: root, docs, source, command, and public API tree
  contract, including the integration-test taxonomy and support-helper location.
- `tests/boundaries/command_inventory.rs`: `docs/COMMANDS.md` command inventory, section, alias,
  and verification-command contract.
- `tests/contracts.rs`: command behavior, bank, dry-run, and HPC layout contracts.
- `tests/contracts/dry_run.rs`: dry-run command families and VCF toy execution coverage.
- `tests/schemas.rs`: public API surface snapshots.
- `tests/snapshots.rs`: CLI help output snapshots and docs-to-help checks.
- `src/process_exit.rs` unit tests: operator error and exit-code behavior.

Markdown files under `tests/` are not allowed; test taxonomy belongs here.

## Commands
- Fast layout/dependency check:

```text
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --test boundaries --no-default-features
```

- Command behavior contracts:

```text
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --test contracts --no-default-features
```

- Full crate:

```text
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --no-default-features
```

## Snapshot Rules
Update snapshots only when the command surface, output format, or public export intentionally
changes. Snapshot diffs must be reviewed as contract changes, not accepted mechanically.

Scientific fixture command contracts are grouped in `fixtures_fastq`, `fixtures_bam`,
`fixtures_vcf`, `fixtures_ancient_dna`, and `fixtures_amplicon`. The corresponding
`tests/fixtures/` subtrees separate corpus families and observed/expected JSON/text
interfaces. Each executable shares the existing environment/cwd guard; full-profile
repository mutation serialization and function-based slow selection still apply.
`science-fixtures-fast` selects these suites plus its existing corpus, aggregate,
and population smoke targets. Test function names remain stable beneath module
namespaces so function-based routing and slow rosters retain their meaning.

Executable fixture assertions live in `tests/contracts/fixtures/`; `tests/fixtures/` holds input data only. Grouped entrypoints keep their binary and case identities.

Local simulation command contracts link through `bench_local_simulation`. The `contracts/local_simulation` tree separates stage, all-domain and pipeline cases; JSON, file, declared-output and failure assertions retain their original feature conditions. A single shared support module serializes process-global environment/cwd mutations for ordinary Cargo test execution. Nextest still isolates each test process and uses the existing repository mutation locks.

Active-scope completion runs one sandboxed workflow. Its JSON command case compares the persisted report with stdout and covers all nineteen checks, active matrix proof paths, and exact VCF closure detail. A separate workflow solely for file existence and duplicated report counters is unnecessary. This suite does not separately exercise the plain-text path presentation mode.

All-domain catalog and admission contracts link through `bench_readiness_all_domain_catalog` and `bench_readiness_all_domain_admission`. Their modules in `contracts/benchmark/all_domain_readiness/` retain all sixteen JSON, TSV and file cases. Catalog owns stage/tool inventory, expected results and job coverage; admission owns scope blockers and forbidden readiness states. Each binary shares the process-state guard. Nextest still schedules individual cases under the repository mutation group; function-based slow filters retain their names.

Retained-tool readiness uses one complete workflow per BAM/VCF domain. BAM compares stdout and persisted JSON within one sandbox lifetime; VCF inspects its persisted failure report and preserves licensed-evidence blockers. Both retain their exact governed goal ranges and container/external-artifact counts. A separate BAM plain-text path invocation and the duplicate VCF failure invocation are retired.

The HPC stage benchmark array suite renders one private script/manifest pair. That same publication covers exact result commands and output paths, container resolution, SLURM dispatch, rendered and validated path presentation, and independent script/manifest drift rejection. Each mutation restores its original bytes before the next check. Duplicate render setup and a second integration binary are unnecessary.

The HPC pipeline node array suite renders one private script/manifest pair. That same publication covers exact result commands and output paths, dependency lineage, SLURM dispatch, rendered and validated path presentation, and independent script/manifest drift rejection. Each mutation restores its original bytes before the next check. Duplicate render setup and a second integration binary are unnecessary.

The HPC asset-staging suite shares one private publication across governed input/provenance assertions, validated-path presentation and stale-count rejection. The fixture retains its directory guard for cleanup and restores the manifest after drift. Both validator calls remain; duplicate rendering and its integration binary are retired.

Operational readiness passes its current stage table, expected results, rendered commands, output declarations and parser report into harness validation. Existing goal and cross-report checks still run. A failed upstream report becomes a failed dependency instead of being regenerated; standalone harness commands acquire their own reports. The native dependency case checks borrowing, absence of hidden retries and standalone acquisition.

The scratch-layout publication suite retains one private manifest for schema/root/source-link assertions, validated-path presentation, and stale input-link count rejection. All original assertions and both validator invocations remain; duplicate render setup and its second integration executable are retired. The directory guard spans the whole case and drift is restored before completion.

Shared guardrail parser regressions are owned and executed by the policy crate. This boundary suite retains its crate-specific architecture checks; it does not compile another crate's test source. Constructor-only guardrail wrappers provide no source-check coverage and are retired.

Variant truth admission and expected/observed JSON/text output cases share the
`fixtures_vcf` target, grouped by scientific truth family. Pipeline validation JSON
and file-publication cases share `plan_validate_pipelines`, grouped by declared pipeline.
Both retain the repository-mutator nextest group and original per-command guards.
These are command boundary contracts; scientific truth semantics remain owner-tested.

Benchmark path admission/cleanup, schema validation and planning projections are
grouped in `bench_paths`, `bench_schema_validation` and `bench_planning`.
Their modules pair JSON with file, TSV, TOML, shell and argv publication contracts
under each responsibility. All original cases and repository-mutator guards remain;
test execution can be sharded independently of these shared compilation targets.
