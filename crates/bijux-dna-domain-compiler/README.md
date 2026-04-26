# bijux-dna-domain-compiler

## What this crate does
Compiles authored `domain/` YAML into generated runtime configs (`configs/ci/registry/tool_registry.toml`, `configs/ci/stages/stages.toml`, `configs/ci/tools/images.toml`) and validates domain schema completeness.
This crate owns the domain-to-config compilation and validation logic used by CLI/tooling.

## What it must not do (boundaries)
It must not execute pipelines, run tools, or perform runtime orchestration.
It must not own benchmark logic or planner execution behavior.

## Effects & determinism guarantees
Compilation is deterministic for the same domain input and scope.
Outputs are written as generated artifacts and are safe to overwrite.

## Public API / entrypoints
- Library: `compile_domain_configs`, `validate_domain`
- Library: `domain_coverage_report`
- Binaries: `compile_domain_configs`, `domain_validate`
- Command SSOT: [docs/COMMANDS.md](docs/COMMANDS.md)

## Key contracts it owns/consumes
- Owns generated config contract headers and structure.
- Consumes domain schema and stage/tool compatibility declarations.

## Artifacts / Contracts
- `configs/ci/registry/tool_registry.toml`
- `configs/ci/stages/stages.toml`
- `configs/ci/tools/images.toml`

## Failure modes
- Missing required domain fields.
- Invalid stage/tool references.
- Duplicate IDs or invalid compatibility mappings.

## How to run its tests
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-compiler --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-compiler --no-default-features --test boundaries`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo clippy -p bijux-dna-domain-compiler --all-targets --no-default-features -- -D warnings`

Primary test files:
- `tests/boundaries.rs`
- `tests/guardrails.rs`
- `tests/determinism_generated_outputs.rs`
- `tests/planned_tool_registry_boundaries.rs`

## Where the docs live
- [docs/INDEX.md](docs/INDEX.md)
- [docs/BOUNDARY.md](docs/BOUNDARY.md)
- [docs/COMMANDS.md](docs/COMMANDS.md)
- [docs/CONTRACTS.md](docs/CONTRACTS.md)
- [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)
- [docs/PUBLIC_API.md](docs/PUBLIC_API.md)
- [docs/TESTS.md](docs/TESTS.md)
