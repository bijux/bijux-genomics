//! Owner: bijux-dna-bench-workflows
//! Core germline stages vcf stats.
use super::super::{
    path_relative_to_repo, required_output, required_tool_id, run_stats_stage_real, BTreeMap,
    Context, CoreGermlineMicroPipelineRow, Path, Result, Value, VcfStatsParams,
    VcfStatsStageReport, PIPELINE_SAMPLE_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_vcf_stats_stage(
    repo_root: &Path,
    output_root: &Path,
    filter_row: &CoreGermlineMicroPipelineRow,
    tool_ids: &BTreeMap<String, String>,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.stats");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let filtered_vcf = repo_root.join(required_output(filter_row, "filtered_vcf")?);
    let stats = run_stats_stage_real(
        &filtered_vcf,
        &stage_root,
        &VcfStatsParams {
            sample_name: PIPELINE_SAMPLE_ID.to_string(),
            ..VcfStatsParams::default()
        },
    )?;
    let report = VcfStatsStageReport {
        schema_version: "bijux.bench.local_core_germline_micro_pipeline.vcf_stats.v1".to_string(),
        stage_id: stage::VCF_STATS.to_string(),
        tool_id: required_tool_id(tool_ids, stage::VCF_STATS)?,
        consumed_filter_report_path: required_output(filter_row, "filter_report_json")?,
        stats_json_path: path_relative_to_repo(repo_root, &stats.stats_json),
        bcftools_stats_path: path_relative_to_repo(repo_root, &stats.bcftools_stats_txt),
        variant_count: stats.metrics.variants_total,
        snp_count: stats.metrics.snps,
        indel_count: stats.metrics.indels,
        ti_tv: stats.metrics.ti_tv,
        sample_count: stats.metrics.sample_count,
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::VCF_STATS.to_string(),
        domain: "vcf".to_string(),
        tool_id: report.tool_id.clone(),
        execution_mode: "stage_runtime".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("filtered_vcf".to_string(), required_output(filter_row, "filtered_vcf")?),
            ("filtered_vcf_tbi".to_string(), required_output(filter_row, "filtered_vcf_tbi")?),
            ("filter_report_json".to_string(), required_output(filter_row, "filter_report_json")?),
        ]),
        outputs: BTreeMap::from([
            ("stats_json".to_string(), report.stats_json_path.clone()),
            ("stats_stage_metrics".to_string(), report.stats_json_path.clone()),
            ("bcftools_stats_txt".to_string(), report.bcftools_stats_path.clone()),
        ]),
        metrics: BTreeMap::from([
            ("variant_count".to_string(), Value::from(report.variant_count)),
            ("snp_count".to_string(), Value::from(report.snp_count)),
            ("indel_count".to_string(), Value::from(report.indel_count)),
            ("sample_count".to_string(), Value::from(report.sample_count)),
            ("ti_tv".to_string(), report.ti_tv.map_or(Value::Null, Value::from)),
        ]),
    })
}
