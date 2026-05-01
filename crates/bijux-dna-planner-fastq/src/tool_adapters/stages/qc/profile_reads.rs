use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    stats::FastqStatsParams, stats::STATS_SCHEMA_VERSION, PairedMode,
};
use bijux_dna_domain_fastq::STAGE_PROFILE_READS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_PROFILE_READS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// Build a read profiling plan.
///
/// # Errors
/// Returns an error if planning fails.
pub fn plan_stats_neutral(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_stats_with_threads(tool, r1, r2, out_dir, None)
}

/// Build a read profiling plan with an optional thread override.
///
/// # Errors
/// Returns an error if planning fails.
pub fn plan_stats_with_threads(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    threads_override: Option<u32>,
) -> Result<StagePlanV1> {
    let threads = threads_override.unwrap_or(tool.resources.threads).max(1);
    let mut resources = tool.resources.clone();
    resources.threads = threads;
    let effective_params = FastqStatsParams {
        schema_version: STATS_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads,
    };
    let command_template = profile_reads_command(tool, r1, r2, threads)?;
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
        command: CommandSpecV1 { template: command_template },
        resources,
        io: StageIO {
            inputs,
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("qc_json"),
                    out_dir.join("qc.json"),
                    ArtifactRole::MetricsJson,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("qc_tsv"),
                    out_dir.join("qc.tsv"),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::optional(
                    ArtifactId::from_static("qc_plots_dir"),
                    out_dir.join("plots"),
                    ArtifactRole::Index,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "threads": threads,
            "out_dir": out_dir
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize profile_reads effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn profile_reads_command(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    threads: u32,
) -> Result<Vec<String>> {
    let tool_id = tool.tool_id.as_str();
    if tool_id != "seqkit_stats" {
        return Err(anyhow!("unsupported read-profiling tool for stage planning: {tool_id}"));
    }
    let rendered = crate::tool_adapters::template_render::render_command_template(
        &tool.command.template,
        &[
            ("threads", Some(threads.to_string())),
            ("reads_r1", Some(r1.display().to_string())),
            ("reads_r2", Some(r2.map(|path| path.display().to_string()).unwrap_or_default())),
        ],
    )?;
    let command = rendered.into_iter().filter(|token| !token.is_empty()).collect::<Vec<_>>();
    if command.is_empty() {
        return Err(anyhow!("profile reads command template resolved to an empty command"));
    }
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::plan_stats_with_threads;
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
        ToolVersion,
    };
    use std::path::Path;

    fn seqkit_tool() -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("seqkit_stats"),
            tool_version: ToolVersion::from("2.8.0"),
            image: ContainerImageRefV1 { image: "bijuxdna/seqkit".to_string(), digest: None },
            command: CommandSpecV1 {
                template: vec![
                    "seqkit_stats".to_string(),
                    "-a".to_string(),
                    "-T".to_string(),
                    "-j".to_string(),
                    "{{threads}}".to_string(),
                    "{{reads_r1}}".to_string(),
                    "{{reads_r2}}".to_string(),
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
    fn profile_reads_plan_renders_seqkit_thread_template() {
        let plan = plan_stats_with_threads(
            &seqkit_tool(),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            Some(8),
        )
        .expect("plan");

        assert_eq!(plan.resources.threads, 8);
        assert_eq!(
            plan.command.template,
            vec!["seqkit_stats", "-a", "-T", "-j", "8", "reads_R1.fastq.gz", "reads_R2.fastq.gz",]
        );
    }
}
