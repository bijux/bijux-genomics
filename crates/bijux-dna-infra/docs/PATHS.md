# PATHS

Semantic path normalization is owned by bijux-dna-core.
Infra constructs paths through stable helpers and admits physical evidence-root custody.

## Stability guarantees
- `RUN_LAYOUT_CONTRACT` segment names are stable across releases.
- `paths::run_layout_paths` and `paths::run_stage_dir` only join segments; they do not
  resolve `..` or symlinks.
- `paths::normalize_run_base_dir` only anchors a relative base to a cwd; it does not
  canonicalize or clean the path.
- `configs_file` only applies documented alias remapping before joining under `configs/`.
- `attempt_layout_paths` rejects unsafe identity segments instead of silently normalizing them and
  preserves the permanent run/tool/campaign/attempt hierarchy under an owner-provided root.
- `validate_attempt_owner_roots` refuses equal or nested container and benchmark evidence roots.
- Attempt archives and rollback receipts remain under hidden owner-local roots and preserve the
  permanent run/tool/campaign/attempt identity without crossing container/benchmark ownership.

`validate_attempt_owner_roots` resolves an existing ancestor and retains the missing path suffix
before checking owner-root equality or nesting. That physical alias/overlap check stays in infra;
it does not change domain identifiers, canonical JSON or the lexical construction guarantees above.
Semantic contract canonicalization remains in bijux-dna-core `contract::canonical`.
