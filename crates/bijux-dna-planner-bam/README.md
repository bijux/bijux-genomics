# bijux-dna-planner-bam

Repository policy: apply `README.md` and `README.md` before changing this crate.

## What this crate does
BAM planner: selects tools, assembles stage plans, emits BAM execution graphs, and produces explain payloads. It plans commands; it does not execute them.

## Stage group ownership
- **Pre-alignment and filtering**: validation, alignment, QC, mapping summaries, filtering, and overlap correction.
- **Post-alignment QC**: duplicate marking, complexity, coverage, insert-size, GC-bias, endogenous-content, and recalibration planning.
- **Ancient-DNA analysis**: damage, authenticity, contamination, and sex inference planning.
- **Downstream analysis**: optional feature-gated genotyping, haplogroup, kinship, and bias-mitigation planning.

Planner owns selection and graph construction. Domain, stage, and pipeline crates own stage IDs, artifact contracts, metrics, params, and profile stage ordering.

## Explainability guarantee
Explain output includes defaults diff, reasons for tool selection, and contract hashes.
See `docs/EXPLAIN_OUTPUT.md`.

## What it must not do (boundaries)
No runtime command routing, CLI parsing, process spawning, network access, tool-output parsing, or product execution.

## Role in the stack
Upstream: BAM domain, BAM stages, stage-contract, core, and pipelines. Downstream: engine, runner, API, CLI, and analysis applications.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PUBLIC_API.md`, `docs/ARCHITECTURE.md`, `docs/COMMANDS.md`, and `docs/EXPLAIN_OUTPUT.md`.

## Key contracts it owns/consumes
Owns plan JSON, explain payloads, deterministic graph assembly, tool selection, and planned command specs. Consumes upstream domain/stage/pipeline contracts.

## Artifacts / Contracts
See `docs/ARCHITECTURE.md`, `docs/EXPLAIN_OUTPUT.md`, `docs/COMMANDS.md`, and snapshots under `tests/snapshots/`.

## Effects & determinism guarantees
Pure planning with deterministic ordering and hashes. See `docs/DETERMINISM.md` and the contract tests below.

## How to run its tests
See `docs/TESTS.md`. Standard command:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-bam --no-default-features
```

## Start here in code
- `src/lib.rs` for public planning entrypoints.
- `src/api.rs` for request/config structs and `stage_api`.
- `src/profile_catalog.rs` for supported BAM pipeline profiles.
- `src/selection/` for tool registry and default-tool selection.
- `src/stage_dispatch/` for stage-family dispatch.
- `src/tool_adapters/` for planned command spec builders.

## Where the docs live
Start at `docs/INDEX.md`. The crate root intentionally keeps only this `README.md`; all other Markdown docs belong under `docs/`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/ARCHITECTURE.md`, `docs/COMMANDS.md`, and `docs/EXPLAIN_OUTPUT.md`.
