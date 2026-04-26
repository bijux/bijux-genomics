# bijux-dna-domain-fastq Architecture

`bijux-dna-domain-fastq` is a pure FASTQ domain library. It owns typed domain truth and contract
queries; execution crates decide when and how to run tools.

## Layout

```text
src/
  artifacts/              stage report and manifest schemas
  banks/                  adapter, contaminant, and polyX banks plus selection
  bench/                  benchmark query context and repository contracts
  comparison_contract/    stage comparison artifact contracts
  execution_support/      manifest-backed execution support catalog
  integration_matrix/     stage/tool compatibility and benchmark scenarios
  invariants/             invariant specs, thresholds, and evaluation
  metrics/                metric types, classes, specs, and deltas
  observer/               parser contracts and governed parser implementations
  params/                 descriptors, defaults, parsing, and effective params
  pipeline_contract/      pipeline ordering, transitions, and dependency graph
  run/                    FASTQ input discovery and benchmark corpus helpers
  stage_tool_governance/  tool layout, readiness, maturity, and input layout policy
  stages/                 stage IDs, specs, ports, semantics, and contract JSON
  types/                  shared FASTQ domain value types
  lib.rs                  public facade and compatibility re-exports
```

## Ownership Rules

- `stages/` is the authority for FASTQ stage IDs, semantics, IO, and contract JSON.
- `params/`, `metrics/`, and `invariants/` must stay aligned; changing one usually requires tests
  for the others.
- `banks/` owns bank content, preset resolution, provenance, and deterministic hashes.
- `observer/` may parse governed tool reports, but it must not execute tools or own runtime policy.
- `execution_support/`, `integration_matrix/`, and `stage_tool_governance/` expose domain readiness
  and compatibility metadata; they do not schedule execution.
- `run/` may discover FASTQ inputs and benchmark corpus descriptors, but it must remain domain
  discovery, not runner orchestration.

## Data Flow

1. Domain metadata and typed modules define IDs, artifacts, params, metrics, and invariants.
2. Contract query APIs expose stable views to planners, stages, benchmark tooling, and analyzers.
3. Parser and observer helpers normalize governed tool outputs into typed reports.
4. Tests lock parity with `domain/fastq/` manifests, snapshots, and semantic fixtures.
