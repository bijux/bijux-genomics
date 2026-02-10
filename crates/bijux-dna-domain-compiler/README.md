# bijux-dna-domain-compiler

## What this crate does
Compiles authored `domain/**` YAML into generated runtime configs (`configs/tool_registry.toml`, `configs/stages.toml`, `configs/images.toml`) and validates domain schema completeness.
This crate owns the domain-to-config compilation and validation logic used by CLI/tooling.

## What it must not do (boundaries)
It must not execute pipelines, run tools, or perform runtime orchestration.
It must not own benchmark logic or planner execution behavior.

## Effects & determinism guarantees
Compilation is deterministic for the same domain input and scope.
Outputs are written as generated artifacts and are safe to overwrite.

## Public API / entrypoints
- Library: `compile_domain_configs`, `validate_domain`
- Binary: `compile_domain_configs`

## Key contracts it owns/consumes
- Owns generated config contract headers and structure.
- Consumes domain schema and stage/tool compatibility declarations.

## Artifacts / Contracts
- `configs/tool_registry.toml`
- `configs/stages.toml`
- `configs/images.toml`

## Failure modes
- Missing required domain fields.
- Invalid stage/tool references.
- Duplicate IDs or invalid compatibility mappings.

## How to run its tests
- `cargo test -p bijux-dna-domain-compiler`
- `cargo test -p bijux-dna-domain-compiler --test guardrails`
- `cargo test -p bijux-dna-domain-compiler --lib`
- `cargo test -p bijux-dna-domain-compiler --test boundaries`
- `cargo test -p bijux-dna-domain-compiler --test contracts`
- `crates/bijux-dna-domain-compiler/tests/guardrails.rs`
- `crates/bijux-dna-domain-compiler/src/lib.rs`
- `crates/bijux-dna-domain-compiler/src/main.rs`

## Where the docs live
- [docs/INDEX.md](docs/INDEX.md)
- [docs/TESTS.md](docs/TESTS.md)
