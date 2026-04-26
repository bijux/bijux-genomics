# Boundary

`bijux-dna-planner-vcf` is the VCF planning boundary. It converts typed VCF domain inputs, reference context, registry declarations, tool choices, and stage contracts into deterministic stage plans, execution graphs, and explain payloads.

## Why this crate exists
The crate gives downstream callers one planner surface for VCF workflows without mixing runtime execution, CLI routing, or tool-output parsing into planning code.

## Allowed Inputs
- `bijux-dna-domain-vcf` stage taxonomy, coverage regimes, and invariant contracts.
- Reference bundle, panel, and map views from `bijux-dna-db-ref`.
- Stage contract types from `bijux-dna-stage-contract`.
- Repository-owned registry files under `configs/ci/` for VCF tools, stages, and params.
- Caller-provided `VcfPipelineInputs` with typed stage overrides.

## Allowed dependencies
- `bijux-dna-core` for graph and plan policy contracts.
- `bijux-dna-domain-vcf` for VCF domain vocabulary and validation.
- `bijux-dna-db-ref` for reference catalog handoff types.
- `bijux-dna-stage-contract` for stage plan payloads.
- Serialization, hashing, and TOML parsing crates needed for deterministic plan assembly.

## Forbidden Dependencies
- Runtime, runner, engine, CLI, API, environment, benchmark, or product execution crates.
- FASTQ or BAM planner and stage crates.
- Network clients or database clients.

## Forbidden Effects
- Process spawning.
- Runtime tool discovery.
- Network access.
- Product execution.
- CLI parsing or command routing.
- Tool-output parsing.
- Generated configuration mutation.

## Validation
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-vcf --no-default-features
```
