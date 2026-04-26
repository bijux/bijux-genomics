# bijux-dna-infra Public API

`src/lib.rs` is the public root surface. `src/stable_surface.rs` and each module-level
`stable_surface.rs` own grouped re-exports so the root stays explicit and reviewable.

## Public Modules

- `formats`: JSON, TOML, and optional YAML helpers for config-compatible payloads.

## Stable Root Exports

- Hashing: `hash_file_sha256`.
- Filesystem IO: `ensure_dir`, `create_file`, `atomic_write_bytes`, `atomic_write_json`,
  `atomic_write_with`, `atomic_write_bytes_with_retry`, `write_bytes`, `write_string`,
  `append_line`, `copy_file`, `rename`, `read_to_end_bounded`, `remove_file`,
  `remove_file_if_exists`, `remove_dir_all`, `remove_path_if_exists`, `IoError`, `IoErrorKind`,
  and `classify_io_error`.
- Locking and logging: `FileLock` and `init_logging`.
- Paths: `bench_base_dir`, `bench_data_dir`, `bench_suites_dir`, `bench_tools_dir`,
  `configs_dir`, `configs_file`, `normalize_run_base_dir`, `pipeline_run_dir`,
  `run_layout_paths`, and `run_stage_dir`.
- Retry: `RetryPolicy`, `Clock`, `SystemClock`, `backoff_delay`, and `retry_with`.
- Run layout: `RunLayoutContract`, `RunLayoutPaths`, `RUN_LAYOUT_CONTRACT`,
  `PIPELINE_RUN_DIR_TEMPLATE`, `lock_run`, and `publish_run`.
- Temp directories: `temp_dir` and `temp_dir_in`.

## Stability Rules

- New root exports require this file and the public surface snapshot to change together.
- New format helpers must remain config-compatible and must not claim contract canonicalization.
- New path helpers must preserve the `bijux-dna-core` canonicalization boundary documented in
  `PATHS.md`.
- New command or process surfaces are forbidden unless `COMMANDS.md`, `BOUNDARY.md`, and boundary
  tests change in the same commit.
- Breaking behavior changes require explicit approval, focused contract tests, and snapshot updates
  where the public surface changes.
- Internal helpers are not stable unless they are re-exported from `src/lib.rs`.
