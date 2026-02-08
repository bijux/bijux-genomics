# PATHS

Path normalization is owned by bijux-core.
Infra can only construct paths via stable run-layout helpers.

## Stability guarantees
- `RUN_LAYOUT_CONTRACT` segment names are stable across releases.
- `run_layout_paths` and `run_stage_dir` only join segments; they do not
  resolve `..` or symlinks.
- `normalize_run_base_dir` only anchors a relative base to a cwd; it does not
  canonicalize or clean the path.

No independent normalization allowed outside bijux-core `contract::canonical`.
