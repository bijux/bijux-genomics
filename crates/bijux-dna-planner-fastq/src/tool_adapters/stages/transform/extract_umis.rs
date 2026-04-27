use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{umi::FastqUmiParams, umi::UMI_SCHEMA_VERSION, PairedMode};
use bijux_dna_domain_fastq::STAGE_EXTRACT_UMIS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_EXTRACT_UMIS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);
const DEFAULT_UMI_PATTERN: &str = "NNNNNNNN";
pub type ExtractUmisPlanOptions = crate::ExtractUmisStageParams;

/// # Errors
/// Returns an error if any requested UMI extraction tool is not admitted for `fastq.extract_umis`.
pub fn normalize_umi_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

/// Build a UMI plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_umi(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    umi_pattern: Option<&str>,
) -> Result<StagePlanV1> {
    let options =
        ExtractUmisPlanOptions { threads: None, umi_pattern: umi_pattern.map(ToOwned::to_owned) };
    plan_umi_with_options(tool, r1, r2, out_dir, &options)
}

/// # Errors
/// Returns an error if the requested UMI extraction tool or options are unsupported, or if the
/// stage plan cannot be built.
pub fn plan_umi_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    options: &ExtractUmisPlanOptions,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_umi_tool_list(std::slice::from_ref(&tool_id))?;
    let output_r1 = out_dir.join("umi_tools.r1.fastq.gz");
    let output_r2 = out_dir.join("umi_tools.r2.fastq.gz");
    let report_json = out_dir.join("umi_report.json");
    let raw_backend_report = out_dir.join("umi_tools.extract.log");
    let umi_pattern = options.umi_pattern.as_deref().unwrap_or(DEFAULT_UMI_PATTERN);
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let effective_params = FastqUmiParams {
        schema_version: UMI_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: effective_threads,
        umi_pattern: Some(umi_pattern.to_string()),
    };
    let mut resources = tool.resources.clone();
    resources.threads = effective_threads;
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
            template: crate::tool_adapters::template_render::render_command_template(
                &tool.command.template,
                &[
                    ("reads_r1", Some(r1.display().to_string())),
                    ("reads_r2", Some(r2.display().to_string())),
                    ("umi_reads_r1", Some(output_r1.display().to_string())),
                    ("umi_reads_r2", Some(output_r2.display().to_string())),
                    ("report_json", Some(report_json.display().to_string())),
                    ("raw_backend_report", Some(raw_backend_report.display().to_string())),
                    ("umi_pattern", Some(umi_pattern.to_string())),
                ],
            )?,
        },
        resources,
        io: StageIO {
            inputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("reads_r1"),
                    r1.to_path_buf(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("reads_r2"),
                    r2.to_path_buf(),
                    ArtifactRole::Reads,
                ),
            ],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("umi_reads_r1"),
                    output_r1.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("umi_reads_r2"),
                    output_r2.clone(),
                    ArtifactRole::Reads,
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
            "r1": r1,
            "r2": r2,
            "out_dir": out_dir,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report_json,
            "raw_backend_report": raw_backend_report,
            "raw_backend_report_format": "umi_tools_log",
            "threads": effective_threads,
            "umi_pattern": umi_pattern
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize umi effective params: {error}"))?,
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

#[cfg(test)]
mod tests {
    use super::plan_umi;
    use bijux_dna_core::id_catalog;
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
    };
    use std::path::Path;

    fn tool() -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::from_static(id_catalog::TOOL_UMI_TOOLS),
            tool_version: "test".to_string(),
            image: ContainerImageRefV1 { image: "example/umi_tools".to_string(), digest: None },
            command: CommandSpecV1 {
                template: vec![
                    "umi_tools".to_string(),
                    "extract".to_string(),
                    "--stdin".to_string(),
                    "{{reads_r1}}".to_string(),
                    "--stdout".to_string(),
                    "{{umi_reads_r1}}".to_string(),
                    "--read2-in".to_string(),
                    "{{reads_r2}}".to_string(),
                    "--read2-out".to_string(),
                    "{{umi_reads_r2}}".to_string(),
                    "--bc-pattern".to_string(),
                    "{{umi_pattern}}".to_string(),
                    "--log".to_string(),
                    "{{raw_backend_report}}".to_string(),
                ],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 2,
            },
        }
    }

    #[test]
    fn plan_umi_renders_command_placeholders() {
        let plan = plan_umi(
            &tool(),
            Path::new("reads_R1.fastq.gz"),
            Path::new("reads_R2.fastq.gz"),
            Path::new("out"),
            Some("NNNNCCCC"),
        )
        .expect("plan");
        assert!(plan.command.template.iter().any(|token| token == "reads_R1.fastq.gz"));
        assert!(plan.command.template.iter().any(|token| token == "out/umi_tools.extract.log"));
        assert!(plan.command.template.iter().any(|token| token == "NNNNCCCC"));
    }
}
