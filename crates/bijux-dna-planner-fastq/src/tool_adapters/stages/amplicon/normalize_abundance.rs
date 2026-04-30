use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::edna::AbundanceNormalizationEffectiveParams;
use bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_ABUNDANCE;
use bijux_dna_domain_fastq::{
    NormalizeAbundanceReportV1, NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION,
};
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_NORMALIZE_ABUNDANCE;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone)]
pub struct NormalizeAbundancePlanOptions {
    pub method: String,
}

impl Default for NormalizeAbundancePlanOptions {
    fn default() -> Self {
        Self { method: "relative_abundance".to_string() }
    }
}

/// # Errors
/// Returns an error if abundance normalization cannot be planned for the requested tool.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    abundance_table: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_with_options(tool, abundance_table, out_dir, &NormalizeAbundancePlanOptions::default())
}

/// # Errors
/// Returns an error if the requested abundance-normalization method is unsupported or the stage
/// plan cannot be built.
pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    abundance_table: &Path,
    out_dir: &Path,
    options: &NormalizeAbundancePlanOptions,
) -> Result<StagePlanV1> {
    let effective_params = effective_params_for_method(&options.method)?;
    let output_tsv = out_dir.join("abundance_normalized.tsv");
    let report_json = out_dir.join("normalize_abundance_report.json");
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: Some(crate::tool_adapters::default_stage_instance_id(
            &STAGE_ID,
            &tool.tool_id,
        )),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: normalize_abundance_command(
                abundance_table,
                &output_tsv,
                &report_json,
                &effective_params,
                tool.tool_id.as_str(),
            )?,
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("abundance_table"),
                abundance_table.to_path_buf(),
                ArtifactRole::SummaryTsv,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("normalized_abundance_tsv"),
                    output_tsv.clone(),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("report_json"),
                    report_json.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_table": abundance_table,
            "normalized_abundance_tsv": output_tsv,
            "report_json": report_json,
            "method": effective_params.method,
        }),
        effective_params: serde_json::to_value(&effective_params)?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::new(
            PlanReasonKind::Default,
            "amplicon abundance normalization",
        ),
    })
}

fn effective_params_for_method(method: &str) -> Result<AbundanceNormalizationEffectiveParams> {
    let (normalized_value_column, compositional_rule, scale_factor) = match method {
        "relative_abundance" => {
            ("normalized_abundance".to_string(), "per_sample_sum_to_one".to_string(), None)
        }
        "counts_per_million" => (
            "counts_per_million".to_string(),
            "per_sample_sum_to_one_million".to_string(),
            Some(1_000_000.0),
        ),
        _ => return Err(anyhow!("unsupported fastq.normalize_abundance method `{method}`")),
    };
    Ok(AbundanceNormalizationEffectiveParams {
        schema_version: bijux_dna_domain_fastq::params::edna::EDNA_SCHEMA_VERSION.to_string(),
        method: method.to_string(),
        expected_columns: vec![
            "sample_id".to_string(),
            "feature_id".to_string(),
            "abundance".to_string(),
        ],
        input_value_column: "abundance".to_string(),
        normalized_value_column,
        compositional_rule,
        scale_factor,
        report_artifact: "report_json".to_string(),
    })
}

fn normalize_abundance_command(
    input_table: &Path,
    output_tsv: &Path,
    report_json: &Path,
    effective_params: &AbundanceNormalizationEffectiveParams,
    tool_id: &str,
) -> Result<Vec<String>> {
    let report = NormalizeAbundanceReportV1 {
        schema_version: NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.as_str().to_string(),
        stage_id: STAGE_ID.as_str().to_string(),
        tool_id: tool_id.to_string(),
        method: effective_params.method.clone(),
        input_table: input_table.display().to_string(),
        normalized_abundance_tsv: output_tsv.display().to_string(),
        expected_columns: effective_params.expected_columns.clone(),
        input_value_column: effective_params.input_value_column.clone(),
        normalized_value_column: effective_params.normalized_value_column.clone(),
        compositional_rule: effective_params.compositional_rule.clone(),
        scale_factor: effective_params.scale_factor,
        table_rows: 0,
        sample_count: 0,
        feature_count: 0,
        zero_fraction: 0.0,
        per_sample_sums: Vec::new(),
        runtime_s: None,
        memory_mb: None,
        raw_backend_report: None,
        raw_backend_report_format: None,
        used_fallback: false,
        backend_metrics: None,
    };
    let scale_factor = effective_params.scale_factor.unwrap_or(1.0);
    let script = format!(
        "set -euo pipefail\nawk -v method={method} -v outcol={outcol} -v scale={scale} 'BEGIN {{ FS=OFS=\"\\t\" }} NR==1 {{ if ($1 != \"sample_id\" || $2 != \"feature_id\" || $3 != \"abundance\") {{ exit 64 }}; next }} {{ rows[++n]=$0; total[$1]+=$3 }} END {{ print \"sample_id\", \"feature_id\", outcol; for (i = 1; i <= n; i++) {{ split(rows[i], cols, FS); if (method == \"relative_abundance\") {{ value = total[cols[1]] > 0 ? cols[3] / total[cols[1]] : 0 }} else if (method == \"counts_per_million\") {{ value = total[cols[1]] > 0 ? (cols[3] * scale) / total[cols[1]] : 0 }} else {{ exit 65 }}; printf \"%s\\t%s\\t%.6f\\n\", cols[1], cols[2], value }} }}' {input} > {output}\nprintf '%s\\n' {report} > {report_path}\n",
        method = shell_quote_str(&effective_params.method),
        outcol = shell_quote_str(&effective_params.normalized_value_column),
        scale = scale_factor,
        input = shell_quote_path(input_table),
        output = shell_quote_path(output_tsv),
        report = shell_quote_str(
            &serde_json::to_string(&report)
                .map_err(|error| anyhow!("serialize normalize abundance report: {error}"))?
        ),
        report_path = shell_quote_path(report_json),
    );
    Ok(vec!["bash".to_string(), "-lc".to_string(), script])
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
