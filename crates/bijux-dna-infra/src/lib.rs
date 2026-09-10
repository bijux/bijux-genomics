mod hashing;
mod io;
mod locking;
mod logging;
mod paths;
mod retry;
mod run_directories;
mod stable_surface;
mod temp;

pub mod formats;
pub use stable_surface::init_logging;
pub use stable_surface::FileLock;
pub use stable_surface::{
    append_line, atomic_create_bytes, atomic_create_json, atomic_create_with, atomic_write_bytes,
    atomic_write_json, atomic_write_with, classify_io_error, copy_file, create_file, ensure_dir,
    read_to_end_bounded, remove_dir_all, remove_empty_dir, remove_file, remove_file_if_exists,
    remove_path_if_exists, rename, same_filesystem, write_bytes, write_payload, write_string,
    IoError, IoErrorKind,
};
pub use stable_surface::{
    archive_attempt_layout, attempt_archive_paths, attempt_archive_rollback_schema,
    attempt_archive_schema, attempt_layout_paths, attempt_layout_schema, ensure_attempt_layout,
    normalize_run_base_dir, pipeline_run_dir, publish_run, rollback_attempt_archive,
    run_layout_paths, run_stage_dir, try_lock_run, validate_attempt_owner_roots,
    verify_attempt_archive, AttemptArchiveDisposition, AttemptArchiveFileV1, AttemptArchivePaths,
    AttemptArchiveReceiptV1, AttemptArchiveRollbackReceiptV1, AttemptEvidenceOwner,
    AttemptLayoutContract, AttemptLayoutDirectoriesV1, AttemptLayoutIdentity,
    AttemptLayoutManifestV1, AttemptLayoutPaths, RunLayoutContract, RunLayoutPaths,
    ATTEMPT_ARCHIVE_ROLLBACK_SCHEMA_V1, ATTEMPT_ARCHIVE_SCHEMA_V1, ATTEMPT_LAYOUT_CONTRACT,
    ATTEMPT_LAYOUT_SCHEMA_V1, PIPELINE_RUN_DIR_TEMPLATE, RUN_LAYOUT_CONTRACT,
};
pub use stable_surface::{backoff_delay, RetryPolicy};
pub use stable_surface::{
    bench_base_dir, bench_data_dir, bench_suites_dir, bench_tools_dir, configs_dir, configs_file,
};
pub use stable_surface::{hash_bytes_sha256, hash_file_sha256, hash_path_sha256};
pub use stable_surface::{private_temp_file_in, temp_dir, temp_dir_in};
