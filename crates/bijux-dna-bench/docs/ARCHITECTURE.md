# Architecture

`bijux-dna-bench` is a benchmark contract crate. It reads governed benchmark
observations and suite specifications, then produces deterministic summaries,
gate decisions, comparisons, and benchmark artifacts. It plans generic benchmark
campaigns from governed registries but does not execute product workflows.

## Root Layout

- `Cargo.toml` defines the crate dependency graph.
- `README.md` is the only root documentation file.
- `crates/bijux-dna-bench/bench/suites/` contains checked-in benchmark suite TOML files.
- `docs/` contains the 10 authoritative crate docs.
- `src/` contains the library implementation.
- `tests/` contains boundary, contract, determinism, semantic, fixture, and
  snapshot tests.

`bench/` is data/catalog ownership, not a documentation directory. Markdown docs
belong under `docs/`.

## Source Layout

`src/workflow/pipeline_profiles/` owns benchmark profile selection and scientific handoff requirements, grouped into ancient-DNA, germline, imputation, population, taxonomy and amplicon rules. `PipelineProfileEvidence` supplies admitted corpus compatibility and VCF matrix data; repository acquisition and process execution are outside these profile rules.

- `src/lib.rs` keeps the crate root thin and re-exports `public_api`.
- `src/public_api/` owns the curated stable surface and stable-surface owner.
- `src/workflow/` owns suite loading, summarization, comparison, gate evaluation,
  suite persistence, runtime attempt-lifecycle composition, fairness checks,
  scope grouping, and summary statistics.
- `src/workflow/campaign_engine/` separates generic case planning, immutable
  evidence persistence, dependency-injected stage-plugin resolution,
  normalization, aggregation, and report projection.
- `src/workflow/run_suite/` separates suite orchestration from artifact
  persistence.
- The stage-contract crate owns bindings and downstream probes. Domain stage
  crates implement normalization behind `StagePlugin`; the benchmark workflow
  owns their generic lifecycle and receives implementations through that shared
  contract.
- `src/workflow/summary/` separates grouping, row metrics, and stratum assembly.
- `src/repo/` owns repository-root discovery, workspace path policy, domain-specific
  FASTQ and BAM corpus admission, repository contracts, run metadata, run artifact loaders,
  and sqlite run-index queries.
- `src/repo/run_artifacts/` separates manifest, metrics, and observations
  loading.
- `src/repo/sqlite/queries/run_index/` separates run-index queries from
  metadata-path policy.
- `src/artifacts/` owns source content identities, output admission, and deterministic serialization.
- `src/artifacts/source_assets.rs` inspects files, directories, and index-prefix
  bundles before staging, with typed source kinds and stable member digests.
- `src/artifacts/writer/` separates observation reading, observation writing,
  and structured JSON writing.

## Test Layout

- `tests/boundaries/` protects source, docs, and root layout.
- `tests/contracts/api/` protects the public API and API hygiene.
- `tests/contracts/benching/` protects benchmark contracts, suite catalog rules,
  workspace paths, and ownership.
- `tests/contracts/docs/` protects docs-to-fixture alignment.
- `tests/determinism/` protects stable ordering, comparison output, and realistic
  snapshots.
- `tests/semantics/` protects gate semantics and metric rejection behavior.
- `tests/fixtures/` contains governed benchmark input fixtures.
- `tests/snapshots/` contains governed public API and compare snapshots.

Test documentation lives in `docs/TESTS.md`; README files are intentionally not
allowed below `tests/`.

## Dependency Direction

Allowed normal dependencies are lower-level model, core, environment, infra,
runtime, and science contracts required to admit data, derive independent truth,
load finished artifacts, validate container catalog identity, and write
deterministic benchmark outputs. The BAM domain contract may provide read-only BAM header identity;
stage and planner crates remain downstream. Concrete stage plugins are injected
by an application composition root. `bijux-dna-bench` must not depend on API,
planner, runner, or domain execution crates for production behavior.

## Guardrails

`tests/boundaries/architecture_tree.rs` enforces the root layout, docs allowance,
source tree, test tree, and Markdown location rules. Contract tests enforce the
public API snapshot, no raw JSON outside repo/artifacts, no public API panics,
and suite catalog governance.

`src/repo/corpus/` groups BAM and FASTQ governed admission and normalized FASTQ metadata. Normalized discovery, accession joins, and publication cohort selection serve benchmark execution and dossier consumers through the public API.

`src/repo/stage_inventory/` admits local domain matrices from an explicit repository root. `src/workflow/stage_inventory.rs` aggregates the admitted inventories and publishes their validated report atomically. Persisted inventory models and report validators belong to `bijux-dna-bench-model`; CLI consumers use the curated API facade.

`repo/pipeline_reference.rs` admits benchmark pipeline reference identities using explicit-root db-ref catalog queries. The reference owner reads complete catalog schemas; benchmark admission checks the declared default build and bundle binding. The CLI calls this service without duplicating catalog DTOs or IO.

`repo/pipeline_config.rs` adapts governed TOML and paired YAML files into benchmark-model pipeline inputs. Bundle selection, contextual parse failures and strict field handling have one repository owner; CLI/HPC callers consume the public service facade. Scientific and execution-context admission remain separate from transport parsing.

Pipeline inventory acquisition and explicit site-profile lookup belong to repository adapters. The benchmark model admits execution-context symbols without filesystem access; repository admission then checks custom site-profile availability. Pipeline graph planning and profile semantics retain their existing owners, with no additional normal Cargo edges.
