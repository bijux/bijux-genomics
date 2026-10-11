//! Owner: bijux-dna-bench-corpus
//! FASTQ corpus model ownership.
use super::{Deserialize, PathBuf, Serialize};

pub const DEFAULT_CORPUS_01_MINI_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-mini/manifest.toml";

pub const DEFAULT_CORPUS_01_VCF_COHORT_FASTQ_MINI_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-vcf-cohort-fastq-mini/manifest.toml";

pub const FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.fastq_corpus_fixture.v1";

pub const FASTQ_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.fastq_corpus_fixture_validation.v1";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FastqCorpusFixtureCompression {
    Gzip,
}

impl FastqCorpusFixtureCompression {
    pub(crate) fn expected_suffix(self) -> &'static str {
        match self {
            Self::Gzip => ".fastq.gz",
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Gzip => "gzip",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FastqCorpusFixtureLayout {
    Se,
    Pe,
}

impl FastqCorpusFixtureLayout {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Se => "se",
            Self::Pe => "pe",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqCorpusFixtureManifest {
    pub schema_version: String,
    pub corpus_id: String,
    pub species: String,
    pub description: String,
    pub compression: FastqCorpusFixtureCompression,
    #[serde(default)]
    pub reference_id: Option<String>,
    pub samples: Vec<FastqCorpusFixtureSample>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqCorpusFixtureSample {
    pub sample_id: String,
    pub cohort: String,
    pub layout: FastqCorpusFixtureLayout,
    pub r1_path: PathBuf,
    pub r2_path: Option<PathBuf>,
    pub expected_read_count_r1: u64,
    pub expected_read_count_r2: Option<u64>,
    pub expected_read_count_total: u64,
    pub source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FastqCorpusFixtureSampleValidationReport {
    pub sample_id: String,
    pub cohort: String,
    pub layout: String,
    pub r1_path: String,
    pub r2_path: Option<String>,
    pub source_paths: Vec<String>,
    pub observed_read_count_r1: u64,
    pub observed_read_count_r2: Option<u64>,
    pub observed_read_count_total: u64,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FastqCorpusFixtureValidationReport {
    pub schema_version: &'static str,
    pub manifest_path: String,
    pub corpus_id: String,
    pub species: String,
    pub compression: String,
    pub reference_id: Option<String>,
    pub sample_count: usize,
    pub single_end_sample_count: usize,
    pub paired_end_sample_count: usize,
    pub valid: bool,
    pub samples: Vec<FastqCorpusFixtureSampleValidationReport>,
}
