# Public API

This crate has one stable public namespace:

```rust
pub mod v1;
```

Everything else in `src/lib.rs` is crate-private wiring. Public callers should
enter through `bijux_dna_api::v1::api`.

## Front Door Exports

`src/v1/api/front_door.rs` exports:

- Operations: `plan`, `execute`, `execute_and_report`, `dry_run`, `status`,
  `pause_run`, `resume_run`, `cancel_run`, `operator_health`, `explain`,
  `policy_audit`, `render_report`, `render_report_bundle_html`,
  `workspace_edges`, and `write_workspace_audit`.
- Contract types: `PlanRequest`, `PlanResponse`, `ExecuteRequest`,
  `ExecuteResponse`, `DryRunRequest`, `DryRunResponse`, `RunStatus`,
  `RunControlResponse`, `OperatorHealthResponse`, `RenderReportRequest`,
  `RenderReportResult`, `ExplainResponse`, `ExplainToolSelection`,
  `PlanExplainV1`, and `VcfRunRequest`.
- Curated helper namespaces: `bench`, `plan`, `run`, `report`, `bam`, `fastq`,
  `env`, `qualification`, and `shared`.
- The `qualification` namespace exposes provisional composition and explicit
  `derive_container_qualification_report_with_release` composition for immutable release
  publications outside the repository worktree.
- The `bench` namespace composes benchmark-owned corpus materialization through
  `materialize_fastq_corpus(CorpusMaterializationRequest)` without moving acquisition IO
  into the API or CLI crates.
- The `bench` namespace composes runner-owned corpus materialization Slurm preparation,
  submission, one-shot observation, and terminal collection without owning scheduler state.
- The `bench` namespace exposes `BenchmarkStagePluginComposition` and
  `validate_benchmark_stage_plugin_composition` so application entrypoints can
  prove exact, non-ambiguous FASTQ/BAM/VCF plugin ownership before lifecycle
  effects.
- The `bench` namespace exposes the model-owned FASTQ, BAM, and VCF comparable-metric
  readiness contracts, typed status and direction vocabularies, schema generators, and
  fail-closed validators used by thin CLI renderers.
- The `bench` namespace exposes model-owned stage-, tool-, and corpus-centric readiness
  view contracts, schemas, and fail-closed aggregate validators.
- The `bench` namespace exposes the model-owned stage/tool benchmark-readiness envelope,
  generated schema, and fail-closed validator used by the thin CLI renderer.
- The `bench` namespace exposes model-owned VCF stage-matrix declarations, render reports,
  validation reports, generated schemas, and fail-closed validators.
- The `bench` namespace exposes model-owned VCF stage-catalog declarations, render reports,
  generated schemas, and fail-closed validators.
- The `bench` namespace exposes benchmark-owned local VCF catalog/matrix derivation, rendering,
  and strict validation workflows used by thin CLI adapters.
- The `bench` namespace exposes model-owned all-domain stage/tool scope, retained-tool,
  and registry-drift report contracts with explicit licensed-binding retention.
- The `bench` namespace exposes benchmark-owned stage/tool closure derivation and checked-in run
  registry publication so thin application entrypoints do not depend on `bijux-dna-bench`
  directly.
- The `bench` namespace exposes the model-owned all-domain parser-collector report,
  including exact real-smoke coverage and generated schema validation.
- The `bench` namespace exposes the model-owned Goal 211-229 local VCF smoke-suite
  record, generated schema, and fail-closed aggregate validator.

## Export Policy

- Export only stable, versioned APIs from `v1`.
- Prefer narrow re-exports over exposing complete lower-level crates.
- Keep internal handler, runtime, support, and surface modules crate-private.
- Add new public operations to `docs/COMMANDS.md` in the same change.
- Add or update schema snapshots for any public shape change.
- Keep `PlanRequest` and `PlanResponse` aligned with the governed workflow and
  plan manifest contracts exported from `bijux-dna-core`.
- Keep `ExecuteRequest`, `ExecuteResponse`, `DryRunResponse`, and `RunStatus`
  aligned with the governed run-state, runtime-policy, executor-descriptor,
  checkpoint, and failure contracts exported from `bijux-dna-runtime`.
- Keep `RunControlResponse` and `OperatorHealthResponse` aligned with the
  governed run-control and operator-health contracts exported from
  `bijux-dna-runtime`.
- `execute` now distinguishes `simulation`, `advisory`, and `enforced` modes;
  `dry-run` remains a separate public operation with its own response contract.

## Non-Public Surfaces

The following modules are intentionally private implementation detail:

- `src/internal/`
- `src/runtime/`
- `src/support/`
- `src/surface/`

They may change when the v1 public contract remains stable.

## Stability Tiers

- Stable: `pub mod v1`, `bijux_dna_api::v1::api`, and the contract types/operations documented above.
- `BenchmarkStagePluginComposition` is the stable v1 application composition for exact
  FASTQ/BAM/VCF plugin ownership; adding an implementation extends its canonical binding input
  without changing the generic composition API.
- Experimental: no experimental public namespace is exported today; any future one must be called out explicitly here before use.
- Internal: `src/internal/`, `src/runtime/`, `src/support/`, `src/surface/`, and any item not re-exported through `v1::api`.

The benchmark API re-exports normalized FASTQ corpus discovery and publication metadata services from `bijux-dna-bench`, including `NormalizedFastqSample`. This metadata contract is separate from governed data-index content admission.

The `v1::api::bench` facade exposes `BenchmarkLocalStageInventoryDomain`, `load_benchmark_local_stage_inventory` and `render_benchmark_all_domain_stage_inventory`. It delegates explicit-root matrix admission and validated aggregate publication to the benchmark owner; persisted inventory schema identities remain unchanged.

`v1::api::env::delivery` exposes container-owned supported-tool selection and external-artifact declaration admission. The API forwards the canonical delivery view and contract records without adding another parser or projection authority.

## Workspace Dependency Reports

`read_workspace_crate_manifests` returns the named manifests declared by a root
workspace in crate-name order through `WorkspaceCrateManifest`. It preserves
caller-supplied manifest paths and the existing manifest read/parse diagnostics.

`workspace_edges_at` resolves Cargo metadata from an explicit root and rejects
metadata whose canonical workspace root differs. Relative paths and symlink
aliases are accepted; a member directory resolved to its parent workspace is
rejected. `workspace_edges` retains its existing current-directory contract.
Both methods retain Cargo metadata's default dependency kinds, feature resolution,
and strict output/error decoding.

`write_workspace_dependency_map` returns and atomically publishes
`CrateDependencyMapReport`, with `CrateDependencyNode` and `CrateDependencyEdge`,
using the existing `bijux.crates.dependency_map.v1` schema, field names, ordering,
and caller-relative path rendering. This service owns report composition and
publication; it does not implement cycle-admission policy. Cargo metadata retains
its existing lockfile and dependency-resolution effects.

## Image QA Compatibility Hook

`v1::api::env::run_image_qa` retains a compatibility success result without
loading a platform, probing tools, or publishing QA evidence. Its optional
platform argument is not validated. A successful return does not certify an
image or tool. Explicit QA execution and pass-record validation belong to
`bijux-dna-containers-qa`; its `image_qa` binary is the image QA entrypoint.
