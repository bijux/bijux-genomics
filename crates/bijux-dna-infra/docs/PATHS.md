# PATHS

Path normalization is owned by bijux-dna-core.
Infra can only construct paths via stable path helpers.

## Stability guarantees
- `RUN_LAYOUT_CONTRACT` segment names are stable across releases.
- `paths::run_layout_paths` and `paths::run_stage_dir` only join segments; they do not
  resolve `..` or symlinks.
- `paths::normalize_run_base_dir` only anchors a relative base to a cwd; it does not
  canonicalize or clean the path.
- `configs_file` only applies documented alias remapping before joining under `configs/`.

No independent normalization allowed outside bijux-dna-core `contract::canonical`.
