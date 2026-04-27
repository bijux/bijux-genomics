# bijux-dna-science Boundary

## Role

`bijux-dna-science` is the science control plane. It loads authored science specs,
validates cross references, compiles deterministic evidence rows, refreshes governed
science outputs, and cuts immutable science release bundles.

## Owned Inputs

- [science/specs/evidence/README.md](../../../science/specs/evidence/README.md)
  is the authored evidence input boundary for sources, evidences, claims,
  assumptions, reasoning, decisions, and bindings
- [science/specs/releases/README.md](../../../science/specs/releases/README.md)
  is the authored release-manifest input boundary

## Owned Outputs

- [science/generated/README.md](../../../science/generated/README.md) is the
  committed generated-output boundary
- [science/generated/current/README.md](../../../science/generated/current/README.md)
  is the current generated snapshot boundary
- [science/generated/current/evidence/README.md](../../../science/generated/current/evidence/README.md)
  inventories the row-level evidence outputs
- [science/generated/indexes/README.md](../../../science/generated/indexes/README.md)
  inventories the rolled-up JSON entrypoints
- `artifacts/science-releases/<release-id>/**`

## Allowed Effects

- Read authored science specs and governed upstream evidence tables.
- Write deterministic generated science outputs.
- Write immutable release bundles under `artifacts/science-releases/`.
- Print command summaries, trace rows, and closure rows to stdout.

## Forbidden Effects

- No workflow execution.
- No stage orchestration.
- No direct tool launching.
- No container runtime invocation.
- No hidden benchmark, runtime, or network side effects.

## Forbidden Dependencies

This crate must not depend on runner backends, stage executors, pipeline planners, or
pipeline runtime crates. It may consume infra helpers for deterministic file IO and
format parsing, but it must keep runtime execution concerns outside the science
control plane.
