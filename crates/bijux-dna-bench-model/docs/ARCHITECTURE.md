# Architecture

`bijux-dna-bench-model` is the pure benchmark model crate. It owns typed
benchmark contracts, deterministic validation, comparison reports, gate policy
and decision records, and statistical helpers.

## Source Map

- `src/public_api/` exposes the curated stable surface.
- `src/model/` owns observations, suites, decisions, summaries, graph nodes, stage-profile data
  fitness, persisted container report-portfolio identities, permanent-run report dispositions,
  normalized-result records and execution-metrics envelopes, and suite support records.
- `src/model/local_pipeline_dag/` separates TOML/YAML configuration inputs from persisted validation reports; three configuration schemas share the model schema registry.
- `src/contract/local_pipeline_dag/` separates input declaration admission from persisted report validation. Project sources, ancient DNA tool policy and reference/variant asset declarations are checked without repository IO.
- `src/contract/` validates records, suite shape, stage governance, graph edges, data-fitness and
  portfolio closure, parameter bindings, container and benchmark report freshness, diversity
  requirements, and analysis requirements.
- `src/compare/` builds deterministic typed diffs between benchmark summaries.
- `src/policy/` owns typed gate configuration, violations, and decision records.
  `bijux-dna-science` owns scientific gate evaluation.
- `src/diagnostics/` owns the stable benchmark error taxonomy.
- `src/stats/` owns deterministic estimators, seeded bootstrap intervals, and
  MAD-based outlier detection.

## Test Map

- Repository policy checks enforce general source, documentation and dependency
  boundaries. The retired empty boundary target supplies no model coverage.
- `tests/contracts.rs` checks validation behavior.
- `tests/determinism.rs` protects seeded and non-random behavior.
- `tests/schemas.rs` locks public exports and docs-linked API expectations.
- `tests/semantics.rs` covers explainability and metric meaning.

## Boundaries

This crate does not run benchmarks, read or write benchmark artifacts, call the
CLI/API, or perform runtime orchestration. Execution and persistence live in
downstream crates.

## Dependency Direction

Normal dependencies are pure model support: core ids/contracts, stage contracts,
FASTQ and VCF domain identifiers and metrics contracts, analysis metric semantics,
serialization, validation errors, and seeded sampling.

## Command Inventory

`docs/COMMANDS.md` lists model-level operations owned by this crate and must
stay aligned with public API and contract tests.

Local pipeline execution-context admission is a pure contract: identity, frontend/compute surface closure, duplicate rejection and optional campaign-path presence. Repository site-profile availability is evaluated by the benchmark repository adapter after this contract passes.

`contract/local_hpc_job_graph.rs` derives deterministic dependency graphs from typed submission jobs and an explicit manifest identity. Duplicate and missing identities, cycles, counters and edge closure are admitted without filesystem access. CLI composition acquires jobs through the submission producer.

`contract/local_hpc_tool_scope.rs` projects selected job membership into deterministic tool scopes. Declared tool identities remain separate even when lookup aliases coincide; runtime availability and transport admission stay with execution owners.

HPC resume and result-collection requests reuse their selected submission jobs for graph and resolver projections. Reuse is limited to that request; subsequent commands reacquire governed inputs.

`contract/local_hpc_dependency_simulation/` separates pure branch-failure simulation from persisted report validation. The graph and output identity are explicit inputs; CLI adapters own filesystem publication and drift comparison.

HPC array resource admission and component-wise envelope construction share one pure model owner. CLI array renderers adapt selected rows; both persisted array validators use the same checked walltime contract.

`model/readiness/` groups expected results, parser observations, collected results, full reports, dashboards, and local smoke observations. Public crate-root types and schema entrypoints remain stable; semantic admission belongs to `contract/` and acquisition belongs to service owners. This module layout does not create a separate Cargo compilation unit.

`policy/hardening_campaign/` admits matrix/appraisal/queue identities and builds hardening campaign reports without IO. Goal metadata and check projections are grouped by diagnostics, execution, corpora, databases, planning, comparisons, evidence, security and operations. Existing goal IDs, catalog order, check strings and readiness meanings are preserved. The goal catalog is assembled once; each report uses the caller's current typed observations. Its native tests follow the same policy concerns.

Execution-mode validation uses model-owned report contracts and scope/probe admission, benchmark-owned fresh observation, and CLI serving-map adaptation. Successful declaration validation does not establish executable availability or scientific compatibility.

`contract/hpc_replay_payload.rs` admits replay payload structure and exact campaign/job identity against each transport sidecar and its paired bundle. It parses plaintext once and returns diagnostics before publication. This projection preserves extensible result fields; it does not replace full domain-result schemas or prove execution, scientific completeness, authenticity of an untrusted sidecar, or crash-atomic pair publication.

`policy/bundle_redaction.rs` applies HPC bundle replacements to string values before JSON encoding. It preserves schema keys and nonstring values, handles escaped quotes/control characters, and retains existing replacement order. Text operator logs remain a distinct format; content redaction does not establish replay or scientific admissibility.

`policy/hpc_submission/` owns campaign scope selection and the ordered selected-job dependency graph. Planning borrows campaign rows and finishes admission before command-side effects. The CLI resolves admitted dependencies against the exact preceding scheduler receipt prefix. Combined stage/tool/sample filters are conjunctive. Scoped commands retain declarations referring to known jobs outside their scope without submitting them; they do not establish that those external prerequisites have executed. Unknown, ambiguous, self and forward selected-job dependencies fail before submission.

`model/hpc_campaign/submission_bundle/` owns the existing results/code submission wire contracts, typed assembly and projections from campaign resources, Slurm settings and output paths. Native schema generation owns both submission schemas. These describe the pending-execution producer format; historical runtime-result extensions and scientific/replay semantics still use their own admission contracts. The process adapter supplies current code-freeze observations; model assembly performs no IO. JSON encoding still passes through the ordered value representation before redaction, preserving existing key order and bytes.

BAM campaign source admission, goal selection, evidence joins and check projection belong to `bijux-dna-bench-model::derive_bam_benchmark_campaign`. Its private `policy/bam_campaign/` groups validation, alignment, filtering, metrics, ancient-DNA, contamination, identity and resource/template goals. `crates/bijux-dna-bench/src/repo/bam_campaign.rs` owns persisted report admission and atomic publication; the benchmark/API facade and CLI invocation remain stable. Goal IDs, scenario/check strings, ordering, error contexts and schemas retain their existing meanings.

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

Stage-tool asset declaration coverage belongs to `contract/stage_tool_assets.rs`. Its governed validator admits the existing FASTQ taxonomy/reference/index and BAM reference/panel bindings over acquired `StageToolAssetRow` values, preserving required identity/path checks and failure order. Generic config/report validation remains separate. The CLI acquires planner/corpus inputs and publishes the existing TOML/report; this pure owner performs no IO or native probe.

`policy/hpc_resources.rs` also owns the governed FASTQ stage/tool and typed VCF stage walltime profiles used by stage-tool resource declarations. The profiles retain specific-tool precedence and fallback budgets in minutes. They are benchmark scheduling hints, not runtime or scientific estimates; planner/domain CPU, memory and scratch constraints and input acquisition remain with their existing owners.
