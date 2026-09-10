//! Owner: bijux-dna-bench-corpus
//! BAM corpus model ownership.
use super::{Deserialize, PathBuf, Serialize};

pub const DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/manifest.toml";

pub const DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/manifest.toml";

pub const DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-genotyping-mini/manifest.toml";

pub const DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-kinship-mini/manifest.toml";

pub const DEFAULT_CORPUS_01_VCF_COHORT_BAM_MINI_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-vcf-cohort-bam-mini/manifest.toml";

pub const BAM_CORPUS_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.bam_corpus_fixture.v1";

pub const BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_corpus_fixture_validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BamCorpusFixtureManifest {
    pub schema_version: String,
    pub corpus_id: String,
    pub species: String,
    pub description: String,
    #[serde(default)]
    pub reference_id: Option<String>,
    pub reference_fasta: PathBuf,
    #[serde(default)]
    pub udg_model: Option<String>,
    #[serde(default)]
    pub damage_signal: Option<String>,
    #[serde(default)]
    pub expected_terminal_pattern_class: Option<String>,
    #[serde(default)]
    pub genotyping_contract: Option<BamCorpusFixtureGenotypingContract>,
    #[serde(default)]
    pub kinship_contract: Option<BamCorpusFixtureKinshipContract>,
    pub samples: Vec<BamCorpusFixtureSample>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BamCorpusFixtureGenotypingContract {
    pub sample_id: String,
    pub sites_vcf: PathBuf,
    pub regions: PathBuf,
    pub min_posterior: f64,
    pub min_call_rate: f64,
    pub expected_outputs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BamCorpusFixtureKinshipContract {
    pub reference_panel: String,
    pub reference_panel_path: PathBuf,
    pub reference_build: String,
    pub population_scope: String,
    pub expected_outputs: Vec<String>,
    pub cases: Vec<BamCorpusFixtureKinshipCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BamCorpusFixtureKinshipCase {
    pub sample_id: String,
    pub min_overlap_snps: u32,
    pub expected_status: String,
    pub expected_observed_max_overlap_snps: u32,
    #[serde(default)]
    pub expected_relationship_labels: Vec<String>,
    pub skip_behavior: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BamCorpusFixtureSample {
    pub sample_id: String,
    pub cohort: String,
    pub alignment_path: PathBuf,
    pub index_path: PathBuf,
    pub expected_contigs: Vec<String>,
    pub expected_header_sample_ids: Vec<String>,
    pub expected_read_group_ids: Vec<String>,
    pub expected_record_count: u64,
    pub source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamCorpusFixtureSampleValidationReport {
    pub sample_id: String,
    pub cohort: String,
    pub alignment_path: String,
    pub index_path: String,
    pub source_paths: Vec<String>,
    pub observed_contigs: Vec<String>,
    pub observed_header_sample_ids: Vec<String>,
    pub observed_read_group_ids: Vec<String>,
    pub observed_record_count: u64,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamCorpusFixtureValidationReport {
    pub schema_version: &'static str,
    pub manifest_path: String,
    pub corpus_id: String,
    pub species: String,
    pub reference_id: Option<String>,
    pub reference_fasta: String,
    pub udg_model: Option<String>,
    pub damage_signal: Option<String>,
    pub expected_terminal_pattern_class: Option<String>,
    pub genotyping_contract: Option<BamCorpusFixtureGenotypingContractReport>,
    pub kinship_contract: Option<BamCorpusFixtureKinshipContractReport>,
    pub reference_contigs: Vec<String>,
    pub sample_count: usize,
    pub valid: bool,
    pub samples: Vec<BamCorpusFixtureSampleValidationReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamCorpusFixtureGenotypingContractReport {
    pub sample_id: String,
    pub sites_vcf: String,
    pub regions: String,
    pub min_posterior: f64,
    pub min_call_rate: f64,
    pub expected_outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamCorpusFixtureKinshipContractReport {
    pub reference_panel: String,
    pub reference_panel_path: String,
    pub reference_build: String,
    pub population_scope: String,
    pub expected_outputs: Vec<String>,
    pub cases: Vec<BamCorpusFixtureKinshipCaseReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamCorpusFixtureKinshipCaseReport {
    pub sample_id: String,
    pub min_overlap_snps: u32,
    pub expected_status: String,
    pub expected_observed_max_overlap_snps: u32,
    pub expected_relationship_labels: Vec<String>,
    pub skip_behavior: String,
}
