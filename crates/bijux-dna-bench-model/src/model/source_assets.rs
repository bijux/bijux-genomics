//! Benchmark input content identities independent of filesystem acquisition.
/// Storage representation of a benchmark input, independent of execution backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkSourceKind {
    /// One declared input file.
    File,
    /// A directory tree, hashed by sorted relative member paths.
    Directory,
    /// Files sharing a declared index prefix followed by a dot.
    PrefixBundle,
}

impl BenchmarkSourceKind {
    /// Stable spelling used in persisted source-asset manifests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::PrefixBundle => "prefix_bundle",
        }
    }
}

/// Content identity of a source input before staging or execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkSourceAsset {
    /// Physical representation of the input.
    pub kind: BenchmarkSourceKind,
    /// Caller-declared path, retained for provenance and staging.
    pub source_path: String,
    /// File SHA-256 or the stable sorted-member digest for compound inputs.
    pub checksum_sha256: String,
    /// Total bytes across regular-file members.
    pub size_bytes: u64,
    /// Number of regular-file members represented by the digest.
    pub member_count: usize,
}
