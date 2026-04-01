mod hashing;
mod io;
mod locking;
mod logging;
mod paths;
mod retry;
mod run_directories;
mod temp;

pub mod formats;

// Hashing
pub use hashing::hash_file_sha256;

// Filesystem IO
pub use io::{
    atomic_write_bytes, atomic_write_bytes_with_retry, atomic_write_json, atomic_write_with,
    classify_io_error, create_file, ensure_dir, read_to_end_bounded, remove_dir_all, remove_file,
    remove_file_if_exists, remove_path_if_exists, rename, write_bytes, write_string, IoError,
    IoErrorKind,
};

// Locking and logging
pub use locking::FileLock;
pub use logging::init_logging;

// Path construction
pub use paths::{
    bench_base_dir, bench_data_dir, bench_suites_dir, bench_tools_dir, configs_dir, configs_file,
};

// Retry orchestration
pub use retry::{backoff_delay, retry_with, Clock, RetryPolicy, SystemClock};

// Run layout contracts and operations
pub use run_directories::{
    lock_run, normalize_run_base_dir, pipeline_run_dir, publish_run, run_layout_paths,
    run_stage_dir, RunLayoutContract, RunLayoutPaths, PIPELINE_RUN_DIR_TEMPLATE,
    RUN_LAYOUT_CONTRACT,
};

// Temporary directories
pub use temp::{temp_dir, temp_dir_in};
