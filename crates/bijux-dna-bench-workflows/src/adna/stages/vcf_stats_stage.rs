//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages vcf stats stage.
use super::super::{
    fs, path_relative_to_repo, required_output, run_stats_stage_real, succeeded_row,
    AdnaMicroPipelineRow, AdnaStatsReport, BTreeMap, Context, Path, Result, Value,
    ADNA_PIPELINE_SAMPLE_ID, VCF_STATS_TOOL_ID,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_vcf_stats_stage_row(
    repo_root: &Path,
    output_root: &Path,
    damage_filter_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.stats");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let filtered_vcf = repo_root.join(required_output(damage_filter_row, "damage_filtered_vcf")?);
    let outputs = run_stats_stage_real(
        &filtered_vcf,
        &stage_root,
        &bijux_dna_domain_vcf::params::VcfStatsParams {
            sample_name: ADNA_PIPELINE_SAMPLE_ID.to_string(),
            ..bijux_dna_domain_vcf::params::VcfStatsParams::default()
        },
    )?;
    let stats_json: bijux_dna_domain_vcf::VcfStatsMetricsV1 = serde_json::from_slice(
        &fs::read(&outputs.stats_json)
            .with_context(|| format!("read {}", outputs.stats_json.display()))?,
    )
    .with_context(|| format!("parse {}", outputs.stats_json.display()))?;
    let report = AdnaStatsReport {
        schema_version: "bijux.bench.local_adna_micro_pipeline.stats.v1".to_string(),
        stage_id: stage::VCF_STATS.to_string(),
        tool_id: VCF_STATS_TOOL_ID.to_string(),
        stats_json_path: path_relative_to_repo(repo_root, &outputs.stats_json),
        bcftools_stats_path: path_relative_to_repo(repo_root, &outputs.bcftools_stats_txt),
        variant_count: stats_json.variants_total,
        snp_count: stats_json.snps,
        indel_count: stats_json.indels,
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::VCF_STATS,
            domain: "vcf",
            tool_id: VCF_STATS_TOOL_ID,
            execution_mode: "governed_runtime",
        },
        path_relative_to_repo(repo_root, &report_path),
        &report.schema_version,
        BTreeMap::from([
            (
                "damage_filtered_vcf".to_string(),
                required_output(damage_filter_row, "damage_filtered_vcf")?,
            ),
            (
                "damage_filtered_vcf_tbi".to_string(),
                required_output(damage_filter_row, "damage_filtered_vcf_tbi")?,
            ),
        ]),
        BTreeMap::from([
            ("stats_json".to_string(), path_relative_to_repo(repo_root, &outputs.stats_json)),
            (
                "bcftools_stats_txt".to_string(),
                path_relative_to_repo(repo_root, &outputs.bcftools_stats_txt),
            ),
        ]),
        BTreeMap::from([
            ("variant_count".to_string(), Value::from(report.variant_count)),
            ("snp_count".to_string(), Value::from(report.snp_count)),
            ("indel_count".to_string(), Value::from(report.indel_count)),
        ]),
        "summarized the retained ancient-DNA pseudohaploid calls after damage filtering"
            .to_string(),
    ))
}
