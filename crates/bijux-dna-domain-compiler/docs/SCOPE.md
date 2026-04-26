# bijux-dna-domain-compiler Scope

This crate owns domain compilation and validation. It is upstream of execution crates and downstream
of authored domain YAML.

## In Scope

- Validate FASTQ, BAM, and VCF domain schemas, vocabularies, indexes, stages, tools, and catalogs.
- Compile domain source into governed generated config views under `configs/ci/`.
- Keep generated output order deterministic.
- Preserve planned alternatives in metadata without promoting planned-only tools into governed
  runtime registries.
- Expose the two command binaries documented in [COMMANDS.md](COMMANDS.md).

## Out of Scope

- Runtime execution, planner decisions, pipeline scheduling, API serving, benchmarking, database
  access, and developer control-plane workflows.
- Network calls or container launches.
- Owning runtime config semantics outside the generated files it writes.
- Creating new command surfaces without updating [COMMANDS.md](COMMANDS.md) and boundary tests.

## Default Invocation

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs --scope pre_hpc_pre_vcf
```
