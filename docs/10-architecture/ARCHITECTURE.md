# Architecture

## SSOT Rule
Domain is the authored SSOT; configs are generated; code consumes generated configs; makefiles call CLI only.

Generated config set is fixed and compiler-owned: `configs/tool_registry.toml`, `configs/stages.toml`, and `configs/images.toml`.
