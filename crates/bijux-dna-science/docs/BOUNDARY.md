# bijux-dna-science Boundary

## Role

`bijux-dna-science` is the science control plane. It loads authored science specs,
validates cross references, compiles deterministic evidence rows, refreshes governed
science outputs, and cuts immutable science release bundles.

## Owned Inputs

- `science/specs/evidence/sources/**`
- `science/specs/evidence/evidences/**`
- `science/specs/evidence/claims/**`
- `science/specs/evidence/assumptions/**`
- `science/specs/evidence/reasoning/**`
- `science/specs/evidence/decisions/**`
- `science/specs/evidence/bindings/**`
- `science/specs/releases/manifests/**`

## Owned Outputs

- `science/generated/current/evidence/**`
- `science/generated/indexes/science_index.json`
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

