# bijux-dna-domain-compiler

`bijux-dna-domain-compiler` validates authored domain metadata and compiles it into deterministic
generated config views. It is the bridge from `domain/` YAML into governed `configs/ci/` TOML.

## Responsibilities

- Validate FASTQ, BAM, and VCF domain schemas, indexes, vocabularies, stages, tools, and catalogs.
- Compile active domain scope metadata into generated tool, stage, image, and required-tool views.
- Keep generated output stable for identical input and scope.
- Preserve planned alternatives as metadata without promoting planned-only tools into governed
  runtime registries.
- Own the four command binaries listed in [docs/COMMANDS.md](docs/COMMANDS.md).

## Boundaries

This crate must not execute pipelines, run bioinformatics tools, launch containers, schedule work,
open network connections, or own planner/runtime behavior. Execution-facing crates consume generated
outputs; they are not compiler dependencies.

## Public API

- `compile_domain_configs`
- `validate_domain`
- `domain_coverage_report`
- `build_domain_registry_bundle`
- `load_domain_registry_bundle`
- `write_domain_registry_bundle`
- `query_domain_registry_bundle`
- `CompileOptions`
- `ValidateOptions`
- `DEFAULT_DOMAIN_DIR`
- `DEFAULT_CONFIGS_DIR`
- `DEFAULT_COMPILE_SCOPE`

See [docs/PUBLIC_API.md](docs/PUBLIC_API.md) for signatures and stability rules.

## Generated Outputs

`compile_domain_configs` writes the generated files listed in [docs/CONTRACTS.md](docs/CONTRACTS.md):

- `ci/registry/tool_registry.toml`
- `ci/registry/tool_registry_experimental.toml`
- `ci/registry/tool_registry_vcf.toml`
- `ci/registry/domain_registry_release_bundle.json`
- `ci/registry/domain_defaults_snapshot.json`
- `ci/registry/domain_artifact_contract_snapshots.json`
- `ci/registry/domain_metric_catalogs.json`
- `ci/registry/domain_deprecations_snapshot.json`
- `ci/registry/domain_invariant_catalogs.json`
- `ci/registry/domain_evidence_contracts.json`
- `ci/stages/stages.toml`
- `ci/stages/stages_vcf.toml`
- `ci/tools/images.toml`
- `ci/tools/required_tools.toml`

## Verification

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-compiler --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo clippy -p bijux-dna-domain-compiler --all-targets --no-default-features -- -D warnings
```

## Documentation

- [docs/INDEX.md](docs/INDEX.md)
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/BOUNDARY.md](docs/BOUNDARY.md)
- [docs/COMMANDS.md](docs/COMMANDS.md)
- [docs/CONTRACTS.md](docs/CONTRACTS.md)
- [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)
- [docs/EFFECTS.md](docs/EFFECTS.md)
- [docs/PUBLIC_API.md](docs/PUBLIC_API.md)
- [docs/SCOPE.md](docs/SCOPE.md)
- [docs/TESTS.md](docs/TESTS.md)
