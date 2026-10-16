# Architecture

`bijux-dna-bench` owns benchmark admission, workflow composition and deterministic
summaries, decisions, comparisons and publications. Persisted schemas and pure
semantic rules belong to `bijux-dna-bench-model`. Concrete stage implementations
are supplied through the shared stage contract by an application composition root.

## Root Layout

- `Cargo.toml` declares dependencies; `README.md` introduces the crate.
- `benchmarks/suites/` holds governed suite TOML data; Markdown belongs in `docs/`.
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
- Benchmark source identity types belong to `bijux-dna-bench-model`;
  `repo/source_assets.rs` inspects current files, directories and index-prefix bundles.
  `artifacts/` owns output admission and serialization. `artifacts/writer/` separates
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

Hardening campaign decisions belong to `bijux-dna-bench-model::derive_hardening_benchmark_campaign`. `repo/hardening_campaign.rs` admits persisted reports and owns atomic publication. The benchmark facade preserves all three existing public operations; source joins, the goal catalog and check evaluation do not acquire files or depend on runtime services.

`repo/hpc_inputs/` owns current source inspection, stage-hint provenance joins and produced-output exclusion. Its private path classifier handles the supported argv and shell forms. The CLI adapts stage-plan hints and supplies the generated-output root explicitly; source contents are reacquired for each request.

HPC asset staging accepts explicit typed command rows and source hints. Model policy assembles canonical jobs and compares current identities; `repo/hpc_asset_staging.rs` owns destination admission, observations and persistence; `workflow/hpc_asset_staging.rs` composes collection. CLI adapters acquire upstream command/stage declarations and present results.

Scratch layouts separate model policy (job paths, cleanup and admitted aggregate counters), repository observation/persistence, and workflow composition. The service accepts selected jobs, typed command rows, resource declarations and stage input hints. Duplicate or mismatched command identities reject before source observation. Collection does not publish directories; explicit publication uses infrastructure atomic replacement.

Runtime probe acquisition belongs to `repo/runtime_probe.rs`. It validates requested path components and the source declaration tool identity, preserves preferred-domain then canonical fallback order, and returns the exact selected source path. An existing malformed or contradictory preferred declaration fails without fallback. The model carries only the runtime-readiness projection; full domain-tool schema and capability admission remain with the domain contract.

`repo/tool_execution_modes.rs` reads current execution-mode TOML and delegates schema and assignment admission to the model owner. The CLI adapts serving-map bindings. `workflow/readiness/tool_execution_modes.rs` acquires current probes against a model-owned admitted scope and only finishes a report after every requested tool/domain observation passes. The model owns report shape, counts and semantic admission; executable availability remains a separate contract.

`workflow/hpc_campaign/submission/` owns selected-job ordering and encrypted publication sequencing. It receives scheduler, code-observation and bundle-publication capabilities through `HpcSubmissionBackend`; it never imports runner or API. `repo/hpc_campaign/submission.rs` owns scripts and operator files, while `artifacts/hpc_campaign.rs` encodes model-owned payloads. Real submissions preserve scheduler capture files. Failures stop the sequence; prior external effects remain observable and are not a transactional rollback.

BAM campaign source admission, goal selection, evidence joins and check projection belong to `bijux-dna-bench-model::derive_bam_benchmark_campaign`. Its private `policy/bam_campaign/` groups validation, alignment, filtering, metrics, ancient-DNA, contamination, identity and resource/template goals. `repo/bam_campaign.rs` owns persisted report admission and atomic publication; the benchmark/API facade and CLI invocation remain stable. Goal IDs, scenario/check strings, ordering, error contexts and schemas retain their existing meanings.

Benchmark taxonomy judgment separates declared corpus truth from classifier summary observation. `bijux-dna-bench-model` owns typed output records, the supported summary codec and pure expected-presence/count evaluation. `bijux-dna-bench` acquires explicit manifest/table/report files and publishes the existing JSON result. The CLI requests those services through the API benchmark facade. Summary aliases and legacy verdicts remain compatible; summary admission does not validate the full classifier stage report or establish biological truth.

FASTQ campaign source admission, goal selection, row/finding/queue joins and summary
construction belong to `bijux-dna-bench-model::derive_fastq_benchmark_campaign`.
Its private `policy/fastq_campaign/` groups validation, read layout, sampling,
preprocessing, depletion and evidence goals. Each concern owns its existing catalog
declarations and check expressions. The benchmark `workflow/fastq_campaign.rs`
retains persisted report admission and atomic publication and reexports the pure
operation through the unchanged benchmark/API/CLI facade. Goal IDs, scenario and
check literals, canonical order, schemas and error priorities keep their existing
meaning; these are benchmark-readiness projections, not benchmark execution or
scientific qualification.

VCF campaign source admission, goal selection, row/finding/queue joins and summary
construction belong to `bijux-dna-bench-model::derive_vcf_benchmark_campaign`.
Its private `policy/vcf_campaign/` groups variant admission, calling, haplotypes,
cohort analysis, annotation and evidence goals. Each concern owns its existing catalog
declarations and check expressions. The benchmark `workflow/vcf_campaign.rs`
retains persisted report admission and atomic publication and reexports the pure
operation through the unchanged benchmark/API/CLI facade. Goal IDs, scenario and
check literals, canonical order, schemas and error priorities keep their existing
meaning; these are benchmark-readiness projections, not benchmark execution or
scientific qualification.

Cross-domain campaign source admission, goal selection, row/finding/queue joins and summary
construction belong to `bijux-dna-bench-model::derive_cross_benchmark_campaign`.
Its private `policy/cross_campaign/` groups domain handoffs, execution policy,
source integrity and reviewer evidence goals. Each concern owns its existing catalog
declarations and check expressions. The benchmark `workflow/cross_campaign.rs`
retains persisted report admission and atomic publication and reexports the pure
operation through the unchanged benchmark/API/CLI facade. Goal IDs, scenario and
check literals, canonical order, schemas and error priorities keep their existing
meaning; these are benchmark-readiness projections, not benchmark execution or
scientific qualification.
The cross-domain queue join keeps each complete queue entry when any affected
row matches a goal. It preserves the original affected-row list, unlike FASTQ
and VCF goal projections, which trim that list to matched rows.

`workflow/alias_resolution.rs` owns normalized benchmark tool cluster membership as well as canonical spelling preference. Callers supply their exact acquired tool scope; repeated identical IDs and singleton groups do not create aliases. Its private `alias_resolution/violations.rs` classifies stale stage and tool spellings across acquired candidate, active, expected-result and rendered-command rows, preserving surface metadata and stable violation order. The CLI keeps acquisition, report envelopes and validation, and publication before the existing stale-alias refusal, so FASTQ/BAM normalization and all-domain auditing retain their separate coverage.

`workflow/readiness/retained_tools.rs` derives retained tool rows from explicit
executable binding observations and external-artifact delivery contracts. Binding
counts retain repeated observations while stage/domain sets remain unique and
ordered. External contracts extend retention without manufacturing ready evidence.
The CLI acquires both input families, validates the persisted model-owned report,
and presents or publishes the unchanged inventory.
