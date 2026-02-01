//! Owner: bijux-bench
//! Run repositories for bench.

pub mod run_repo;
pub mod sqlite_run_index;

pub use run_repo::RunRepository;
#[allow(unused_imports)]
pub use sqlite_run_index::RunIndexRepository;
