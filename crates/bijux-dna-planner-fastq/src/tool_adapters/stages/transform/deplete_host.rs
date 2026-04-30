use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::host_depletion_artifact_paths;
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

struct HostDepletionPaths {
    report: std::path::PathBuf,
    raw_backend_report: std::path::PathBuf,
    output_r1: std::path::PathBuf,
    output_r2: Option<std::path::PathBuf>,
    removed_r1: std::path::PathBuf,
    removed_r2: Option<std::path::PathBuf>,
}

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
    ensure_host_depletion_options(options)?;
    let paired = r2.is_some();
    let paths = host_depletion_paths(out_dir, paired);
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let effective_params = host_depletion_effective_params(
        paired,
        effective_threads,
        reference_index_backend,
        options,
    );
    let inputs = host_depletion_inputs(r1, r2, reference_index);
    let outputs = host_depletion_outputs(&paths);
    let params = host_depletion_params(&HostDepletionParamContext {
        tool_id: &tool.tool_id.0,
        r1,
        r2,
        reference_index,
        reference_index_backend,
        options,
        effective_threads,
        paths: &paths,
    });
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
        command: CommandSpecV1 {
            template: host_depletion_command(
                &tool.tool_id.0,
                r1,
                r2,
                reference_index,
                out_dir,
                paths.raw_backend_report.as_path(),
                effective_threads,
            )?,
        },
        resources,
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params,
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize host depletion effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn ensure_host_depletion_options(options: &DepleteHostPlanOptions) -> Result<()> {
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
    Ok(())
}

fn host_depletion_paths(out_dir: &Path, paired: bool) -> HostDepletionPaths {
    let named = host_depletion_artifact_paths(out_dir, paired);
    HostDepletionPaths {
        report: named.report_json,
        raw_backend_report: named.raw_backend_report,
        output_r1: named.retained_r1,
        output_r2: named.retained_r2,
        removed_r1: named.rejected_r1,
        removed_r2: named.rejected_r2,
    }
}

fn host_depletion_effective_params(
    paired: bool,
    effective_threads: u32,
    reference_index_backend: &str,
    options: &DepleteHostPlanOptions,
) -> HostDepletionEffectiveParams {
    HostDepletionEffectiveParams {
        schema_version: HOST_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
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
        retain_unmapped_pairs: options.retain_unmapped_only && paired,
    }
}

fn host_depletion_inputs(r1: &Path, r2: Option<&Path>, reference_index: &Path) -> Vec<ArtifactRef> {
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
    if let Some(r2) = r2 {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
    inputs
}

fn host_depletion_outputs(paths: &HostDepletionPaths) -> Vec<ArtifactRef> {
    let mut outputs = vec![
        ArtifactRef::required(
            ArtifactId::from_static("host_depleted_reads_r1"),
            paths.output_r1.clone(),
            ArtifactRole::Reads,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("removed_host_reads_r1"),
            paths.removed_r1.clone(),
            ArtifactRole::Reads,
        ),
    ];
    if let Some(output_r2) = &paths.output_r2 {
        outputs.insert(
            1,
            ArtifactRef::required(
                ArtifactId::from_static("host_depleted_reads_r2"),
                output_r2.clone(),
                ArtifactRole::Reads,
            ),
        );
    }
    if let Some(removed_r2) = &paths.removed_r2 {
        outputs.push(ArtifactRef::optional(
            ArtifactId::from_static("removed_host_reads_r2"),
            removed_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("host_depletion_report_json"),
        paths.report.clone(),
        ArtifactRole::ReportJson,
    ));
    outputs
}

struct HostDepletionParamContext<'a> {
    tool_id: &'a str,
    r1: &'a Path,
    r2: Option<&'a Path>,
    reference_index: &'a Path,
    reference_index_backend: &'a str,
    options: &'a DepleteHostPlanOptions,
    effective_threads: u32,
    paths: &'a HostDepletionPaths,
}

fn host_depletion_params(context: &HostDepletionParamContext<'_>) -> serde_json::Value {
    let mut params = serde_json::json!({
        "tool": context.tool_id,
        "input_r1": context.r1,
        "reference_index": context.reference_index,
        "reference_index_backend": context.reference_index_backend,
        "host_identity_threshold": context.options.host_identity_threshold,
        "retain_unmapped_only": context.options.retain_unmapped_only,
        "threads": context.effective_threads,
        "report_json": context.paths.report,
        "raw_backend_report": context.paths.raw_backend_report,
        "raw_backend_report_format": "bowtie2_met_file",
    });
    if let Some(r2) = context.r2 {
        params["input_r2"] = serde_json::json!(r2);
        params["output_r1"] = serde_json::json!(context.paths.output_r1);
        params["output_r2"] = serde_json::json!(context.paths.output_r2);
        params["removed_host_r1"] = serde_json::json!(context.paths.removed_r1);
        params["removed_host_r2"] = serde_json::json!(context.paths.removed_r2);
    } else {
        params["output"] = serde_json::json!(context.paths.output_r1);
        params["removed_host_reads"] = serde_json::json!(context.paths.removed_r1);
    }
    params
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
