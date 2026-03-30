# Architecture

## Layout principles
- `src/lib.rs` is a public facade and compatibility layer, not a place for domain implementations.
- Domain subsystems own their own directories so the filesystem matches Rust module ownership.
- Manifest loaders live next to the domain they catalog instead of being embedded inside policy code.
- Stage truth lives under `src/stages/`; compatibility surfaces may re-export it, but they do not redefine it.

## Preferred tree
```text
src/
  artifacts/              stage reports and manifests
  banks/                  adapter, contaminant, and polyX banks
  bench/                  benchmark query context and repository contracts
  execution_support/      execution support models and manifest catalog
  invariants/             invariant specs, evaluation, and thresholds
  metrics/                metric types, specs, and deltas
  observer/
    contracts/            observer specialization catalog
    parse/                governed parser implementations
  params/
    edna/                 amplicon parameter shapes
    processing/           preprocessing and transform parameter shapes
    quality/              QC and filtering parameter shapes
  pipeline_contract/      pipeline catalog and graph assembly
  run/                    FASTQ input discovery and benchmark corpus helpers
  stage_tool_governance/  tool layout catalog and governance policy
  stages/                 authoritative stage IDs, contracts, semantics, and IO
  types/                  shared FASTQ domain types
```

## Ownership map
- `artifacts/` owns governed stage report schemas and manifest contracts.
- `observer/` owns parser behavior and the stage-tool specialization map that explains normalization surfaces.
- `pipeline_contract/` owns stage ordering and dependency graph assembly, but not stage semantics.
- `execution_support/` and `stage_tool_governance/` own manifest-backed readiness/catalog views.
- `bench/` owns benchmark query matching, lineage metadata, and repository contracts.

## Data flow
- Provides FASTQ domain truth to planners, stages, API layers, and benchmark tooling.
- Consumes SSOT manifests under `domain/fastq/` without reaching into execution/runtime behavior.
