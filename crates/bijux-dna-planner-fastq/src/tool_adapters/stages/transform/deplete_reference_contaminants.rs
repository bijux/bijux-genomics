use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    screen::{ReferenceContaminantEffectiveParams, REFERENCE_DEPLETION_SCHEMA_VERSION},
    PairedMode,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_REFERENCE_CONTAMINANTS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_DEPLETE_REFERENCE_CONTAMINANTS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub type DepleteReferenceContaminantsPlanOptions = crate::DepleteReferenceContaminantsStageParams;

/// # Errors
/// Returns an error if any requested contaminant-depletion tool is not admitted for
/// `fastq.deplete_reference_contaminants`.
pub fn normalize_contaminant_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

/// Build a contaminant screen plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_contaminant_screen(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    reference_index: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_contaminant_screen_with_index_backend(
        tool,
        r1,
        r2,
        reference_index,
        out_dir,
        &DepleteReferenceContaminantsPlanOptions::baseline(),
        "bowtie2_build",
    )
}

/// Build a contaminant screen plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_contaminant_screen_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    reference_index: &Path,
    out_dir: &Path,
    options: &DepleteReferenceContaminantsPlanOptions,
) -> Result<StagePlanV1> {
    plan_contaminant_screen_with_index_backend(
        tool,
        r1,
        r2,
        reference_index,
        out_dir,
        options,
        "bowtie2_build",
    )
}

/// Build a contaminant screen plan with explicit upstream reference-index provenance.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_contaminant_screen_with_index_backend(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    reference_index: &Path,
    out_dir: &Path,
    options: &DepleteReferenceContaminantsPlanOptions,
    reference_index_backend: &str,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_contaminant_screen_tool_list(std::slice::from_ref(&tool_id))?;
    let output_r1 = if r2.is_some() {
        out_dir.join("contaminant_screened_R1.fastq.gz")
    } else {
        out_dir.join("contaminant_screened.fastq.gz")
    };
    let output_r2 = r2.map(|_| out_dir.join("contaminant_screened_R2.fastq.gz"));
    let report = out_dir.join("contaminant_screen_report.json");
    let raw_backend_report = out_dir.join("bowtie2.contaminant.metrics.txt");
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let effective_params = ReferenceContaminantEffectiveParams {
        schema_version: REFERENCE_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: effective_threads,
        reference_catalog_id: "contaminant_reference".to_string(),
        contaminant_reference: options.decoy_mode.clone(),
        index_artifact: "reference_index".to_string(),
        reference_index_backend: reference_index_backend.to_string(),
        reference_build_id: None,
        reference_digest: None,
        retain_unmapped_pairs: r2.is_some(),
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
    if let Some(r2) = r2 {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("contaminant_screened_reads_r1"),
        output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("contaminant_screened_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("contaminant_screen_report_json"),
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
            template: contaminant_screen_command(
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
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "reference_index": reference_index,
            "reference_index_backend": reference_index_backend,
            "decoy_mode": options.decoy_mode,
            "threads": effective_threads,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report,
            "raw_backend_report": raw_backend_report,
            "raw_backend_report_format": "bowtie2_met_file",
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize contaminant screen effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn contaminant_screen_command(
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
                    out_dir.join("contaminant_screened_R%.fastq.gz").display().to_string(),
                ]);
            } else {
                command.extend([
                    "-U".to_string(),
                    r1.display().to_string(),
                    "--un-gz".to_string(),
                    out_dir.join("contaminant_screened.fastq.gz").display().to_string(),
                ]);
            }
            command.extend(["--met-file".to_string(), raw_backend_report.display().to_string()]);
            Ok(command)
        }
        _ => Err(anyhow!("unsupported contaminant depletion tool for stage planning: {tool_id}")),
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
    fn contaminant_screen_tracks_explicit_reference_backend() -> Result<()> {
        let plan = plan_contaminant_screen_with_index_backend(
            &tool("bowtie2"),
            Path::new("reads_R1.fastq.gz"),
            None,
            Path::new("reference.index"),
            Path::new("out"),
            &DepleteReferenceContaminantsPlanOptions::baseline(),
            "star",
        )?;

        assert_eq!(plan.effective_params["reference_index_backend"], "star");
        assert_eq!(plan.params["reference_index_backend"], "star");
        Ok(())
    }

    #[test]
    fn contaminant_screen_uses_governed_report_and_raw_backend_metrics_paths() -> Result<()> {
        let plan = plan_contaminant_screen_with_options(
            &tool("bowtie2"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("reference.index"),
            Path::new("out"),
            &DepleteReferenceContaminantsPlanOptions {
                decoy_mode: "phix_only".to_string(),
                threads: Some(6),
            },
        )?;

        assert_eq!(plan.params["report_json"], "out/contaminant_screen_report.json");
        assert_eq!(plan.params["raw_backend_report"], "out/bowtie2.contaminant.metrics.txt");
        assert_eq!(plan.effective_params["threads"], 6);
        assert!(plan
            .command
            .template
            .iter()
            .any(|part| part == "out/bowtie2.contaminant.metrics.txt"));
        Ok(())
    }
}
