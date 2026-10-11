//! Owner: bijux-dna-bench-corpus
//! DAMAGE corpus model ownership.
use super::{Deserialize, PathBuf, Serialize};

pub const DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/manifest.toml";

pub const BAM_DAMAGE_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.bam_damage_fixture.v1";

pub(super) const BAM_DAMAGE_FIXTURE_EXPECTATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_damage_expectation.v1";

pub const BAM_DAMAGE_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_damage_fixture_validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BamDamageFixtureManifest {
    pub schema_version: String,
    pub fixture_id: String,
    pub sample_id: String,
    pub species: String,
    pub description: String,
    pub bam_path: PathBuf,
    pub index_path: PathBuf,
    pub reference_fasta: PathBuf,
    pub expected_damage_path: PathBuf,
    pub udg_model: String,
    pub expected_terminal_pattern_class: String,
    pub limitations: Vec<String>,
    pub source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BamDamageFixtureExpectation {
    pub schema_version: String,
    pub fixture_id: String,
    pub sample_id: String,
    pub terminal_c_to_t_5p: f64,
    pub terminal_g_to_a_3p: f64,
    pub short_fragment_fraction: f64,
    pub damage_signal: String,
    pub strict_profile_upgraded: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamDamageFixtureValidationReport {
    pub schema_version: &'static str,
    pub manifest_path: String,
    pub fixture_id: String,
    pub sample_id: String,
    pub species: String,
    pub bam_path: String,
    pub index_path: String,
    pub reference_fasta: String,
    pub expected_damage_path: String,
    pub udg_model: String,
    pub expected_terminal_pattern_class: String,
    pub limitations: Vec<String>,
    pub observed_contigs: Vec<String>,
    pub observed_header_sample_ids: Vec<String>,
    pub expected_damage: BamDamageFixtureExpectation,
    pub source_paths: Vec<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, Default)]
pub(super) struct TinySamHeaderSummary {
    pub(super) contigs: Vec<String>,
    pub(super) sample_ids: Vec<String>,
}
