use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    umi::{
        FastqUmiParams, UmiDedupPolicy, UmiDownstreamPropagation, UmiExtractionLocation,
        UmiFailedExtractionPolicy, UmiGroupingPolicy, UmiReadNameTransform, UMI_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::umi_artifact_paths;
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
    let options = ExtractUmisPlanOptions {
        threads: None,
        umi_pattern: umi_pattern.map(ToOwned::to_owned),
        extraction_location: None,
        read_name_transform: None,
        failed_extraction_policy: None,
        grouping_policy: None,
        downstream_dedup_policy: None,
        downstream_propagation: None,
    };
    plan_umi_with_options(tool, r1, r2, out_dir, &options)
}

/// # Errors
/// Returns an error if the requested UMI extraction tool or options are unsupported, or if the
/// stage plan cannot be built.
#[allow(clippy::too_many_lines)]
pub fn plan_umi_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    options: &ExtractUmisPlanOptions,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_umi_tool_list(std::slice::from_ref(&tool_id))?;
    let artifact_paths = umi_artifact_paths(out_dir, true);
    let output_r1 = artifact_paths.reads_r1;
    let output_r2 = artifact_paths
        .reads_r2
        .ok_or_else(|| anyhow!("paired umi stage must declare an R2 output path"))?;
    let report_json = artifact_paths.report_json;
    let raw_backend_report = artifact_paths
        .raw_backend_report
        .ok_or_else(|| anyhow!("umi stage must declare a raw backend report path"))?;
    let umi_pattern = options.umi_pattern.as_deref().unwrap_or(DEFAULT_UMI_PATTERN);
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let extraction_location = parse_extraction_location(
        options.extraction_location.as_deref().unwrap_or("read1_prefix"),
    )?;
    let read_name_transform = parse_read_name_transform(
        options.read_name_transform.as_deref().unwrap_or("append_to_header"),
    )?;
    let failed_extraction_policy = parse_failed_extraction_policy(
        options.failed_extraction_policy.as_deref().unwrap_or("refuse_stage"),
    )?;
    let grouping_policy =
        parse_grouping_policy(options.grouping_policy.as_deref().unwrap_or("pair_aware"))?;
    let downstream_dedup_policy = parse_downstream_dedup_policy(
        options.downstream_dedup_policy.as_deref().unwrap_or("sequence_identity_recommended"),
    )?;
    let downstream_propagation = parse_downstream_propagation(
        options.downstream_propagation.as_deref().unwrap_or("header_and_report"),
    )?;
    let effective_params = FastqUmiParams {
        schema_version: UMI_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: effective_threads,
        umi_pattern: Some(umi_pattern.to_string()),
        extraction_location,
        read_name_transform,
        failed_extraction_policy,
        grouping_policy,
        downstream_dedup_policy,
        downstream_propagation,
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
            "umi_pattern": umi_pattern,
            "extraction_location": effective_params.extraction_location,
            "read_name_transform": effective_params.read_name_transform,
            "failed_extraction_policy": effective_params.failed_extraction_policy,
            "grouping_policy": effective_params.grouping_policy,
            "downstream_dedup_policy": effective_params.downstream_dedup_policy,
            "downstream_propagation": effective_params.downstream_propagation
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize umi effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
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

fn parse_extraction_location(value: &str) -> Result<UmiExtractionLocation> {
    match value.trim().to_ascii_lowercase().as_str() {
        "read1_prefix" => Ok(UmiExtractionLocation::Read1Prefix),
        "read2_prefix" => Ok(UmiExtractionLocation::Read2Prefix),
        "index_read" => Ok(UmiExtractionLocation::IndexRead),
        "header_tag" => Ok(UmiExtractionLocation::HeaderTag),
        _ => Err(anyhow!("unsupported extraction_location: {value}")),
    }
}

fn parse_read_name_transform(value: &str) -> Result<UmiReadNameTransform> {
    match value.trim().to_ascii_lowercase().as_str() {
        "append_to_header" => Ok(UmiReadNameTransform::AppendToHeader),
        "replace_header" => Ok(UmiReadNameTransform::ReplaceHeader),
        "none" => Ok(UmiReadNameTransform::None),
        _ => Err(anyhow!("unsupported read_name_transform: {value}")),
    }
}

fn parse_failed_extraction_policy(value: &str) -> Result<UmiFailedExtractionPolicy> {
    match value.trim().to_ascii_lowercase().as_str() {
        "refuse_stage" => Ok(UmiFailedExtractionPolicy::RefuseStage),
        "retain_unmodified" => Ok(UmiFailedExtractionPolicy::RetainUnmodified),
        "route_to_rejected" => Ok(UmiFailedExtractionPolicy::RouteToRejected),
        _ => Err(anyhow!("unsupported failed_extraction_policy: {value}")),
    }
}

fn parse_downstream_propagation(value: &str) -> Result<UmiDownstreamPropagation> {
    match value.trim().to_ascii_lowercase().as_str() {
        "header_only" => Ok(UmiDownstreamPropagation::HeaderOnly),
        "header_and_report" => Ok(UmiDownstreamPropagation::HeaderAndReport),
        _ => Err(anyhow!("unsupported downstream_propagation: {value}")),
    }
}

fn parse_grouping_policy(value: &str) -> Result<UmiGroupingPolicy> {
    match value.trim().to_ascii_lowercase().as_str() {
        "exact_sequence" => Ok(UmiGroupingPolicy::ExactSequence),
        "exact_header_tag" => Ok(UmiGroupingPolicy::ExactHeaderTag),
        "pair_aware" => Ok(UmiGroupingPolicy::PairAware),
        _ => Err(anyhow!("unsupported grouping_policy: {value}")),
    }
}

fn parse_downstream_dedup_policy(value: &str) -> Result<UmiDedupPolicy> {
    match value.trim().to_ascii_lowercase().as_str() {
        "observation_only" => Ok(UmiDedupPolicy::ObservationOnly),
        "sequence_identity_recommended" => Ok(UmiDedupPolicy::SequenceIdentityRecommended),
        "coordinate_aware_recommended" => Ok(UmiDedupPolicy::CoordinateAwareRecommended),
        _ => Err(anyhow!("unsupported downstream_dedup_policy: {value}")),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{plan_umi, plan_umi_with_options};
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

    #[test]
    fn plan_umi_surfaces_first_class_extraction_semantics() {
        let plan = plan_umi_with_options(
            &tool(),
            Path::new("reads_R1.fastq.gz"),
            Path::new("reads_R2.fastq.gz"),
            Path::new("out"),
            &crate::ExtractUmisStageParams {
                threads: Some(4),
                umi_pattern: Some("NNNNCCCC".to_string()),
                extraction_location: Some("read2_prefix".to_string()),
                read_name_transform: Some("append_to_header".to_string()),
                failed_extraction_policy: Some("retain_unmodified".to_string()),
                grouping_policy: Some("exact_header_tag".to_string()),
                downstream_dedup_policy: Some("coordinate_aware_recommended".to_string()),
                downstream_propagation: Some("header_only".to_string()),
            },
        )
        .expect("plan");
        assert_eq!(plan.params["extraction_location"], serde_json::json!("read2_prefix"));
        assert_eq!(plan.params["failed_extraction_policy"], serde_json::json!("retain_unmodified"));
        assert_eq!(plan.params["grouping_policy"], serde_json::json!("exact_header_tag"));
        assert_eq!(
            plan.params["downstream_dedup_policy"],
            serde_json::json!("coordinate_aware_recommended")
        );
        assert_eq!(
            plan.effective_params["downstream_propagation"],
            serde_json::json!("header_only")
        );
    }
}
