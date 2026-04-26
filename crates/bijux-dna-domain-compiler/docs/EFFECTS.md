# bijux-dna-domain-compiler Effects

The compiler has a narrow side-effect surface: read authored repository data, validate it, and
write declared generated config files when generation is requested.

## Reads

- Domain YAML under the selected `domain_dir`.
- `configs/domain/shared_tools.toml` relative to the workspace root during validation.
- Local Git metadata or domain file content to derive generated source provenance.

## Writes

`compile_domain_configs` may create the selected `configs_dir` and write only the generated output
paths listed in [CONTRACTS.md](CONTRACTS.md). Test-generated outputs must live under the repository
`artifacts/` tree.

`validate_domain` must not write product outputs.

## Forbidden Effects

- Executing tools, pipelines, or containers.
- Opening network connections.
- Scheduling jobs or mutating runtime state.
- Writing undocumented files outside the selected generated config directory.
- Reading planner/runtime state as an input to validation.

## Determinism

For identical domain input and compiler scope, generated output bytes must be stable across
repeated runs.
