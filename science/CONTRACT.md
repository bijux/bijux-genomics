# Science Contract

## Review-Owned Inputs

Only `science/specs/**` is review-authored input.

## Generated Outputs

Only build commands may write:

- `science/generated/**`
- `artifacts/science-releases/**`

Generated science outputs are never hand-edited.

## Planes

- `science/specs/data/` records authored data-plane declarations
- `science/specs/evidence/` records authored sources, evidences, claims, assumptions, reasoning,
  decisions, and bindings
- `science/specs/results/` records authored result-plane specifications
- `science/specs/reports/` records authored report-plane specifications
- `science/specs/releases/` records authored release manifests

## Cross-Plane Rules

- reports are consumers, not truth sources
- assumptions belong under evidence
- findings belong under results
- cross-plane links use typed IDs

## Initial Scope

The first generated slice covers FASTQ environment and container governance. It compiles the
relationship among:

- `domain/fastq/execution_support.yaml`
- `domain/fastq/stages/**`
- `domain/fastq/tools/**`
- `domain/fastq/docs/DEFAULT_SETTINGS.md`
- `configs/ci/registry/tool_registry.toml`
- `crates/bijux-dna-environment/docs/ENV_REFERENCE.md`
