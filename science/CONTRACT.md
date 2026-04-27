# Science Contract

[science/README.md](README.md) is the operator-facing entrypoint for this
science control surface.

## Review-Owned Inputs

Only authored specs under these governed surfaces are review-authored input:

- [science/specs/data/README.md](specs/data/README.md)
- [science/specs/evidence/README.md](specs/evidence/README.md)
- [science/specs/results/README.md](specs/results/README.md)
- [science/specs/reports/README.md](specs/reports/README.md)
- [science/specs/releases/README.md](specs/releases/README.md)

## Generated Outputs

Only build commands may write:

- [science/generated/README.md](generated/README.md)
- [science/generated/current/README.md](generated/current/README.md)
- [science/generated/current/evidence/README.md](generated/current/evidence/README.md)
- [science/generated/indexes/README.md](generated/indexes/README.md)
- `artifacts/science-releases/**`

Generated science outputs are never hand-edited.

[science/docs/README.md](docs/README.md) is not a generated surface. It is the
local manually managed archive for evidence payloads that should not be
committed.

## Planes

- [science/specs/data/README.md](specs/data/README.md) records authored
  data-plane declarations
- [science/specs/evidence/README.md](specs/evidence/README.md) records authored
  sources, evidences, claims, assumptions, reasoning, decisions, and bindings
- [science/specs/results/README.md](specs/results/README.md) records authored
  result-plane specifications
- [science/specs/reports/README.md](specs/reports/README.md) records authored
  report-plane specifications
- [science/specs/releases/README.md](specs/releases/README.md) records authored
  release manifests

## Cross-Plane Rules

- reports are consumers, not truth sources
- assumptions belong under evidence
- findings belong under results
- cross-plane links use typed IDs
- science source records may point at planned local archive paths under
  `science/docs/**`, but those payloads are not review-authored truth by
  themselves
- local archive payloads must support authored source records rather than replace
  them

## Initial Scope

The first generated slice covers FASTQ environment and container governance. It compiles the
relationship among:

- [domain/fastq/execution_support.yaml](../domain/fastq/execution_support.yaml)
- stage and tool manifests under `domain/fastq/stages/**` and
  `domain/fastq/tools/**`
- [domain/fastq/docs/DEFAULT_SETTINGS.md](../domain/fastq/docs/DEFAULT_SETTINGS.md)
- [configs/ci/registry/tool_registry.toml](../configs/ci/registry/tool_registry.toml)
- [crates/bijux-dna-environment/docs/ENV_REFERENCE.md](../crates/bijux-dna-environment/docs/ENV_REFERENCE.md)
- [science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv](docs/upstream/fastq/tools/EVIDENCE_MAP.tsv)
- [science/docs/upstream/papers/TOOL_PAPER_MAP.tsv](docs/upstream/papers/TOOL_PAPER_MAP.tsv)
