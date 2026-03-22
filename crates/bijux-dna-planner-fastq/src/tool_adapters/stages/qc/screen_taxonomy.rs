use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    screen::{
        ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyDatabaseScope,
        TaxonomyReportFormat, SCREEN_TAXONOMY_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::STAGE_SCREEN_TAXONOMY;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_SCREEN_TAXONOMY;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScreenPlanOptions {
    pub threads: Option<u32>,
}

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
    plan_screen_with_options(tool, r1, r2, out_dir, &ScreenPlanOptions::default())
}

/// Build a screen plan with explicit governed stage options.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_screen_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &ScreenPlanOptions,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_screen_tool_list(std::slice::from_ref(&tool_id))?;
    let outputs = taxonomy_outputs(&tool.tool_id.0, out_dir)?;
    let (classifier, report_format, assignment_format) = classifier_contract(&tool.tool_id.0)?;
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let effective_params = ScreenEffectiveParams {
        schema_version: SCREEN_TAXONOMY_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: effective_threads,
        contaminant_db: None,
        database_catalog_id: "taxonomy_reference".to_string(),
        database_artifact_id: "taxonomy_db".to_string(),
        database_build_id: None,
        database_digest: None,
        database_namespace: Some("read_screening".to_string()),
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
        stage_instance_id: Some(crate::tool_adapters::default_stage_instance_id(
            &STAGE_ID,
            &tool.tool_id,
        )),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 {
            template: tool.command.template.to_vec(),
        },
        resources: {
            let mut resources = tool.resources.clone();
            resources.threads = effective_threads;
            resources
        },
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
            "assignments": outputs.assignments,
            "threads": effective_threads,
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

#[cfg(test)]
mod tests {
    use super::{plan_screen_with_options, ScreenPlanOptions, STAGE_ID};
    use anyhow::Result;
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
        ToolVersion,
    };
    use std::path::Path;

    fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::new(tool_id),
            tool_version: ToolVersion::new("1.0.0"),
            image: ContainerImageRefV1 {
                registry: Some("ghcr.io".to_string()),
                repository: format!("bijux/{tool_id}"),
                tag: "latest".to_string(),
                digest: Some("sha256:test".to_string()),
            },
            command: CommandSpecV1 {
                template: vec![tool_id.to_string()],
            },
            resources: ToolConstraints {
                runtime: "local".to_string(),
                mem_gb: 4,
                tmp_gb: 1,
                threads: 4,
            },
        }
    }

    #[test]
    fn screen_plan_thread_override_updates_resources_and_effective_params() -> Result<()> {
        let plan = plan_screen_with_options(
            &tool("kraken2"),
            Path::new("reads_R1.fastq.gz"),
            None,
            Path::new("out"),
            &ScreenPlanOptions { threads: Some(12) },
        )?;
        assert_eq!(plan.stage_id, STAGE_ID);
        assert_eq!(plan.resources.threads, 12);
        assert_eq!(plan.params["threads"], serde_json::json!(12));
        assert_eq!(plan.effective_params["threads"], serde_json::json!(12));
        Ok(())
    }
}
