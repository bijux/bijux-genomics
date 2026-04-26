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
pub use stable_surface::hash_file_sha256;
pub use stable_surface::init_logging;
pub use stable_surface::FileLock;
pub use stable_surface::{
    append_line, atomic_write_bytes, atomic_write_bytes_with_retry, atomic_write_json,
    atomic_write_with, classify_io_error, create_file, ensure_dir, read_to_end_bounded,
    remove_dir_all, remove_file, remove_file_if_exists, remove_path_if_exists, rename, write_bytes,
    write_string, IoError, IoErrorKind,
};
pub use stable_surface::{backoff_delay, retry_with, Clock, RetryPolicy, SystemClock};
pub use stable_surface::{
    bench_base_dir, bench_data_dir, bench_suites_dir, bench_tools_dir, configs_dir, configs_file,
};
pub use stable_surface::{
    lock_run, normalize_run_base_dir, pipeline_run_dir, publish_run, run_layout_paths,
    run_stage_dir, RunLayoutContract, RunLayoutPaths, PIPELINE_RUN_DIR_TEMPLATE,
    RUN_LAYOUT_CONTRACT,
};
pub use stable_surface::{temp_dir, temp_dir_in};
