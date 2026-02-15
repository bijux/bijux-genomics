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

fn maf_bin_label(maf: f64) -> &'static str {
    if maf < 0.01 {
        "0-0.01"
    } else if maf < 0.05 {
        "0.01-0.05"
    } else if maf < 0.1 {
        "0.05-0.1"
    } else if maf < 0.2 {
        "0.1-0.2"
    } else {
        "0.2-0.5"
    }
}

fn parse_gt_counts(format_field: &str, sample_fields: &[&str]) -> Option<(u64, u64, u64, u64)> {
    let keys = format_field.split(':').collect::<Vec<_>>();
    let gt_idx = keys.iter().position(|k| *k == "GT")?;
    let mut hom_ref = 0_u64;
    let mut het = 0_u64;
    let mut hom_alt = 0_u64;
    let mut total = 0_u64;
    for sample in sample_fields {
        let vals = sample.split(':').collect::<Vec<_>>();
        let Some(gt) = vals.get(gt_idx) else {
            continue;
        };
        if gt.contains('.') {
            continue;
        }
        total += 1;
        match gt.replace('|', "/").as_str() {
            "0/0" => hom_ref += 1,
            "0/1" | "1/0" => het += 1,
            "1/1" => hom_alt += 1,
            _ => {}
        }
    }
    Some((hom_ref, het, hom_alt, total))
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
    let raw = read_vcf_text(input_vcf)?;
    let mut depth = std::collections::BTreeMap::<String, u64>::new();
    let mut info_values = Vec::<f64>::new();
    let mut rsq_values = Vec::<f64>::new();
    let mut af_values = Vec::<f64>::new();
    let mut maf_bins = std::collections::BTreeMap::<String, u64>::new();
    let mut missing = 0_u64;
    let mut called = 0_u64;
    let mut variants = 0_u64;
    let mut site_missingness = Vec::<f64>::new();
    let mut hwe_p_values = Vec::<f64>::new();
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
            *maf_bins.entry(maf_bin_label(v).to_string()).or_insert(0) += 1;
        }
        if fields.len() > 9 {
            let keys = fields[8].split(':').collect::<Vec<_>>();
            if let Some(gt_idx) = keys.iter().position(|k| *k == "GT") {
                let mut site_missing = 0_u64;
                let mut site_total = 0_u64;
                for sample in &fields[9..] {
                    let vals = sample.split(':').collect::<Vec<_>>();
                    if let Some(gt) = vals.get(gt_idx) {
                        site_total += 1;
                        if gt.contains('.') {
                            missing += 1;
                            site_missing += 1;
                        } else {
                            called += 1;
                        }
                    }
                }
                if site_total > 0 {
                    site_missingness.push(site_missing as f64 / site_total as f64);
                }
            }
            if !params.is_ancient_dna {
                if let Some((hom_ref, het, hom_alt, total)) = parse_gt_counts(fields[8], &fields[9..]) {
                    if total > 0 {
                        let n = total as f64;
                        let p = (2.0 * hom_ref as f64 + het as f64) / (2.0 * n);
                        let q = 1.0 - p;
                        let e_hom_ref = n * p * p;
                        let e_het = 2.0 * n * p * q;
                        let e_hom_alt = n * q * q;
                        if e_hom_ref > 0.0 && e_het > 0.0 && e_hom_alt > 0.0 {
                            let chi2 = (hom_ref as f64 - e_hom_ref).powi(2) / e_hom_ref
                                + (het as f64 - e_het).powi(2) / e_het
                                + (hom_alt as f64 - e_hom_alt).powi(2) / e_hom_alt;
                            let p_approx = (-0.5 * chi2).exp();
                            hwe_p_values.push(p_approx);
                        }
                    }
                }
            }
        }
    }
    let missingness_post = if called + missing == 0 { 0.0 } else { missing as f64 / (called + missing) as f64 };
    let missingness_pre = if let Some(pre) = &params.pre_filter_vcf {
        let pre_raw = read_vcf_text(pre)?;
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
    let site_missingness_mean = if site_missingness.is_empty() {
        0.0
    } else {
        site_missingness.iter().sum::<f64>() / site_missingness.len() as f64
    };
    let hwe_p_mean = if hwe_p_values.is_empty() {
        None
    } else {
        Some(hwe_p_values.iter().sum::<f64>() / hwe_p_values.len() as f64)
    };
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
    table.push_str(&format!("site_missingness_mean\t{site_missingness_mean:.6}\n"));
    table.push_str(&format!("imputation_info_mean\t{info_mean:.6}\n"));
    table.push_str(&format!("rsq_mean\t{rsq_mean:.6}\n"));
    if let Some(hwe) = hwe_p_mean {
        table.push_str(&format!("hwe_pvalue_mean\t{hwe:.6}\n"));
    }
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
            "rsq_distribution": rsq_values,
            "maf_bins": maf_bins,
            "site_missingness_distribution": site_missingness,
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
            "maf_bins": maf_bins,
            "site_missingness_mean": site_missingness_mean,
            "depth_distribution": depth,
            "hwe_pvalue_mean": hwe_p_mean,
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
    let source_vcfgz = if input_vcf
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x == "gz" || x == "bcf")
    {
        input_vcf.to_path_buf()
    } else {
        let normalized_input = out_dir.join("stats_input.normalized.vcf.gz");
        let plain_input = out_dir.join("stats_input.normalized.vcf");
        std::fs::copy(input_vcf, &plain_input)?;
        let _ = crate::vcf_io::vcf_index_bgzip_tabix(&plain_input, &normalized_input)?;
        normalized_input
    };
    let bcftools_stats_txt = out_dir.join("bcftools_stats.txt");
    let mut metrics = match crate::vcf_io::vcf_stats_basic(&source_vcfgz, &bcftools_stats_txt) {
        Ok(real) => real,
        Err(_) => {
            let call = parse_vcf_call_summary(input_vcf, &params.sample_name)?;
            let filter = parse_vcf_filter_breakdown(input_vcf, &params.sample_name)?;
            VcfStatsMetricsV1 {
                schema_version: "bijux.vcf.stats.v1".to_string(),
                sample_name: params.sample_name.clone(),
                variants_total: call.variants_called,
                snps: call.snps,
                indels: call.indels,
                ti_tv: None,
                filter_breakdown: filter.filter_breakdown.clone(),
                depth_distribution: std::collections::BTreeMap::new(),
                call_summary: call,
                filter_summary: filter,
            }
        }
    };
    metrics.sample_name = params.sample_name.clone();
    let stats_json = out_dir.join("stats.json");
    atomic_write_json(&stats_json, &serde_json::to_value(&metrics)?)?;
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
    pub max_missingness: f64,
    pub min_segment_kb: u64,
    pub max_gap_bp: u64,
}

impl Default for RohStageParams {
    fn default() -> Self {
        Self {
            min_snp_density_per_mb: 10.0,
            max_missingness: 0.2,
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
