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
