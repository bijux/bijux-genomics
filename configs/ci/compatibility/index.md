# configs/ci/compatibility

Purpose: machine-readable compatibility, deprecation, and release-upgrade inputs.

Files:
- `configs/ci/compatibility/deprecations.toml`
- `configs/ci/compatibility/release_changes.toml`

Rule:
- Keep operator-facing compatibility docs generated from these inputs and the governed code registries.
