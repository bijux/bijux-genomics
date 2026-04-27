# Retired Container Definitions

Purpose: explicit allowlist for container definition files that remain in-repo but are intentionally retired and no longer mapped from active registries.

Authority surfaces:
- [../README.md](../README.md)
- [../index.md](../index.md)
- [TOOL_LIFECYCLE.md](TOOL_LIFECYCLE.md)
- [PLANNED.md](PLANNED.md)
- [../../configs/ci/registry/tool_registry.toml](../../configs/ci/registry/tool_registry.toml)

Contract:
- Any orphan def/dockerfile must be listed here or CI fails.
- Each entry must include a reason and retirement date.

| Tool | Path | Retired On | Reason |
|---|---|---|---|
