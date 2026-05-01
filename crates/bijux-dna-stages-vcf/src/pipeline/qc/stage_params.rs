use super::*;

pub fn assert_bgzip_tabix_artifacts(vcf_path: &Path, tbi_path: &Path) -> Result<()> {
    if !vcf_path.exists() {
        return Err(anyhow!("VCF artifact missing: {}", vcf_path.display()));
    }
    if !tbi_path.exists() {
        return Err(anyhow!("tabix index missing: {}", tbi_path.display()));
    }
    if vcf_path.extension().and_then(|ext| ext.to_str()).is_none_or(|ext| ext != "gz") {
        return Err(anyhow!(
            "VCF artifact must be bgzip-compressed (.vcf.gz): {}",
            vcf_path.display()
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct PrepareReferencePanelParams {
    pub species_id: String,
    pub build_id: String,
    pub panel_id: Option<String>,
    pub map_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrepareReferencePanelOutputs {
    pub panel_root: PathBuf,
    pub prepared_panel_vcf: PathBuf,
    pub prepared_panel_tbi: PathBuf,
    pub panel_manifest_json: PathBuf,
    pub overlap_json: PathBuf,
    pub panel_overlap_json: PathBuf,
    pub panel_files_json: PathBuf,
    pub overlap_tsv: PathBuf,
    pub chunks_json: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PhasingBackend {
    Auto,
    Shapeit5,
    Beagle,
    Eagle,
}

impl PhasingBackend {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Shapeit5 => "shapeit5",
            Self::Beagle => "beagle",
            Self::Eagle => "eagle",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhasingStageParams {
    pub species_id: String,
    pub build_id: String,
    pub backend: PhasingBackend,
    pub map_id: Option<String>,
    pub threads: usize,
    pub seed: u64,
    pub region: Option<String>,
    pub allow_gl_only_input: bool,
}

fn detect_gl_or_gp_in_vcf(raw: &str) -> bool {
    raw.lines().any(|line| {
        let Some(fields) = parse_record_fields(line) else {
            return false;
        };
        fields.len() > 8 && format_has_token(fields[8], &["GL", "GP"])
    })
}

pub(crate) fn resolve_phasing_backend(
    params: &PhasingStageParams,
    raw_vcf: &str,
) -> PhasingBackend {
    if params.backend != PhasingBackend::Auto {
        return params.backend;
    }
    if detect_gl_or_gp_in_vcf(raw_vcf) {
        return PhasingBackend::Beagle;
    }
    if params.map_id.is_some() {
        return PhasingBackend::Shapeit5;
    }
    PhasingBackend::Beagle
}

#[derive(Debug, Clone, Serialize)]
pub struct PhasingStageOutputs {
    pub phased_vcf: PathBuf,
    pub phased_tbi: PathBuf,
    pub phase_block_stats_tsv: PathBuf,
    pub switch_error_proxy_tsv: PathBuf,
    pub phasing_qc_json: PathBuf,
    pub phasing_manifest_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImputeBackend {
    Glimpse,
    Impute5,
    Minimac4,
    Beagle,
}

impl ImputeBackend {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Glimpse => "glimpse",
            Self::Impute5 => "impute5",
            Self::Minimac4 => "minimac4",
            Self::Beagle => "beagle",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImputeStageParams {
    pub species_id: String,
    pub build_id: String,
    pub backend: ImputeBackend,
    pub panel_id: Option<String>,
    pub map_id: Option<String>,
    pub threads: usize,
    pub seed: u64,
    pub emit_ds: bool,
    pub emit_gp: bool,
    pub truth_vcf: Option<PathBuf>,
    pub imputation_accept_mode: ImputationAcceptMode,
    pub chunk_window_bp: Option<u64>,
    pub chunk_overlap_bp: u64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImputationAcceptMode {
    Fail,
    MarkNonProduction,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImputeStageOutputs {
    pub imputed_vcf: PathBuf,
    pub imputed_tbi: PathBuf,
    pub imputation_qc_json: PathBuf,
    pub imputation_qc_tsv: PathBuf,
    pub maf_bin_quality_tsv: PathBuf,
    pub info_hist_json: PathBuf,
    pub warnings_json: PathBuf,
    pub imputation_accept_json: PathBuf,
    pub overlap_stats_json: PathBuf,
    pub imputation_manifest_json: PathBuf,
    pub panel_mismatch_diagnostics_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImputationOrchestrationOutputs {
    pub imputed_vcf: PathBuf,
    pub imputed_tbi: PathBuf,
    pub imputation_manifest_json: PathBuf,
    pub orchestration_manifest_json: PathBuf,
    pub imputation_qc_json: PathBuf,
    pub imputation_accept_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PostprocessStageParams {
    pub species_id: String,
    pub build_id: String,
    pub per_chr_inputs: Vec<PathBuf>,
    pub retain_info_fields: Vec<String>,
    pub remove_info_fields: Vec<String>,
    pub compression_level: u8,
    pub compression_threads: usize,
    pub emit_bcf: bool,
    pub normalize_indels: bool,
    pub split_multiallelic: bool,
    pub run_level_checksums_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostprocessStageOutputs {
    pub merged_vcf: PathBuf,
    pub merged_tbi: PathBuf,
    pub merged_bcf: Option<PathBuf>,
    pub artifact_checksums_json: PathBuf,
    pub normalization_contract_json: PathBuf,
    pub validate_outputs_json: PathBuf,
    pub final_manifest_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PopulationPreprocessingParams {
    pub ld_window: usize,
    pub ld_step: usize,
    pub ld_r2_threshold: f64,
    pub ld_pruning_policy: Option<String>,
    pub maf_threshold: f64,
    pub max_missingness: f64,
}

impl Default for PopulationPreprocessingParams {
    fn default() -> Self {
        Self {
            ld_window: 50,
            ld_step: 5,
            ld_r2_threshold: 0.2,
            ld_pruning_policy: Some("plink2_indep_pairwise".to_string()),
            maf_threshold: 0.01,
            max_missingness: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PcaStageParams {
    pub toolchain: String,
    pub components: usize,
    pub sample_metadata_manifest: Option<PathBuf>,
    pub preprocessing: PopulationPreprocessingParams,
}

impl Default for PcaStageParams {
    fn default() -> Self {
        Self {
            toolchain: "plink2".to_string(),
            components: 10,
            sample_metadata_manifest: None,
            preprocessing: PopulationPreprocessingParams::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PcaStageOutputs {
    pub eigenvec_tsv: PathBuf,
    pub eigenval_tsv: PathBuf,
    pub pca_manifest_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PopulationStructureStageParams {
    pub toolchain: String,
    pub smartpca: bool,
    pub run_admixture: bool,
    pub sample_metadata_manifest: Option<PathBuf>,
    pub admixture_params: Option<AdmixtureStageParams>,
    pub preprocessing: PopulationPreprocessingParams,
}

impl Default for PopulationStructureStageParams {
    fn default() -> Self {
        Self {
            toolchain: "plink2".to_string(),
            smartpca: true,
            run_admixture: false,
            sample_metadata_manifest: None,
            admixture_params: None,
            preprocessing: PopulationPreprocessingParams::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PopulationStructureStageOutputs {
    pub pruned_variants_tsv: PathBuf,
    pub population_structure_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AdmixtureStageParams {
    pub toolchain: String,
    pub k_values: Vec<usize>,
    pub sample_metadata_manifest: Option<PathBuf>,
}

impl Default for AdmixtureStageParams {
    fn default() -> Self {
        Self {
            toolchain: "plink2".to_string(),
            k_values: vec![2, 3, 4],
            sample_metadata_manifest: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AdmixtureStageOutputs {
    pub q_matrix_tsv: PathBuf,
    pub k_selection_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RohStageParams {
    pub toolchain: String,
    pub min_snp_density_per_mb: f64,
    pub max_missingness: f64,
    pub low_coverage_missingness_threshold: f64,
    pub allow_pseudohaploid_low_coverage: bool,
    pub min_segment_kb: u64,
    pub max_gap_bp: u64,
    pub max_segment_count: u64,
    pub plink_homozyg_window_snp: u64,
    pub plink_homozyg_kb: u64,
    pub plink_homozyg_gap_kb: u64,
}

impl Default for RohStageParams {
    fn default() -> Self {
        Self {
            toolchain: "plink2".to_string(),
            min_snp_density_per_mb: 10.0,
            max_missingness: 0.2,
            low_coverage_missingness_threshold: 0.35,
            allow_pseudohaploid_low_coverage: false,
            min_segment_kb: 500,
            max_gap_bp: 1_000_000,
            max_segment_count: 20_000,
            plink_homozyg_window_snp: 50,
            plink_homozyg_kb: 500,
            plink_homozyg_gap_kb: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RohStageOutputs {
    pub roh_segments_tsv: PathBuf,
    pub roh_per_sample_tsv: PathBuf,
    pub roh_json: PathBuf,
    pub metrics_json: PathBuf,
    pub roh_summary_json: PathBuf,
    pub roh_metrics_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone)]
pub struct IbdStageParams {
    pub toolchain: String,
    pub expected_build: Option<String>,
    pub min_variant_density_per_mb: f64,
    pub max_missingness: f64,
    pub min_samples: usize,
    pub min_segment_cm: f64,
    pub min_markers_per_segment: usize,
}

impl Default for IbdStageParams {
    fn default() -> Self {
        Self {
            toolchain: "germline+ibdhap".to_string(),
            expected_build: None,
            min_variant_density_per_mb: 1.0,
            max_missingness: 0.2,
            min_samples: 2,
            min_segment_cm: 2.0,
            min_markers_per_segment: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IbdStageOutputs {
    pub ibd_input_tsv: PathBuf,
    pub ibd_segments_tsv: PathBuf,
    pub ibd_merged_segments_tsv: PathBuf,
    pub ibd_filtered_segments_tsv: PathBuf,
    pub ibd_summary_json: PathBuf,
    pub ibd_metrics_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DemographyStageParams {
    pub min_segments: usize,
    pub expected_build: Option<String>,
}

impl Default for DemographyStageParams {
    fn default() -> Self {
        Self { min_segments: 1, expected_build: None }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DemographyStageOutputs {
    pub ne_trajectory_tsv: PathBuf,
    pub demography_json: PathBuf,
    pub demography_metrics_json: PathBuf,
    pub logs_txt: PathBuf,
}
