# bijux-dna-domain-vcf

`bijux-dna-domain-vcf` is the VCF domain contract crate. It owns VCF stage IDs, typed parameter
contracts, metric schemas, downstream taxonomy, invariant checks, panel governance, coverage
reporting, and deterministic registry materialization helpers.

## Responsibilities

- Expose canonical VCF call/filter/stats stage IDs and downstream VCF stage taxonomy.
- Define typed VCF params and schema-versioned metrics.
- Normalize governed retained raw VCF artifacts into shared stage metrics payloads.
- Validate VCF invariants, species context, panel maps, and reference panel governance.
- Render deterministic VCF param and required-tool registry TOML strings.
- Report domain coverage for contract-only, domain-only, and execution-ready surfaces.

## Boundaries

This crate must not execute tools, spawn processes, launch containers, schedule pipelines, open
network connections, write generated configs, or depend on runner/planner/runtime/stage crates.
Generated config writes belong to callers that choose governed output paths.

## Command Surface

This is a library crate with no executable command surface. `docs/COMMANDS.md` is the SSOT for that
boundary.

## Start In Code

- `src/lib.rs` for public catalogs and exports.
- `src/contracts/` for IO, metrics, delivery, panel, and invariant contracts.
- `src/params/` and `src/metrics.rs` for typed payload contracts.
- `src/parsers/` for retained raw artifact normalization into governed metrics payloads.
- `src/taxonomy/` for downstream stage taxonomy and ordering.
- `src/registry_emit.rs` for deterministic registry TOML rendering.

## Verification

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-vcf --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo clippy -p bijux-dna-domain-vcf --all-targets --no-default-features -- -D warnings
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
