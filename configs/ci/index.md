# configs/ci

## What
Configuration files for CI policy gates, SSOT registry validation, and contract authority.

## Philosophy
Keep CI-critical configuration scoped to this directory so ownership is explicit and drift is easy to detect.

## CI Tiers And Control Files
- Fast tier: `domains.toml`, `stages.toml`, `tool_registry.toml`, `required_tools.toml`, `param_registry.toml`.
- VCF tier: `stages_vcf.toml`, `tool_registry_vcf.toml`, `required_tools_vcf.toml`, `param_registry_vcf.toml`.
- Release/pinning tier: `tool_registry.lock.sha256`, `tool_registry_experimental.toml`.
- Image parity tier: `images.toml`.
