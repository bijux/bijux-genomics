# Configs SSOT Flow

This directory contains generated and hand-edited configuration files.

## Canonical generator
- Use one canonical command flow:
  - `scripts/generate-configs.sh`
  - internally runs `cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs`

## Generated files (do not hand edit)
- `tool_registry.toml`
- `tool_registry_experimental.toml`
- `tool_registry_vcf.toml`
- `stages.toml`
- `stages_vcf.toml`
- `required_tools.toml`
- `required_tools_vcf.toml`

These must contain the generated header marker.

## Hand-edited files
- `images.toml`
- `platforms.toml`
- `coverage.toml`
- `profile.local.toml`
- `domains.toml` (domain metadata and SSOT pointers)

## Contracts
- SSOT and completeness rules are enforced by `bijux-dna-policies` contract tests.
- See `docs/CONTRACT_AUTHORITY.md` for authority definitions.
