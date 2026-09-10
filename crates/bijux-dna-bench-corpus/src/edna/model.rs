//! Owner: bijux-dna-bench-corpus
//! EDNA corpus model ownership.
use super::{anyhow, Deserialize, FastqCorpusFixtureCompression, PathBuf, Result, Serialize};

pub const DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml";

pub const EDNA_CORPUS_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.edna_corpus_fixture.v1";

pub const EDNA_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.edna_corpus_fixture_validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdnaCorpusFixtureManifest {
    pub schema_version: String,
    pub corpus_id: String,
    pub community_id: String,
    pub description: String,
    pub compression: FastqCorpusFixtureCompression,
    pub expected_taxa_path: PathBuf,
    pub expected_taxa: Vec<EdnaExpectedTaxon>,
    pub samples: Vec<EdnaCorpusFixtureSample>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EdnaExpectedTaxon {
    pub taxon_id: u64,
    pub name: String,
    pub rank: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdnaCorpusFixtureSample {
    pub sample_id: String,
    pub community_label: String,
    pub fastq_path: PathBuf,
    pub expected_read_count: u64,
    pub source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EdnaCorpusFixtureSampleValidationReport {
    pub sample_id: String,
    pub community_label: String,
    pub fastq_path: String,
    pub source_paths: Vec<String>,
    pub observed_read_count: u64,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EdnaCorpusFixtureValidationReport {
    pub schema_version: &'static str,
    pub manifest_path: String,
    pub corpus_id: String,
    pub community_id: String,
    pub compression: String,
    pub sample_count: usize,
    pub expected_taxa_count: usize,
    pub expected_taxa_path: String,
    pub expected_taxa_output_row_count: usize,
    pub expected_present_row_count: usize,
    pub expected_absent_row_count: usize,
    pub expected_taxa: Vec<EdnaExpectedTaxon>,
    pub valid: bool,
    pub samples: Vec<EdnaCorpusFixtureSampleValidationReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EdnaExpectedPresence {
    Present,
    Absent,
}

impl EdnaExpectedPresence {
    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value {
            "present" => Ok(Self::Present),
            "absent" => Ok(Self::Absent),
            _ => Err(anyhow!(
                "eDNA expected taxonomy output presence must be `present` or `absent`, found `{value}`"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EdnaExpectedTaxonRow {
    pub sample_id: String,
    pub taxon_id: u64,
    pub name: String,
    pub rank: String,
    pub expected_presence: EdnaExpectedPresence,
}
