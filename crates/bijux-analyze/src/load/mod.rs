//! Owner: bijux-analyze
//! Load facts and run artifacts from disk.

pub mod facts;
pub mod run_index;
pub mod run_summary;
pub mod sqlite;

pub use facts::*;
pub use run_index::*;
pub use run_summary::*;
pub use sqlite::*;

#[derive(thiserror::Error, Debug)]
pub enum AnalyzeError {
    #[error("missing file: {path}")]
    MissingFile { path: String },
    #[error("invalid schema version: {found} (expected {expected})")]
    InvalidSchemaVersion { found: String, expected: String },
    #[error("invalid jsonl row at line {line}: {message}")]
    InvalidJsonlRow { line: usize, message: String },
    #[error("invalid json: {message}")]
    InvalidJson { message: String },
    #[error("unsupported parquet: {message}")]
    UnsupportedParquet { message: String },
}
