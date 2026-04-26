# bijux-dna-domain-fastq

`bijux-dna-domain-fastq` is the FASTQ domain contract crate. It owns typed FASTQ stage truth,
artifacts, parameters, metrics, invariants, banks, observer semantics, and benchmark metadata.

## Responsibilities

- Expose stable FASTQ stage IDs, stage contracts, stage IO, semantics, and canonical ordering.
- Define typed report/manifest schemas for governed FASTQ stage outputs.
- Define parameter descriptors, defaults, canonical parsing, metric types, and invariant rules.
- Own adapter, contaminant, and polyX bank models, presets, selection, and provenance.
- Expose observer parser contracts and stage/tool governance metadata without executing tools.
- Provide FASTQ input discovery and benchmark corpus descriptors as domain metadata.

## Boundaries

This crate must not execute pipelines, spawn tools, launch containers, schedule work, open network
connections, write generated configs, or own planner/runtime behavior. It is consumed by planners,
stages, analyzers, API layers, and benchmark tooling; those layers must not be dependencies here.

## Command Surface

This is a library crate with no executable command surface. `docs/COMMANDS.md` is the SSOT for that
boundary.

## Start In Code

- `src/stages/` for stage IDs, semantics, IO, contracts, and ordering.
- `src/params/`, `src/metrics/`, and `src/invariants/` for typed domain behavior.
- `src/banks/` for adapter, contaminant, and polyX bank contracts.
- `src/observer/` for parser contracts and semantic surfaces.
- `src/stage_tool_governance/` and `src/integration_matrix/` for stage/tool compatibility.
- `src/lib.rs` for the public facade and compatibility re-exports.

## Verification

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-fastq --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo clippy -p bijux-dna-domain-fastq --all-targets --no-default-features -- -D warnings
```

## Documentation

- [docs/INDEX.md](docs/INDEX.md)
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/BOUNDARY.md](docs/BOUNDARY.md)
- [docs/COMMANDS.md](docs/COMMANDS.md)
- [docs/CONTRACTS.md](docs/CONTRACTS.md)
- [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)
- [docs/DOMAIN_MODEL.md](docs/DOMAIN_MODEL.md)
- [docs/EFFECTS.md](docs/EFFECTS.md)
- [docs/PUBLIC_API.md](docs/PUBLIC_API.md)
- [docs/TESTS.md](docs/TESTS.md)
