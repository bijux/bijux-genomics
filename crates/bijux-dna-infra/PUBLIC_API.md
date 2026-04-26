# bijux-dna-infra Public API

Public module exported from `src/lib.rs`:
- `formats`

Stable root re-exports are grouped by responsibility:
- Hashing: `hash_file_sha256`
- Filesystem IO: `atomic_write_*`, `write_*`, `copy_file`, `read_to_end_bounded`, removal helpers, `IoError`, `IoErrorKind`
- Locking and logging: `FileLock`, `init_logging`
- Paths: bench paths, config paths, run-layout path builders
- Retry: `RetryPolicy`, `Clock`, `SystemClock`, `backoff_delay`, `retry_with`
- Run layout: `RunLayoutContract`, `RunLayoutPaths`, `RUN_LAYOUT_CONTRACT`, `PIPELINE_RUN_DIR_TEMPLATE`, `lock_run`, `publish_run`
- Temp directories: `temp_dir`, `temp_dir_in`
