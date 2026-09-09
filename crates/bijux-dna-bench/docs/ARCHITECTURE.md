# Architecture

`bijux-dna-bench` owns benchmark admission, workflow composition and deterministic
summaries, decisions, comparisons and publications. Persisted schemas and pure
semantic rules belong to `bijux-dna-bench-model`. Concrete stage implementations
are supplied through the shared stage contract by an application composition root.

## Root Layout

- `Cargo.toml` declares dependencies; `README.md` introduces the crate.
- `bench/suites/` holds governed suite TOML data; Markdown belongs in `docs/`.
- `src/` owns the library; `tests/` owns its contract and boundary evidence.

## Source Layout

- `lib.rs` re-exports the curated `public_api/` stable surface.
- `workflow/` composes suite loading, comparison, gates, fairness, scope grouping,
  summary statistics, persistence and runtime attempt lifecycles.
- `workflow/campaign_engine/` separates case planning, evidence persistence,
  injected stage resolution, normalization, aggregation and report projection.
- `workflow/run_suite/` separates orchestration from artifact persistence;
  `workflow/summary/` separates grouping, row metrics and stratum assembly.
- `workflow/pipeline_profiles/` owns ancient-DNA, germline, imputation, population,
  taxonomy and amplicon requirements. `PipelineProfileEvidence` supplies admitted
  corpus compatibility and VCF matrices without exposing a process API.
- `repo/` owns explicit repository inputs, root discovery, path policy, run metadata,
  artifact loading and SQLite queries. `repo/run_artifacts/` separates manifest,
  metrics and observations; `repo/sqlite/queries/run_index/` separates queries
  from metadata-path policy.
- `repo/corpus/` owns BAM/FASTQ admission, normalized metadata, accession joins and
  cohort selection shared by execution and dossier consumers.
- `repo/stage_inventory/` admits domain matrices; `workflow/stage_inventory.rs`
  aggregates and publishes validated reports using model-owned schemas.
- `repo/pipeline_config.rs` adapts strict TOML and paired YAML input. Pipeline
  reference admission uses db-ref's complete catalog schemas; execution admission
  applies pure model rules before site lookup. Pipeline inventory adapters supply
  explicit-root readiness and VCF profile evidence.
- `artifacts/` owns source identities, output admission and serialization.
  `artifacts/source_assets.rs` inspects files, directories and index-prefix bundles
  with typed source kinds and member digests. `artifacts/writer/` separates
  observation reading, observation writing and structured JSON output.

## Dependency Direction

Normal dependencies provide model, core, infrastructure, runtime, science,
container, ENA and reference contracts. BAM domain contracts support BAM admission.
Stage plugins are injected; production code must not depend on API, planner,
runner or concrete stage crates. Profile rules do not acquire inputs or execute tools.

## Test Layout

- `tests/boundaries/` protects source, docs, dependencies and root layout.
- `tests/contracts/{api,benching,docs}/` covers API, catalog, ownership and fixtures.
- `tests/determinism/` covers stable ordering and snapshots;
  `tests/semantics/` covers gate semantics and rejection behavior.
- `tests/fixtures/` and `tests/snapshots/` hold governed evidence.

Test guidance belongs in `docs/TESTS.md`, with no README files below `tests/`.
Architecture and API contracts enforce the tree, Markdown locations, curated
exports, panic boundaries and repository/artifact ownership of raw JSON.

`repo/reference_fasta.rs` acquires and publishes paired benchmark FASTA/FAI inputs. The reference owner admits indexable byte layouts first; exact existing regular file pairs retain their identity, and malformed input leaves previous outputs intact.
