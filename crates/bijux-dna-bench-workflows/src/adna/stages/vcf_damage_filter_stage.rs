//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages vcf damage filter stage.
use super::super::{
    count_json_u64, count_vcf_variants, fs, path_relative_to_repo, required_output,
    run_damage_filter_stage, succeeded_row, AdnaDamageFilterReport, AdnaMicroPipelineRow, BTreeMap,
    Context, DamageFilterStageParams, DamageUdgRegime, Path, Result, Value,
    VCF_DAMAGE_FILTER_TOOL_ID,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_vcf_damage_filter_stage_row(
    repo_root: &Path,
    output_root: &Path,
    pseudohaploid_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.damage_filter");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let input_vcf = repo_root.join(required_output(pseudohaploid_row, "pseudohaploid_vcf")?);
    let outputs = run_damage_filter_stage(
        &input_vcf,
        &stage_root,
        &DamageFilterStageParams {
            min_qual: 0.0,
            max_damage_ratio: 0.35,
            udg_regime: DamageUdgRegime::NonUdg,
            strict_regime: false,
        },
    )?;
    let summary_json: serde_json::Value = serde_json::from_slice(
        &fs::read(&outputs.damage_filter_summary_json)
            .with_context(|| format!("read {}", outputs.damage_filter_summary_json.display()))?,
    )
    .with_context(|| format!("parse {}", outputs.damage_filter_summary_json.display()))?;
    let counts_json: serde_json::Value = serde_json::from_slice(
        &fs::read(&outputs.damage_filter_counts_json)
            .with_context(|| format!("read {}", outputs.damage_filter_counts_json.display()))?,
    )
    .with_context(|| format!("parse {}", outputs.damage_filter_counts_json.display()))?;
    let retained_variants = count_vcf_variants(&outputs.filtered_vcf)?;
    let report = AdnaDamageFilterReport {
        schema_version: "bijux.bench.local_adna_micro_pipeline.damage_filter.v1".to_string(),
        stage_id: stage::VCF_DAMAGE_FILTER.to_string(),
        tool_id: VCF_DAMAGE_FILTER_TOOL_ID.to_string(),
        filtered_vcf_path: path_relative_to_repo(repo_root, &outputs.filtered_vcf),
        filtered_tbi_path: path_relative_to_repo(repo_root, &outputs.filtered_tbi),
        summary_path: path_relative_to_repo(repo_root, &outputs.damage_filter_summary_json),
        counts_path: path_relative_to_repo(repo_root, &outputs.damage_filter_counts_json),
        retained_variants,
        damage_ratio_filtered_variants: count_json_u64(
            &counts_json,
            "/counts/damage_ratio_exceeded",
        ),
        terminal_damage_filtered_variants: count_json_u64(
            &counts_json,
            "/counts/terminal_damage_filtered",
        ),
        proxy_only_mode: summary_json
            .pointer("/prefilter/read_position_signal/proxy_used_records")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            > 0,
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::VCF_DAMAGE_FILTER,
            domain: "vcf",
            tool_id: VCF_DAMAGE_FILTER_TOOL_ID,
            execution_mode: "governed_runtime",
        },
        path_relative_to_repo(repo_root, &report_path),
        &report.schema_version,
        BTreeMap::from([
            (
                "pseudohaploid_vcf".to_string(),
                required_output(pseudohaploid_row, "pseudohaploid_vcf")?,
            ),
            (
                "pseudohaploid_vcf_tbi".to_string(),
                required_output(pseudohaploid_row, "pseudohaploid_vcf_tbi")?,
            ),
        ]),
        BTreeMap::from([
            (
                "damage_filtered_vcf".to_string(),
                path_relative_to_repo(repo_root, &outputs.filtered_vcf),
            ),
            (
                "damage_filtered_vcf_tbi".to_string(),
                path_relative_to_repo(repo_root, &outputs.filtered_tbi),
            ),
            (
                "damage_bias_audit_report".to_string(),
                path_relative_to_repo(repo_root, &report_path),
            ),
            (
                "damage_filter_metrics".to_string(),
                path_relative_to_repo(repo_root, &outputs.damage_filter_counts_json),
            ),
        ]),
        BTreeMap::from([
            ("retained_variants".to_string(), Value::from(retained_variants)),
            (
                "damage_ratio_filtered_variants".to_string(),
                Value::from(report.damage_ratio_filtered_variants),
            ),
            (
                "terminal_damage_filtered_variants".to_string(),
                Value::from(report.terminal_damage_filtered_variants),
            ),
        ]),
        "applied damage-aware proxy filtering to the pseudohaploid VCF".to_string(),
    ))
}
