# bijux-dna-science Schemas

## Authored Specs

Authored YAML specs live under
[science/specs/evidence/README.md](../../../science/specs/evidence/README.md)
and [science/specs/releases/README.md](../../../science/specs/releases/README.md).
Those authored surfaces declare explicit schema versions for sources, evidence,
claims, assumptions, reasoning, decisions, bindings, and releases.

Accepted schema constants are exported from `schema`:

- `bijux.science.source.v1`
- `bijux.science.evidence.v1`
- `bijux.science.claim.v1`
- `bijux.science.assumption.v1`
- `bijux.science.reasoning.v1`
- `bijux.science.decision.v1`
- `bijux.science.binding.v1`
- `bijux.science.release.v1`

## Compiled Model

`compile::compile_workspace` loads authored specs, validates cross references, and
derives generated science rows for source inventories, archive gaps, claim evidence,
decision reasoning, binding resolution, FASTQ container references, download backlog,
paper archive coverage, environment rows, closure gates, truth deltas, missing
closure prerequisites, default binding risk, unresolved references, and the science
index.

## Generated Evidence

Generated TSV and JSON outputs are committed under
[science/generated/README.md](../../../science/generated/README.md) only when
they match the compiled model exactly. The row-level ledger inventory is
documented by
[science/generated/current/evidence/README.md](../../../science/generated/current/evidence/README.md),
and the rolled-up JSON entrypoints are documented by
[science/generated/indexes/README.md](../../../science/generated/indexes/README.md).
