# Registry Lock Rules

`tool_registry_lock.sha256` is computed from canonical registry inputs in this exact order:

1. `configs/ci/registry/tool_registry.toml`
2. `configs/ci/registry/tool_registry_experimental.toml`
3. `configs/ci/registry/tool_registry_vcf.toml`
4. `configs/ci/registry/domains.toml`
5. `configs/ci/registry/deprecations.toml`

Computation contract:
- For each input file, compute `sha256(file-bytes)` (hex).
- Build canonical payload lines as `<relative-path><space><sha256>`.
- Join payload lines with `\n` in the order above and append trailing `\n`.
- Compute `sha256(payload)` and write only that hex digest to `tool_registry_lock.sha256`.

Use `scripts/domain/lock-registry.sh` to update the lock deterministically.
