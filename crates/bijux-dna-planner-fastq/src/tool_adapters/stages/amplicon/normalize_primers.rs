#![allow(clippy::format_push_string, clippy::too_many_arguments, clippy::uninlined_format_args)]

use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::edna::{
    PrimerNormalizationEffectiveParams, EDNA_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_PRIMERS;
use bijux_dna_domain_fastq::{NormalizePrimersReportV1, NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION};
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_NORMALIZE_PRIMERS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone)]
pub struct NormalizePrimersPlanOptions {
    pub primer_set_id: String,
    pub marker_id: Option<String>,
    pub primer_fasta: Option<std::path::PathBuf>,
    pub orientation_policy: String,
    pub max_mismatch_rate: f64,
    pub min_overlap_bp: u32,
    pub strict_5p_anchor: bool,
    pub allow_iupac_codes: bool,
}

impl Default for NormalizePrimersPlanOptions {
    fn default() -> Self {
        Self {
            primer_set_id: "default".to_string(),
            marker_id: None,
            primer_fasta: None,
            orientation_policy: "normalize_to_forward_primer".to_string(),
            max_mismatch_rate: 0.10,
            min_overlap_bp: 10,
            strict_5p_anchor: true,
            allow_iupac_codes: true,
        }
    }
}

/// # Errors
/// Returns an error if primer normalization cannot be planned for the requested tool.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_with_options(tool, r1, r2, out_dir, &NormalizePrimersPlanOptions::default())
}

/// # Errors
/// Returns an error if the requested primer-normalization options are unsupported or the stage
/// plan cannot be built.
pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &NormalizePrimersPlanOptions,
) -> Result<StagePlanV1> {
    let output_r1 = if r2.is_some() {
        out_dir.join("R1.primer_normalized.fastq.gz")
    } else {
        out_dir.join("primer_normalized.fastq.gz")
    };
    let output_r2 = r2.map(|_| out_dir.join("R2.primer_normalized.fastq.gz"));
    let report_json = out_dir.join("normalize_primers_report.json");
    let orientation_report = out_dir.join("primer_orientation.tsv");
    let primer_stats = out_dir.join("primer_stats.json");
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    if let Some(r2) = r2 {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
    if let Some(primer_fasta) = &options.primer_fasta {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("primer_fasta"),
            primer_fasta.clone(),
            ArtifactRole::Reference,
        ));
    }
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("normalized_reads_r1"),
        output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("normalized_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        report_json.clone(),
        ArtifactRole::ReportJson,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("primer_orientation_report"),
        orientation_report.clone(),
        ArtifactRole::SummaryTsv,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("primer_stats_json"),
        primer_stats.clone(),
        ArtifactRole::MetricsJson,
    ));

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
            template: normalize_primers_command(
                &tool.tool_id.0,
                r1,
                r2,
                &output_r1,
                output_r2.as_deref(),
                &report_json,
                &orientation_report,
                &primer_stats,
                options,
            )?,
        },
        resources: tool.resources.clone(),
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report_json,
            "primer_orientation_report": orientation_report,
            "primer_stats_json": primer_stats,
            "primer_set_id": options.primer_set_id,
            "marker_id": options.marker_id,
            "primer_fasta": options.primer_fasta,
            "orientation_policy": options.orientation_policy,
            "max_mismatch_rate": options.max_mismatch_rate,
            "min_overlap_bp": options.min_overlap_bp,
            "strict_5p_anchor": options.strict_5p_anchor,
            "allow_iupac_codes": options.allow_iupac_codes,
            "raw_backend_report": primer_stats,
            "raw_backend_report_format": match &*tool.tool_id.0 {
                "cutadapt" => Some("cutadapt_json"),
                _ => None,
            },
        }),
        effective_params: serde_json::to_value(PrimerNormalizationEffectiveParams {
            schema_version: EDNA_SCHEMA_VERSION.to_string(),
            paired_mode: PairedMode::from_has_r2(r2.is_some()),
            threads: Some(tool.resources.threads),
            orientation_policy: options.orientation_policy.clone(),
            primer_set_id: options.primer_set_id.clone(),
            marker_id: options.marker_id.clone(),
            primer_fasta: options.primer_fasta.as_ref().map(|path| path.display().to_string()),
            max_mismatch_rate: options.max_mismatch_rate,
            min_overlap_bp: options.min_overlap_bp,
            strict_5p_anchor: options.strict_5p_anchor,
            allow_iupac_codes: options.allow_iupac_codes,
        })?,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "amplicon primer normalization"),
    })
}

fn normalize_primers_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    orientation_report: &Path,
    primer_stats: &Path,
    options: &NormalizePrimersPlanOptions,
) -> Result<Vec<String>> {
    let governed_report = build_governed_normalize_primers_report(
        tool_id,
        r1,
        r2,
        output_r1,
        output_r2,
        orientation_report,
        primer_stats,
        options,
    )?;
    match tool_id {
        "cutadapt" => {
            let primer_arg = options.primer_fasta.as_ref().map_or_else(
                || "file:primers.fa".to_string(),
                |path| format!("file:{}", path.display()),
            );
            let mut script = format!(
                "set -euo pipefail\ncutadapt -g {} --overlap {} --error-rate {} --revcomp --info-file {} --json {} -o {}",
                shell_quote_str(&primer_arg),
                options.min_overlap_bp,
                options.max_mismatch_rate,
                shell_quote_path(orientation_report),
                shell_quote_path(primer_stats),
                shell_quote_path(output_r1),
            );
            if let Some(output_r2) = output_r2 {
                script.push_str(&format!(" -p {}", shell_quote_path(output_r2)));
            }
            script.push_str(&format!(" {}", shell_quote_path(r1)));
            if let Some(r2) = r2 {
                script.push_str(&format!(" {}", shell_quote_path(r2)));
            }
            script.push_str(&format!(
                "\nprintf '%s\\n' {} > {}\n",
                shell_quote_str(&governed_report),
                shell_quote_path(report_json),
            ));
            Ok(vec!["bash".to_string(), "-lc".to_string(), script])
        }
        _ => Err(anyhow!("unsupported primer normalization tool for stage planning: {tool_id}")),
    }
}

fn build_governed_normalize_primers_report(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    orientation_report: &Path,
    primer_stats: &Path,
    options: &NormalizePrimersPlanOptions,
) -> Result<String> {
    let report = NormalizePrimersReportV1 {
        schema_version: NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.as_str().to_string(),
        stage_id: STAGE_ID.as_str().to_string(),
        tool_id: tool_id.to_string(),
        paired_mode: PairedMode::from_has_r2(r2.is_some()),
        primer_set_id: options.primer_set_id.clone(),
        marker_id: options.marker_id.clone(),
        primer_fasta: options.primer_fasta.as_ref().map(|path| path.display().to_string()),
        orientation_policy: options.orientation_policy.clone(),
        max_mismatch_rate: options.max_mismatch_rate,
        min_overlap_bp: options.min_overlap_bp,
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        reads_in: None,
        reads_out: None,
        bases_in: None,
        bases_out: None,
        pairs_in: None,
        pairs_out: None,
        primer_trimmed_reads: None,
        primer_trimmed_fraction: None,
        orientation_forward_fraction: None,
        primer_orientation_report: orientation_report.display().to_string(),
        primer_stats_json: primer_stats.display().to_string(),
        raw_backend_report: Some(primer_stats.display().to_string()),
        raw_backend_report_format: match tool_id {
            "cutadapt" => Some("cutadapt_json".to_string()),
            _ => None,
        },
        runtime_s: None,
        memory_mb: None,
        used_fallback: false,
        backend_metrics: None,
    };
    serde_json::to_string(&report)
        .map_err(|error| anyhow!("serialize normalize primers governed report: {error}"))
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
