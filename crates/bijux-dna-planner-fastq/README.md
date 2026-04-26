# bijux-dna-planner-fastq

`bijux-dna-planner-fastq` is the deterministic FASTQ planner. It selects governed tool bindings, composes `StagePlanV1` values, builds execution graphs, and emits stable explain metadata without running tools or parsing runtime output.

## Repository Policy
This crate is governed by `README.md` and `README.md`. Re-read those files before changing this crate or committing work in this repository.

## Role
- Upstream contracts: `bijux-dna-domain-fastq`, `bijux-dna-stages-fastq`, `bijux-dna-stage-contract`, and `bijux-dna-pipelines`.
- Downstream consumers: runner, engine, CLI, API, and analysis crates.
- Boundary: plan construction only; execution and parsing stay downstream.

## Entry Points
- `src/lib.rs` exports the stable planner API.
- `src/surface.rs` collects root reexports and planner constants.
- `src/stage_api.rs` provides the curated compatibility surface for stage-level planning.
- `src/planner/` builds graph-backed FASTQ plans and benchmark fan-out graphs.
- `src/compose/` binds inputs, outputs, params, and stage dependencies.
- `src/tool_adapters/` builds stage-specific command specs.

## Documentation
Start with `docs/INDEX.md`. Public API, architecture, command inventory, dependency boundaries, effects, determinism, explain output, and test contracts are documented under `docs/`, including `docs/TESTS.md`.

## Validation
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-fastq --no-default-features
```
