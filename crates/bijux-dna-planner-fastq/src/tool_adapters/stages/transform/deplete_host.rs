use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    screen::{
        HostDepletionEffectiveParams, MappingReportFormat, ReadRetentionPolicy,
        ReferenceDecoyPolicy, ReferenceMaskingPolicy, ReferenceScope,
        HOST_DEPLETION_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_HOST;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_DEPLETE_HOST;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub type DepleteHostPlanOptions = crate::DepleteHostStageParams;

/// # Errors
/// Returns an error if any requested host-depletion tool is not admitted for `fastq.deplete_host`.
pub fn normalize_host_depletion_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

/// Build a host depletion plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_host_depletion(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    reference_index: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_host_depletion_with_index_backend(
        tool,
        r1,
        r2,
        reference_index,
        out_dir,
        &DepleteHostPlanOptions::baseline(),
        "bowtie2_build",
    )
}

/// Build a host depletion plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_host_depletion_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    reference_index: &Path,
    out_dir: &Path,
    options: &DepleteHostPlanOptions,
) -> Result<StagePlanV1> {
    plan_host_depletion_with_index_backend(
        tool,
        r1,
        r2,
        reference_index,
        out_dir,
        options,
        "bowtie2_build",
    )
}

/// Build a host depletion plan with explicit upstream reference-index provenance.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_host_depletion_with_index_backend(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    reference_index: &Path,
    out_dir: &Path,
    options: &DepleteHostPlanOptions,
    reference_index_backend: &str,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_host_depletion_tool_list(std::slice::from_ref(&tool_id))?;
    if (options.host_identity_threshold - 0.95).abs() > f64::EPSILON {
        return Err(anyhow!(
            "fastq.deplete_host with bowtie2 currently requires host_identity_threshold=0.95"
        ));
    }
    if !options.retain_unmapped_only {
        return Err(anyhow!(
            "fastq.deplete_host with bowtie2 currently requires retain_unmapped_only=true"
        ));
    }
    let report = out_dir.join("host_depletion_report.json");
    let raw_backend_report = out_dir.join("bowtie2.host.metrics.txt");
    let paired_mode = if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd };
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let effective_params = HostDepletionEffectiveParams {
        schema_version: HOST_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode,
        threads: effective_threads,
        reference_scope: ReferenceScope::Host,
        reference_catalog_id: "host_reference".to_string(),
        reference_index_artifact_id: "reference_index".to_string(),
        reference_index_backend: reference_index_backend.to_string(),
        reference_build_id: None,
        reference_digest: None,
        masking_policy: ReferenceMaskingPolicy::Unmasked,
        decoy_policy: ReferenceDecoyPolicy::None,
        decoy_catalog_id: None,
        identity_threshold: options.host_identity_threshold,
        retained_read_policy: ReadRetentionPolicy::KeepNonHostReads,
        emit_removed_reads: true,
        report_format: MappingReportFormat::Bowtie2MetricsFile,
        retain_unmapped_pairs: options.retain_unmapped_only && r2.is_some(),
    };
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    inputs.push(ArtifactRef::required(
        ArtifactId::from_static("reference_index"),
        reference_index.to_path_buf(),
        ArtifactRole::Index,
    ));
    let mut outputs = Vec::new();
    let mut params = serde_json::json!({
        "tool": tool.tool_id.0,
        "input_r1": r1,
        "reference_index": reference_index,
        "reference_index_backend": reference_index_backend,
        "host_identity_threshold": options.host_identity_threshold,
        "retain_unmapped_only": options.retain_unmapped_only,
        "threads": effective_threads,
        "report_json": report,
        "raw_backend_report": raw_backend_report,
        "raw_backend_report_format": "bowtie2_met_file",
    });
    if let Some(r2) = r2 {
        let output_r1 = out_dir.join("host_depleted_R1.fastq.gz");
        let output_r2 = out_dir.join("host_depleted_R2.fastq.gz");
        let removed_r1 = out_dir.join("removed_host_R1.fastq.gz");
        let removed_r2 = out_dir.join("removed_host_R2.fastq.gz");
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("host_depleted_reads_r1"),
            output_r1.clone(),
            ArtifactRole::Reads,
        ));
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("host_depleted_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("removed_host_reads_r1"),
            removed_r1.clone(),
            ArtifactRole::Reads,
        ));
        outputs.push(ArtifactRef::optional(
            ArtifactId::from_static("removed_host_reads_r2"),
            removed_r2.clone(),
            ArtifactRole::Reads,
        ));
        params["input_r2"] = serde_json::json!(r2);
        params["output_r1"] = serde_json::json!(output_r1);
        params["output_r2"] = serde_json::json!(output_r2);
        params["removed_host_r1"] = serde_json::json!(removed_r1);
        params["removed_host_r2"] = serde_json::json!(removed_r2);
    } else {
        let output = out_dir.join("host_depleted.fastq.gz");
        let removed = out_dir.join("removed_host.fastq.gz");
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("host_depleted_reads_r1"),
            output.clone(),
            ArtifactRole::Reads,
        ));
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("removed_host_reads_r1"),
            removed.clone(),
            ArtifactRole::Reads,
        ));
        params["output"] = serde_json::json!(output);
        params["removed_host_reads"] = serde_json::json!(removed);
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("host_depletion_report_json"),
        report.clone(),
        ArtifactRole::ReportJson,
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
            template: host_depletion_command(
                &tool.tool_id.0,
                r1,
                r2,
                reference_index,
                out_dir,
                raw_backend_report.as_path(),
                effective_threads,
            )?,
        },
        resources: tool.resources.clone(),
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params,
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize host depletion effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn host_depletion_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    reference_index: &Path,
    out_dir: &Path,
    raw_backend_report: &Path,
    threads: u32,
) -> Result<Vec<String>> {
    match tool_id {
        "bowtie2" => {
            let mut command = vec![
                "bowtie2".to_string(),
                "-x".to_string(),
                reference_index.display().to_string(),
                "--threads".to_string(),
                threads.to_string(),
                "-S".to_string(),
                "/dev/null".to_string(),
            ];
            if let Some(r2) = r2 {
                command.extend([
                    "-1".to_string(),
                    r1.display().to_string(),
                    "-2".to_string(),
                    r2.display().to_string(),
                    "--un-conc-gz".to_string(),
                    out_dir.join("host_depleted_R%.fastq.gz").display().to_string(),
                    "--al-conc-gz".to_string(),
                    out_dir.join("removed_host_R%.fastq.gz").display().to_string(),
                ]);
            } else {
                command.extend([
                    "-U".to_string(),
                    r1.display().to_string(),
                    "--un-gz".to_string(),
                    out_dir.join("host_depleted.fastq.gz").display().to_string(),
                    "--al-gz".to_string(),
                    out_dir.join("removed_host.fastq.gz").display().to_string(),
                ]);
            }
            command.extend(["--met-file".to_string(), raw_backend_report.display().to_string()]);
            Ok(command)
        }
        _ => Err(anyhow!("unsupported host depletion tool for stage planning: {tool_id}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolId};

    fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::new(tool_id.to_string()),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/dummy:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["echo".to_string(), tool_id.to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        }
    }

    #[test]
    fn host_depletion_tracks_explicit_reference_backend() -> Result<()> {
        let plan = plan_host_depletion_with_index_backend(
            &tool("bowtie2"),
            Path::new("reads_R1.fastq.gz"),
            None,
            Path::new("reference.index"),
            Path::new("out"),
            &DepleteHostPlanOptions::baseline(),
            "star",
        )?;

        assert_eq!(plan.effective_params["reference_index_backend"], "star");
        assert_eq!(plan.params["reference_index_backend"], "star");
        Ok(())
    }

    #[test]
    fn host_depletion_rejects_unsupported_identity_override() {
        let error = plan_host_depletion_with_index_backend(
            &tool("bowtie2"),
            Path::new("reads_R1.fastq.gz"),
            None,
            Path::new("reference.index"),
            Path::new("out"),
            &DepleteHostPlanOptions {
                host_identity_threshold: 0.97,
                ..DepleteHostPlanOptions::baseline()
            },
            "bowtie2_build",
        )
        .expect_err("unsupported host_identity_threshold must fail");

        assert!(error.to_string().contains("host_identity_threshold=0.95"));
    }

    #[test]
    fn host_depletion_separates_governed_report_from_raw_backend_metrics() -> Result<()> {
        let plan = plan_host_depletion_with_index_backend(
            &tool("bowtie2"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("reference.index"),
            Path::new("out"),
            &DepleteHostPlanOptions { threads: Some(8), ..DepleteHostPlanOptions::baseline() },
            "bowtie2_build",
        )?;

        assert_eq!(plan.params["threads"], 8);
        assert_eq!(plan.params["report_json"], "out/host_depletion_report.json");
        assert_eq!(plan.params["raw_backend_report"], "out/bowtie2.host.metrics.txt");
        assert_eq!(plan.params["raw_backend_report_format"], "bowtie2_met_file");
        assert_eq!(plan.effective_params["threads"], 8);
        assert!(plan
            .command
            .template
            .windows(2)
            .any(|window| window == ["--met-file", "out/bowtie2.host.metrics.txt"]));
        Ok(())
    }
}
