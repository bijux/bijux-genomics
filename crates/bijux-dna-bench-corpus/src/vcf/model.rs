//! Owner: bijux-dna-bench-corpus
//! VCF corpus model ownership.
use super::{Deserialize, PathBuf, Serialize};

pub const DEFAULT_VCF_MINI_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/manifest.toml";

pub const VCF_CORPUS_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.vcf_corpus_fixture.v1";

pub const VCF_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.vcf_corpus_fixture_validation.v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcfCorpusFixtureManifest {
    pub schema_version: String,
    pub corpus_id: String,
    pub reference_id: String,
    pub description: String,
    pub reference_fasta_path: PathBuf,
    pub reference_fasta_index_path: PathBuf,
    pub reference_dict_path: PathBuf,
    pub raw_vcf_path: PathBuf,
    pub filtered_vcf_path: PathBuf,
    pub multisample_vcf_path: PathBuf,
    pub phased_vcf_path: PathBuf,
    pub panel_vcf_path: PathBuf,
    pub target_sites_bed_path: PathBuf,
    pub sample_metadata_path: PathBuf,
    pub population_metadata_path: PathBuf,
    pub expected_raw_sample_ids: Vec<String>,
    pub expected_filtered_sample_ids: Vec<String>,
    pub expected_multisample_sample_ids: Vec<String>,
    pub expected_phased_sample_ids: Vec<String>,
    pub expected_panel_sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfCorpusFixtureValidationReport {
    pub schema_version: &'static str,
    pub manifest_path: String,
    pub corpus_id: String,
    pub reference_id: String,
    pub reference_fasta_path: String,
    pub reference_fasta_index_path: String,
    pub reference_dict_path: String,
    pub reference_contigs: Vec<String>,
    pub target_sites_bed_path: String,
    pub target_interval_count: usize,
    pub sample_metadata_path: String,
    pub population_metadata_path: String,
    pub sample_count: usize,
    pub population_count: usize,
    pub valid: bool,
    pub variant_sets: Vec<VcfVariantSetValidationReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfVariantSetValidationReport {
    pub variant_role: String,
    pub vcf_path: String,
    pub observed_sample_ids: Vec<String>,
    pub observed_variant_count: u64,
    pub observed_contigs: Vec<String>,
    pub phased_genotypes_only: bool,
    pub valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VcfVariantRole {
    Raw,
    Filtered,
    Multisample,
    Phased,
    Panel,
}

impl VcfVariantRole {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Filtered => "filtered",
            Self::Multisample => "multisample",
            Self::Phased => "phased",
            Self::Panel => "panel",
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ReferenceContig {
    pub(super) name: String,
    pub(super) length: usize,
}

#[derive(Debug, Clone)]
pub(super) struct FastaIndexRow {
    pub(super) name: String,
    pub(super) length: usize,
}

#[derive(Debug, Clone)]
pub(super) struct ReferenceDictRow {
    pub(super) name: String,
    pub(super) length: usize,
}

#[derive(Debug, Clone)]
pub struct SampleMetadataRow {
    pub sample_id: String,
    pub population_id: String,
    pub sex: String,
    pub role: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub(super) struct VcfDocumentSummary {
    pub(super) sample_ids: Vec<String>,
    pub(super) variant_count: u64,
    pub(super) contigs: Vec<String>,
    pub(super) phased_genotypes_only: bool,
}
