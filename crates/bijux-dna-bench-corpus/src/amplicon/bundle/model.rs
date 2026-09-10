//! Owner: bijux-dna-bench-corpus
//! Amplicon truth-bundle wire contracts.
use crate::amplicon::AmpliconAbundanceTruthRow;
use crate::amplicon::AmpliconExpectedAsvTruthRow;
use crate::amplicon::AmpliconExpectedChimeraTruthRow;
use crate::amplicon::AmpliconPrimerTruthRow;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
pub const AMPLICON_TRUTH_FIXTURE_ID: &str = "amplicon-truth";
pub const AMPLICON_TRUTH_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.amplicon_truth.v1";
pub(crate) const AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.amplicon_truth.expected.v1";
pub(crate) const AMPLICON_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.amplicon_truth.validation.v1";
pub(crate) const OTU_ABUNDANCE_TABLE_KIND: &str = "otu_abundance";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AmpliconTruthManifest {
    pub(crate) schema_version: String,
    pub(crate) fixture_id: String,
    pub(crate) description: String,
    pub(crate) expected_path: PathBuf,
    pub(crate) corpus_manifest_path: PathBuf,
    pub(crate) asv_representatives_path: PathBuf,
    pub(crate) non_chimeric_representatives_path: PathBuf,
    pub(crate) otu_representatives_path: PathBuf,
    pub(crate) normalized_abundance_path: PathBuf,
    pub(crate) source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmpliconTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_section_count: usize,
    pub validated_row_count: usize,
    pub valid: bool,
    pub checked_sections: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AmpliconTruthBundle {
    pub(crate) schema_version: String,
    pub(crate) fixture_id: String,
    pub(crate) primer_truths: Vec<AmpliconPrimerTruthRow>,
    pub(crate) asv_truths: Vec<AmpliconExpectedAsvTruthRow>,
    pub(crate) chimera_truths: Vec<AmpliconExpectedChimeraTruthRow>,
    pub(crate) asv_representatives: Vec<FastaSequenceTruth>,
    pub(crate) non_chimeric_representatives: Vec<FastaSequenceTruth>,
    pub(crate) otu_representatives: Vec<FastaSequenceTruth>,
    pub(crate) otu_abundances: Vec<AmpliconAbundanceTruthRow>,
    pub(crate) normalized_abundances: Vec<NormalizedAbundanceTruthRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct FastaSequenceTruth {
    pub(crate) id: String,
    pub(crate) sequence: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NormalizedAbundanceTruthRow {
    pub(crate) sample_id: String,
    pub(crate) feature_id: String,
    pub(crate) normalized_abundance: String,
}
