use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    screen::{
        ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier,
        TaxonomyDatabaseScope, TaxonomyReportFormat, SCREEN_TAXONOMY_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::STAGE_SCREEN_TAXONOMY;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_SCREEN_TAXONOMY;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

/// Build a screen plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_screen(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_screen_tool_list(std::slice::from_ref(&tool_id))?;
    let outputs = taxonomy_outputs(&tool.tool_id.0, out_dir)?;
    let (classifier, report_format, assignment_format) = classifier_contract(&tool.tool_id.0)?;
    let effective_params = ScreenEffectiveParams {
        schema_version: SCREEN_TAXONOMY_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: tool.resources.threads,
        contaminant_db: None,
        database_artifact_id: "taxonomy_db".to_string(),
        database_build_id: None,
        database_scope: TaxonomyDatabaseScope::ReadScreening,
        classifier,
        report_format,
        assignment_format,
        minimum_confidence: None,
        emit_unclassified: true,
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
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: None,
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 {
            template: tool.command.template.to_vec(),
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs,
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("screen_report_tsv"),
                    outputs.report.clone(),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("classification_report_json"),
                    outputs.assignments.clone(),
                    ArtifactRole::MetricsJson,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "out_dir": out_dir,
            "report": outputs.report,
            "assignments": outputs.assignments
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize screen effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn normalize_tools_with_allowlist(
    tools: &[String],
    allowlist: &[bijux_dna_core::ids::ToolId],
) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}

struct TaxonomyOutputs {
    report: std::path::PathBuf,
    assignments: std::path::PathBuf,
}

fn taxonomy_outputs(tool_id: &str, out_dir: &Path) -> Result<TaxonomyOutputs> {
    let outputs = match tool_id {
        "kraken2" => TaxonomyOutputs {
            report: out_dir.join("kraken2.report.tsv"),
            assignments: out_dir.join("kraken2.classifications.json"),
        },
        "krakenuniq" => TaxonomyOutputs {
            report: out_dir.join("krakenuniq.report.tsv"),
            assignments: out_dir.join("krakenuniq.classifications.json"),
        },
        "centrifuge" => TaxonomyOutputs {
            report: out_dir.join("centrifuge.report.tsv"),
            assignments: out_dir.join("centrifuge.classifications.json"),
        },
        "kaiju" => TaxonomyOutputs {
            report: out_dir.join("kaiju.summary.tsv"),
            assignments: out_dir.join("kaiju.classifications.json"),
        },
        _ => return Err(anyhow!("unsupported taxonomy screening tool: {tool_id}")),
    };
    Ok(outputs)
}

fn classifier_contract(
    tool_id: &str,
) -> Result<(
    TaxonomyClassifier,
    TaxonomyReportFormat,
    TaxonomyAssignmentFormat,
)> {
    let contract = match tool_id {
        "kraken2" => (
            TaxonomyClassifier::Kraken2,
            TaxonomyReportFormat::KrakenReport,
            TaxonomyAssignmentFormat::KrakenAssignments,
        ),
        "krakenuniq" => (
            TaxonomyClassifier::KrakenUniq,
            TaxonomyReportFormat::KrakenUniqReport,
            TaxonomyAssignmentFormat::KrakenUniqAssignments,
        ),
        "centrifuge" => (
            TaxonomyClassifier::Centrifuge,
            TaxonomyReportFormat::CentrifugeReport,
            TaxonomyAssignmentFormat::CentrifugeAssignments,
        ),
        "kaiju" => (
            TaxonomyClassifier::Kaiju,
            TaxonomyReportFormat::KaijuSummary,
            TaxonomyAssignmentFormat::KaijuAssignments,
        ),
        _ => return Err(anyhow!("unsupported taxonomy screening tool: {tool_id}")),
    };
    Ok(contract)
}
