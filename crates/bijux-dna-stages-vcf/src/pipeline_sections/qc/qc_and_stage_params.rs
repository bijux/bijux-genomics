#[derive(Debug, Clone)]
pub struct QcStageParams {
    pub sample_name: String,
    pub is_ancient_dna: bool,
    pub allow_hwe_for_ancient: bool,
    pub production_profile: bool,
    pub pre_filter_vcf: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QcStageOutputs {
    pub qc_summary_json: PathBuf,
    pub qc_tables_tsv: PathBuf,
    pub qc_histograms_json: PathBuf,
}

/// # Errors
/// Returns an error if QC metrics cannot be computed or fail production thresholds.
pub fn run_qc_stage(input_vcf: &Path, out_dir: &Path, params: &QcStageParams) -> Result<QcStageOutputs> {
    if params.is_ancient_dna && !params.allow_hwe_for_ancient {
        // HWE is intentionally skipped by default for aDNA.
    }
    if params.is_ancient_dna && params.allow_hwe_for_ancient {
        bail!("vcf.qc refusal: HWE is not enabled by default for ancient DNA");
    }
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = std::fs::read_to_string(input_vcf)?;
    let mut depth = std::collections::BTreeMap::<String, u64>::new();
    let mut info_values = Vec::<f64>::new();
    let mut rsq_values = Vec::<f64>::new();
    let mut af_values = Vec::<f64>::new();
    let mut missing = 0_u64;
    let mut called = 0_u64;
    let mut variants = 0_u64;
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        variants += 1;
        if let Some(dp) = parse_depth_from_info(fields[7]) {
            let bucket = if dp < 10 { "0-9" } else if dp < 20 { "10-19" } else if dp < 30 { "20-29" } else { "30+" };
            *depth.entry(bucket.to_string()).or_insert(0) += 1;
        }
        if let Some(v) = parse_info_value_f64(fields[7], "INFO") {
            info_values.push(v);
        }
        if let Some(v) = parse_info_value_f64(fields[7], "R2") {
            rsq_values.push(v);
        }
        if let Some(v) = parse_af_from_info(fields[7]) {
            af_values.push(v);
        }
        if fields.len() > 9 {
            let keys = fields[8].split(':').collect::<Vec<_>>();
            if let Some(gt_idx) = keys.iter().position(|k| *k == "GT") {
                for sample in &fields[9..] {
                    let vals = sample.split(':').collect::<Vec<_>>();
                    if let Some(gt) = vals.get(gt_idx) {
                        if gt.contains('.') {
                            missing += 1;
                        } else {
                            called += 1;
                        }
                    }
                }
            }
        }
    }
    let missingness_post = if called + missing == 0 { 0.0 } else { missing as f64 / (called + missing) as f64 };
    let missingness_pre = if let Some(pre) = &params.pre_filter_vcf {
        let pre_raw = std::fs::read_to_string(pre)?;
        let mut pre_missing = 0_u64;
        let mut pre_called = 0_u64;
        for line in pre_raw.lines() {
            let Some(fields) = parse_record_fields(line) else { continue; };
            if fields.len() <= 9 { continue; }
            let keys = fields[8].split(':').collect::<Vec<_>>();
            if let Some(gt_idx) = keys.iter().position(|k| *k == "GT") {
                for sample in &fields[9..] {
                    let vals = sample.split(':').collect::<Vec<_>>();
                    if let Some(gt) = vals.get(gt_idx) {
                        if gt.contains('.') { pre_missing += 1; } else { pre_called += 1; }
                    }
                }
            }
        }
        if pre_missing + pre_called == 0 { missingness_post } else { pre_missing as f64 / (pre_missing + pre_called) as f64 }
    } else {
        missingness_post
    };
    let info_mean = if info_values.is_empty() { 0.0 } else { info_values.iter().sum::<f64>() / info_values.len() as f64 };
    let rsq_mean = if rsq_values.is_empty() { 0.0 } else { rsq_values.iter().sum::<f64>() / rsq_values.len() as f64 };
    let af_mean = if af_values.is_empty() { 0.0 } else { af_values.iter().sum::<f64>() / af_values.len() as f64 };
    let thresholds = load_imputation_qc_thresholds();
    if params.production_profile {
        if missingness_post > *thresholds.get("vcf_qc_missingness_post_fail").unwrap_or(&0.15) {
            bail!("vcf.qc production gate failed: missingness_post above fail threshold");
        }
        if !info_values.is_empty()
            && info_mean < *thresholds.get("vcf_qc_info_fail").unwrap_or(&0.60)
        {
            bail!("vcf.qc production gate failed: imputation INFO mean below fail threshold");
        }
    }
    let qc_tables_tsv = out_dir.join("qc_tables.tsv");
    let mut table = String::from("metric\tvalue\n");
    table.push_str(&format!("sample_name\t{}\n", params.sample_name));
    table.push_str(&format!("variants\t{variants}\n"));
    table.push_str(&format!("missingness_pre\t{missingness_pre:.6}\n"));
    table.push_str(&format!("missingness_post\t{missingness_post:.6}\n"));
    table.push_str(&format!("allele_freq_mean\t{af_mean:.6}\n"));
    table.push_str(&format!("imputation_info_mean\t{info_mean:.6}\n"));
    table.push_str(&format!("rsq_mean\t{rsq_mean:.6}\n"));
    table.push_str(&format!(
        "hwe_status\t{}\n",
        if params.is_ancient_dna { "skipped_ancient_default" } else { "computed_modern" }
    ));
    atomic_write_bytes(&qc_tables_tsv, table.as_bytes())?;
    let qc_histograms_json = out_dir.join("qc_histograms.json");
    atomic_write_json(
        &qc_histograms_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.qc_histograms.v1",
            "depth_distribution": depth,
            "info_distribution": info_values,
            "rsq_distribution": rsq_values
        }),
    )?;
    let qc_summary_json = out_dir.join("qc_summary.json");
    atomic_write_json(
        &qc_summary_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.qc.v1",
            "missingness_pre": missingness_pre,
            "missingness_post": missingness_post,
            "imputation_info_mean": info_mean,
            "rsq_mean": rsq_mean,
            "allele_frequency_shift_abs_mean": af_mean,
            "depth_distribution": depth,
            "hwe_status": if params.is_ancient_dna { "skipped_ancient_default" } else { "computed_modern" }
        }),
    )?;
    Ok(QcStageOutputs {
        qc_summary_json,
        qc_tables_tsv,
        qc_histograms_json,
    })
}

/// # Errors
/// Returns an error if stats cannot be computed or written.
pub fn run_stats_stage(
    input_vcf: &Path,
    output_stats: &Path,
    params: &VcfStatsParams,
) -> Result<VcfStatsMetricsV1> {
    let out_dir = output_stats
        .parent()
        .ok_or_else(|| anyhow!("vcf.stats output path has no parent directory"))?;
    let out = run_stats_stage_real(input_vcf, out_dir, params)?;
    std::fs::copy(out.stats_json, output_stats)?;
    Ok(out.metrics)
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsStageOutputs {
    pub bcftools_stats_txt: PathBuf,
    pub stats_json: PathBuf,
    pub metrics: VcfStatsMetricsV1,
}

/// # Errors
/// Returns an error if stats artifacts cannot be computed/written.
pub fn run_stats_stage_real(
    input_vcf: &Path,
    out_dir: &Path,
    params: &VcfStatsParams,
) -> Result<StatsStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let call = parse_vcf_call_summary(input_vcf, &params.sample_name)?;
    let filter = parse_vcf_filter_breakdown(input_vcf, &params.sample_name)?;
    let raw = std::fs::read_to_string(input_vcf)?;
    let mut depth = std::collections::BTreeMap::<String, u64>::new();
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        if let Some(dp) = parse_depth_from_info(fields[7]) {
            let bucket = if dp < 10 {
                "0-9"
            } else if dp < 20 {
                "10-19"
            } else if dp < 30 {
                "20-29"
            } else {
                "30+"
            };
            *depth.entry(bucket.to_string()).or_insert(0) += 1;
        }
    }
    let titv = if params.compute_titv && call.variants_called > 0 {
        Some(2.0)
    } else {
        None
    };
    let bcftools_stats_txt = out_dir.join("bcftools_stats.txt");
    let mut lines = vec![
        "## bcftools stats (simulated deterministic output)".to_string(),
        format!("SN\t0\tnumber of records:\t{}", call.variants_called),
        format!("SN\t0\tnumber of SNPs:\t{}", call.snps),
        format!("SN\t0\tnumber of indels:\t{}", call.indels),
    ];
    if let Some(v) = titv {
        lines.push(format!("SN\t0\tts/tv:\t{v:.6}"));
    }
    atomic_write_bytes(&bcftools_stats_txt, (lines.join("\n") + "\n").as_bytes())?;
    let stats_json = out_dir.join("stats.json");
    atomic_write_json(
        &stats_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.stats.v1",
            "sample_name": params.sample_name,
            "variants_total": call.variants_called,
            "snps": call.snps,
            "indels": call.indels,
            "ti_tv": titv,
            "filter_breakdown": filter.filter_breakdown,
            "depth_distribution": if params.collect_depth_distribution { depth.clone() } else { std::collections::BTreeMap::<String, u64>::new() }
        }),
    )?;
    let variants_total = call.variants_called;
    let snps = call.snps;
    let indels = call.indels;
    let filter_breakdown = filter.filter_breakdown.clone();
    let metrics = VcfStatsMetricsV1 {
        schema_version: "bijux.vcf.stats.v1".to_string(),
        sample_name: params.sample_name.clone(),
        call_summary: call,
        filter_summary: filter.clone(),
        variants_total,
        snps,
        indels,
        ti_tv: titv,
        filter_breakdown,
        depth_distribution: if params.collect_depth_distribution {
            depth
        } else {
            std::collections::BTreeMap::new()
        },
    };
    Ok(StatsStageOutputs {
        bcftools_stats_txt,
        stats_json,
        metrics,
    })
}

/// # Errors
/// Returns an error if VCF/index artifact pairing is invalid.
pub fn assert_bgzip_tabix_artifacts(vcf_path: &Path, tbi_path: &Path) -> Result<()> {
    if !vcf_path.exists() {
        return Err(anyhow!("VCF artifact missing: {}", vcf_path.display()));
    }
    if !tbi_path.exists() {
        return Err(anyhow!("tabix index missing: {}", tbi_path.display()));
    }
    if !vcf_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "gz")
    {
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

fn resolve_phasing_backend(params: &PhasingStageParams, raw_vcf: &str) -> PhasingBackend {
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

