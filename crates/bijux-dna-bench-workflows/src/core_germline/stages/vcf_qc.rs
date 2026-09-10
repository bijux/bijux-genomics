//! Owner: bijux-dna-bench-workflows
//! Core germline stages vcf qc.
use super::super::{
    path_relative_to_repo, read_json_document, required_output, required_tool_id, run_qc_stage,
    BTreeMap, Context, CoreGermlineMicroPipelineRow, Path, QcStageParams, Result, Value,
    VcfQcStageReport, PIPELINE_SAMPLE_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_vcf_qc_stage(
    repo_root: &Path,
    output_root: &Path,
    filter_row: &CoreGermlineMicroPipelineRow,
    stats_row: &CoreGermlineMicroPipelineRow,
    tool_ids: &BTreeMap<String, String>,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.qc");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let filtered_vcf = repo_root.join(required_output(filter_row, "filtered_vcf")?);
    let stage_outputs = run_qc_stage(
        &filtered_vcf,
        &stage_root,
        &QcStageParams {
            sample_name: PIPELINE_SAMPLE_ID.to_string(),
            is_ancient_dna: false,
            allow_hwe_for_ancient: false,
            production_profile: false,
            pre_filter_vcf: None,
        },
    )?;
    let qc_summary = read_json_document(&stage_outputs.qc_summary_json)?;
    let report = VcfQcStageReport {
        schema_version: "bijux.bench.local_core_germline_micro_pipeline.vcf_qc.v1".to_string(),
        stage_id: stage::VCF_QC.to_string(),
        tool_id: required_tool_id(tool_ids, stage::VCF_QC)?,
        consumed_stats_json_path: required_output(stats_row, "stats_json")?,
        qc_summary_path: path_relative_to_repo(repo_root, &stage_outputs.qc_summary_json),
        qc_tables_path: path_relative_to_repo(repo_root, &stage_outputs.qc_tables_tsv),
        imputation_qc_path: path_relative_to_repo(repo_root, &stage_outputs.imputation_qc_tsv),
        warnings_path: path_relative_to_repo(repo_root, &stage_outputs.warnings_json),
        qc_histograms_path: path_relative_to_repo(repo_root, &stage_outputs.qc_histograms_json),
        sample_missingness_row_count: qc_summary
            .get("sample_missingness")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        variant_missingness_row_count: qc_summary
            .get("variant_missingness")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        excluded_sample_count: qc_summary
            .get("excluded_samples")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        excluded_variant_count: qc_summary
            .get("excluded_variants")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::VCF_QC.to_string(),
        domain: "vcf".to_string(),
        tool_id: report.tool_id.clone(),
        execution_mode: "stage_runtime".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("filtered_vcf".to_string(), required_output(filter_row, "filtered_vcf")?),
            ("filtered_vcf_tbi".to_string(), required_output(filter_row, "filtered_vcf_tbi")?),
            ("stats_json".to_string(), required_output(stats_row, "stats_json")?),
        ]),
        outputs: BTreeMap::from([
            ("qc_report".to_string(), report.qc_summary_path.clone()),
            ("qc_stage_metrics".to_string(), report.qc_summary_path.clone()),
            ("qc_tables_tsv".to_string(), report.qc_tables_path.clone()),
        ]),
        metrics: BTreeMap::from([
            (
                "sample_missingness_row_count".to_string(),
                Value::from(u64::try_from(report.sample_missingness_row_count).unwrap_or(u64::MAX)),
            ),
            (
                "variant_missingness_row_count".to_string(),
                Value::from(
                    u64::try_from(report.variant_missingness_row_count).unwrap_or(u64::MAX),
                ),
            ),
            (
                "excluded_sample_count".to_string(),
                Value::from(u64::try_from(report.excluded_sample_count).unwrap_or(u64::MAX)),
            ),
            (
                "excluded_variant_count".to_string(),
                Value::from(u64::try_from(report.excluded_variant_count).unwrap_or(u64::MAX)),
            ),
        ]),
    })
}
