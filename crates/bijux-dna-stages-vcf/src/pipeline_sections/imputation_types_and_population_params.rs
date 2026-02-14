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
    pub run_level_checksums_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostprocessStageOutputs {
    pub merged_vcf: PathBuf,
    pub merged_tbi: PathBuf,
    pub merged_bcf: Option<PathBuf>,
    pub artifact_checksums_json: PathBuf,
    pub validate_outputs_json: PathBuf,
    pub final_manifest_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PopulationPreprocessingParams {
    pub ld_window: usize,
    pub ld_step: usize,
    pub ld_r2_threshold: f64,
    pub maf_threshold: f64,
    pub max_missingness: f64,
}

impl Default for PopulationPreprocessingParams {
    fn default() -> Self {
        Self {
            ld_window: 50,
            ld_step: 5,
            ld_r2_threshold: 0.2,
            maf_threshold: 0.01,
            max_missingness: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PcaStageParams {
    pub toolchain: String,
    pub components: usize,
    pub preprocessing: PopulationPreprocessingParams,
}

impl Default for PcaStageParams {
    fn default() -> Self {
        Self {
            toolchain: "plink2".to_string(),
            components: 10,
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
    pub preprocessing: PopulationPreprocessingParams,
}

impl Default for PopulationStructureStageParams {
    fn default() -> Self {
        Self {
            toolchain: "plink2".to_string(),
            smartpca: true,
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
    pub k_values: Vec<usize>,
}

impl Default for AdmixtureStageParams {
    fn default() -> Self {
        Self {
            k_values: vec![2, 3, 4],
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
    pub min_snp_density_per_mb: f64,
    pub min_segment_kb: u64,
    pub max_gap_bp: u64,
}

impl Default for RohStageParams {
    fn default() -> Self {
        Self {
            min_snp_density_per_mb: 10.0,
            min_segment_kb: 500,
            max_gap_bp: 1_000_000,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RohStageOutputs {
    pub roh_segments_tsv: PathBuf,
    pub roh_summary_json: PathBuf,
    pub roh_metrics_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone)]
pub struct IbdStageParams {
    pub min_variant_density_per_mb: f64,
    pub max_missingness: f64,
    pub min_samples: usize,
    pub min_segment_cm: f64,
}

impl Default for IbdStageParams {
    fn default() -> Self {
        Self {
            min_variant_density_per_mb: 1.0,
            max_missingness: 0.2,
            min_samples: 2,
            min_segment_cm: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IbdStageOutputs {
    pub ibd_segments_tsv: PathBuf,
    pub ibd_filtered_segments_tsv: PathBuf,
    pub ibd_summary_json: PathBuf,
    pub ibd_metrics_json: PathBuf,
    pub logs_txt: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DemographyStageParams {
    pub min_segments: usize,
}

impl Default for DemographyStageParams {
    fn default() -> Self {
        Self { min_segments: 1 }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DemographyStageOutputs {
    pub ne_trajectory_tsv: PathBuf,
    pub demography_metrics_json: PathBuf,
    pub logs_txt: PathBuf,
}

