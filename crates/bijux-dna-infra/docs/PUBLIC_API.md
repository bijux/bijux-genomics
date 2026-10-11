# bijux-dna-infra Public API

`src/lib.rs` is the public root surface. `src/stable_surface.rs` and each module-level
`stable_surface.rs` own grouped re-exports so the root stays explicit and reviewable.

## Public Modules

- `formats`: JSON, TOML, and optional YAML helpers for config-compatible payloads.

## Stable Root Exports

- Hashing: `hash_bytes_sha256`, `hash_file_sha256`, and deterministic file-or-directory
  `hash_path_sha256`.
- Filesystem IO: `ensure_dir`, `create_file`, `atomic_create_bytes`, `atomic_create_json`,
  `atomic_create_with`, `atomic_write_bytes`, `atomic_write_private_bytes`, `atomic_write_json`, `atomic_write_with`,
  `write_bytes`, `write_string`, `append_line`, `copy_file`, `rename`, `read_to_end_bounded`,
  `remove_file`, `remove_file_if_exists`, `remove_dir_all`, `remove_path_if_exists`, `IoError`,
  `IoErrorKind`, and `classify_io_error`.
- Locking and logging: one-shot `FileLock::try_acquire` and `init_logging`.
- Paths: `bench_base_dir`, `bench_data_dir`, `bench_suites_dir`, `bench_tools_dir`,
  `configs_dir`, `configs_file`, `normalize_run_base_dir`, `pipeline_run_dir`,
  `run_layout_paths`, and `run_stage_dir`.
- Retry planning: effect-free `RetryPolicy` and `backoff_delay`. A retry is a new immutable attempt
  allocated by the owning runtime on a later lifecycle invocation; infra does not sleep or execute it.
- Run layout: `RunLayoutContract`, `RunLayoutPaths`, `RUN_LAYOUT_CONTRACT`,
  `PIPELINE_RUN_DIR_TEMPLATE`, `try_lock_run`, and `publish_run`.
- Attempt layout: `AttemptLayoutIdentity`, `AttemptLayoutPaths`, `AttemptLayoutManifestV1`,
  `AttemptEvidenceOwner`, `ATTEMPT_LAYOUT_CONTRACT`, `ATTEMPT_LAYOUT_SCHEMA_V1`,
  `attempt_layout_paths`, `attempt_layout_schema`, `ensure_attempt_layout`, and
  `validate_attempt_owner_roots`.
- Attempt archive: `AttemptArchiveDisposition`, `AttemptArchiveReceiptV1`,
  `AttemptArchiveRollbackReceiptV1`, `attempt_archive_paths`, `archive_attempt_layout`,
  `verify_attempt_archive`, `rollback_attempt_archive`, `attempt_archive_schema`, and
  `attempt_archive_rollback_schema`.
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

`atomic_write_private_bytes` retains private temporary-file permissions through atomic replacement: Unix plaintext files are published as mode `0600`, even when replacing a file with broader permissions. The general atomic writer continues to preserve existing permissions. Callers own destination-directory admission, ACL policy on other platforms, and transactions involving multiple files.
