use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    screen::{
        RrnaEffectiveParams, RrnaReportFormat, RrnaScreeningEngine, RRNA_DEPLETION_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::STAGE_DEPLETE_RRNA;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_DEPLETE_RRNA;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub type DepleteRrnaPlanOptions = crate::DepleteRrnaStageParams;

pub fn normalize_rrna_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool {tool}"));
        }
    }
    Ok(normalized)
}

/// Build an rRNA screening plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_rrna(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_rrna_with_options(tool, r1, r2, out_dir, &DepleteRrnaPlanOptions::default())
}

/// Build an rRNA screening plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_rrna_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &DepleteRrnaPlanOptions,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_rrna_tool_list(std::slice::from_ref(&tool_id))?;
    if options.rrna_db.trim().is_empty() {
        return Err(anyhow!("rrna_db must be provided for {}", tool.tool_id));
    }
    if (options.min_identity - 0.95).abs() > f64::EPSILON {
        return Err(anyhow!(
            "sortmerna does not support governed min_identity overrides; requested {}",
            options.min_identity
        ));
    }
    let filtered_reads_r1 = if r2.is_some() {
        out_dir.join("rrna_filtered_R1.fastq.gz")
    } else {
        out_dir.join("rrna_filtered.fastq.gz")
    };
    let filtered_reads_r2 = r2.map(|_| out_dir.join("rrna_filtered_R2.fastq.gz"));
    let report = out_dir.join("rrna_report.tsv");
    let metrics = out_dir.join("rrna_report.json");
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let effective_params = RrnaEffectiveParams {
        schema_version: RRNA_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: effective_threads,
        contaminant_db: Some(options.rrna_db.clone()),
        database_artifact_id: options.rrna_db.clone(),
        database_build_id: None,
        screening_engine: RrnaScreeningEngine::Sortmerna,
        report_format: RrnaReportFormat::SummaryTsvAndJson,
        emit_removed_reads: false,
    };
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
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("rrna_filtered_reads_r1"),
        filtered_reads_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(filtered_reads_r2) = &filtered_reads_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("rrna_filtered_reads_r2"),
            filtered_reads_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("rrna_report_tsv"),
        report.clone(),
        ArtifactRole::SummaryTsv,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("rrna_report_json"),
        metrics.clone(),
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
            template: rrna_command(
                &tool.tool_id.0,
                r1,
                r2,
                &filtered_reads_r1,
                filtered_reads_r2.as_deref(),
                &report,
                &metrics,
                effective_threads,
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
            "rrna_db": options.rrna_db,
            "min_identity": options.min_identity,
            "threads": effective_threads,
            "filtered_reads_r1": filtered_reads_r1,
            "filtered_reads_r2": filtered_reads_r2,
            "report_tsv": report,
            "report_json": metrics
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize rrna effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn rrna_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    filtered_reads_r1: &Path,
    filtered_reads_r2: Option<&Path>,
    report_tsv: &Path,
    report_json: &Path,
    threads: u32,
    options: &DepleteRrnaPlanOptions,
) -> Result<Vec<String>> {
    match tool_id {
        "sortmerna" => {
            let mut command = vec![
                "sortmerna".to_string(),
                "--ref".to_string(),
                options.rrna_db.clone(),
                "--reads".to_string(),
                r1.display().to_string(),
                "--other".to_string(),
                filtered_reads_r1.display().to_string(),
                "--workdir".to_string(),
                report_json
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .display()
                    .to_string(),
                "--threads".to_string(),
                threads.to_string(),
                "--fastx".to_string(),
                "--out2".to_string(),
                "--log".to_string(),
            ];
            if let Some(r2) = r2 {
                command.push("--reads".to_string());
                command.push(r2.display().to_string());
            }
            if let Some(filtered_reads_r2) = filtered_reads_r2 {
                command.push("--paired_out".to_string());
                command.push(filtered_reads_r2.display().to_string());
            }
            command.push("--report".to_string());
            command.push(report_tsv.display().to_string());
            Ok(command)
        }
        _ => Err(anyhow!("unsupported tool {tool_id}")),
    }
}
