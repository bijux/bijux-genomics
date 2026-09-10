//! Owner: bijux-dna-bench-corpus
//! AMPLICON corpus model ownership.
use super::{
    anyhow, BTreeMap, Deserialize, FastqCorpusFixtureCompression, PathBuf, Result, Serialize,
};

pub const DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/manifest.toml";

pub const AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.amplicon_corpus_fixture.v1";

pub const AMPLICON_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.amplicon_corpus_fixture_validation.v1";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AmpliconCorpusSampleKind {
    Biological,
    Control,
}

impl AmpliconCorpusSampleKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Biological => "biological",
            Self::Control => "control",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmpliconCorpusFixtureManifest {
    pub schema_version: String,
    pub corpus_id: String,
    pub assay_id: String,
    pub marker_id: String,
    pub target_region: String,
    pub description: String,
    pub compression: FastqCorpusFixtureCompression,
    pub primer_set_id: String,
    pub forward_primer_id: String,
    pub reverse_primer_id: String,
    pub primer_fasta: PathBuf,
    pub primers_tsv_path: PathBuf,
    pub expected_asvs_path: PathBuf,
    pub chimera_controls_fasta_path: PathBuf,
    pub chimera_expectations_path: PathBuf,
    pub amplicon_governance_path: PathBuf,
    pub controls: Vec<AmpliconCorpusControl>,
    pub abundance_tables: Vec<AmpliconCorpusAbundanceTable>,
    pub samples: Vec<AmpliconCorpusFixtureSample>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AmpliconCorpusControl {
    pub sample_id: String,
    pub control_kind: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmpliconCorpusAbundanceTable {
    pub sample_id: String,
    pub table_kind: String,
    pub table_path: PathBuf,
    pub expected_row_count: u64,
    pub expected_sample_count: u64,
    pub expected_feature_count: u64,
    pub source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmpliconCorpusFixtureSample {
    pub sample_id: String,
    pub sample_kind: AmpliconCorpusSampleKind,
    pub fastq_path: PathBuf,
    pub expected_read_count: u64,
    pub source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmpliconCorpusControlValidationReport {
    pub sample_id: String,
    pub control_kind: String,
    pub purpose: String,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmpliconCorpusFixtureSampleValidationReport {
    pub sample_id: String,
    pub sample_kind: String,
    pub fastq_path: String,
    pub source_paths: Vec<String>,
    pub observed_read_count: u64,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmpliconCorpusAbundanceTableValidationReport {
    pub sample_id: String,
    pub table_kind: String,
    pub table_path: String,
    pub source_paths: Vec<String>,
    pub observed_row_count: u64,
    pub observed_sample_count: u64,
    pub observed_feature_count: u64,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmpliconCorpusFixtureValidationReport {
    pub schema_version: &'static str,
    pub manifest_path: String,
    pub corpus_id: String,
    pub assay_id: String,
    pub marker_id: String,
    pub target_region: String,
    pub compression: String,
    pub primer_set_id: String,
    pub forward_primer_id: String,
    pub reverse_primer_id: String,
    pub primer_fasta: String,
    pub primers_tsv_path: String,
    pub primer_table_row_count: usize,
    pub expected_asvs_path: String,
    pub expected_asv_row_count: usize,
    pub expected_asv_present_row_count: usize,
    pub expected_asv_absent_row_count: usize,
    pub chimera_controls_fasta_path: String,
    pub chimera_expectations_path: String,
    pub chimera_expectation_row_count: usize,
    pub chimera_expected_present_row_count: usize,
    pub chimera_expected_absent_row_count: usize,
    pub amplicon_governance_path: String,
    pub abundance_table_count: usize,
    pub sample_count: usize,
    pub control_count: usize,
    pub valid: bool,
    pub controls: Vec<AmpliconCorpusControlValidationReport>,
    pub abundance_tables: Vec<AmpliconCorpusAbundanceTableValidationReport>,
    pub samples: Vec<AmpliconCorpusFixtureSampleValidationReport>,
}

#[derive(Debug, Deserialize)]
pub(super) struct AmpliconGovernanceDocument {
    pub(super) markers: BTreeMap<String, AmpliconGovernanceMarker>,
}

#[derive(Debug, Deserialize)]
pub(super) struct AmpliconGovernanceMarker {
    pub(super) primer_set_id: String,
    pub(super) primer_fasta: PathBuf,
    pub(super) applicable_assays: Vec<String>,
}

#[derive(Debug)]
pub(super) struct AmpliconPrimerTableRow {
    pub(super) primer_id: String,
    pub(super) forward_sequence: String,
    pub(super) reverse_sequence: String,
    pub(super) target: String,
    pub(super) orientation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AmpliconExpectedPresence {
    Present,
    Absent,
}

impl AmpliconExpectedPresence {
    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value {
            "present" => Ok(Self::Present),
            "absent" => Ok(Self::Absent),
            _ => Err(anyhow!(
                "amplicon expected ASV presence must be `present` or `absent`, found `{value}`"
            )),
        }
    }
}

#[derive(Debug)]
pub(super) struct AmpliconExpectedAsvRow {
    pub(super) asv_id: String,
    pub(super) sequence: String,
    pub(super) sample_id: String,
    pub(super) expected_presence: AmpliconExpectedPresence,
}

#[derive(Debug)]
pub(super) struct AmpliconExpectedChimeraRow {
    pub(super) chimera_id: String,
    pub(super) sequence: String,
    pub(super) sample_id: String,
    pub(super) expected_presence: AmpliconExpectedPresence,
}

#[derive(Debug)]
#[expect(
    clippy::struct_field_names,
    reason = "Observation fields explicitly retain their count units."
)]
pub(super) struct AmpliconAbundanceTableMetrics {
    pub(super) row_count: u64,
    pub(super) sample_count: u64,
    pub(super) feature_count: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AmpliconPrimerTruthRow {
    pub primer_id: String,
    pub forward_sequence: String,
    pub reverse_sequence: String,
    pub target: String,
    pub orientation: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AmpliconExpectedAsvTruthRow {
    pub asv_id: String,
    pub sequence: String,
    pub sample_id: String,
    pub expected_presence: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AmpliconExpectedChimeraTruthRow {
    pub chimera_id: String,
    pub sequence: String,
    pub sample_id: String,
    pub expected_presence: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AmpliconAbundanceTruthRow {
    pub sample_id: String,
    pub feature_id: String,
    pub abundance: u64,
}
