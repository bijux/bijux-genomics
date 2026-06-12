use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::{
    params::{stats::READ_LENGTH_PROFILE_SCHEMA_VERSION, PairedMode},
    stages::ids::STAGE_PROFILE_READ_LENGTHS,
    FastqReadLengthProfileParams,
};
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_PROFILE_READ_LENGTHS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// Build a pre-trim length distribution plan.
///
/// # Errors
/// Returns an error if plan serialization fails.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_with_options(tool, r1, r2, out_dir, None, None)
}

/// Build a pre-trim length distribution plan with governed histogram options.
///
/// # Errors
/// Returns an error if plan serialization fails.
pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    threads_override: Option<u32>,
    histogram_bins_override: Option<u32>,
) -> Result<StagePlanV1> {
    let dist_tsv = out_dir.join("length_distribution.tsv");
    let dist_json = out_dir.join("length_distribution.json");
    let report_json = out_dir.join("profile_read_lengths_report.json");
    let threads = threads_override.unwrap_or(tool.resources.threads).max(1);
    let command_template = profile_lengths_command(tool, r1, r2, threads)?;
    let histogram_bins = histogram_bins_override.unwrap_or(100).max(1);
    let effective_params = FastqReadLengthProfileParams {
        schema_version: READ_LENGTH_PROFILE_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads,
        histogram_bins,
    };
    let mut resources = tool.resources.clone();
    resources.threads = threads;
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
                    ArtifactId::from_static("report_json"),
                    report_json.clone(),
                    ArtifactRole::ReportJson,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("length_distribution_tsv"),
                    dist_tsv.clone(),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("length_distribution_json"),
                    dist_json.clone(),
                    ArtifactRole::MetricsJson,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "threads": threads,
            "histogram_bins": histogram_bins,
            "report_json": report_json,
            "output_tsv": dist_tsv,
            "output_json": dist_json,
        }),
        effective_params: serde_json::to_value(&effective_params)?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::new(
            bijux_dna_stage_contract::PlanReasonKind::Default,
            "pre-trim length distribution metrics",
        ),
    })
}

fn profile_lengths_command(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    threads: u32,
) -> Result<Vec<String>> {
    let tool_id = tool.tool_id.as_str();
    let command = match tool_id {
        "seqkit_stats" => {
            let rendered = crate::tool_adapters::template_render::render_command_template(
                &tool.command.template,
                &[
                    ("threads", Some(threads.to_string())),
                    ("reads_r1", Some(r1.display().to_string())),
                    (
                        "reads_r2",
                        Some(r2.map(|path| path.display().to_string()).unwrap_or_default()),
                    ),
                ],
            )?;
            rendered.into_iter().filter(|token| !token.is_empty()).collect::<Vec<_>>()
        }
        "seqfu" => {
            let mut command = vec![
                "seqfu".to_string(),
                "stats".to_string(),
                "-a".to_string(),
                "-T".to_string(),
                "-j".to_string(),
                threads.to_string(),
                r1.display().to_string(),
            ];
            if let Some(r2) = r2 {
                command.push(r2.display().to_string());
            }
            command
        }
        "fastp" => fastp_profile_lengths_command(r1, r2, threads),
        "prinseq" => prinseq_profile_lengths_command(r1, r2, threads),
        _ => {
            return Err(anyhow!(
                "unsupported read-length profiling tool for stage planning: {tool_id}"
            ));
        }
    };
    if command.is_empty() {
        return Err(anyhow!("profile read lengths command template resolved to an empty command"));
    }
    Ok(command)
}

fn fastp_profile_lengths_command(r1: &Path, r2: Option<&Path>, threads: u32) -> Vec<String> {
    let mut command = vec![
        "fastp".to_string(),
        "--in1".to_string(),
        r1.display().to_string(),
        "--out1".to_string(),
        "/dev/null".to_string(),
        "--thread".to_string(),
        threads.to_string(),
        "--json".to_string(),
        "/dev/null".to_string(),
        "--disable_adapter_trimming".to_string(),
        "--disable_quality_filtering".to_string(),
        "--disable_length_filtering".to_string(),
        "--disable_trim_poly_g".to_string(),
        "--dont_eval_duplication".to_string(),
    ];
    if let Some(r2) = r2 {
        command.extend([
            "--in2".to_string(),
            r2.display().to_string(),
            "--out2".to_string(),
            "/dev/null".to_string(),
        ]);
    }
    command
}

fn prinseq_profile_lengths_command(r1: &Path, r2: Option<&Path>, threads: u32) -> Vec<String> {
    let mut command = vec![
        "prinseq++".to_string(),
        "-threads".to_string(),
        threads.to_string(),
        "-fastq".to_string(),
        r1.display().to_string(),
        "-out_good".to_string(),
        "/dev/null".to_string(),
        "-out_bad".to_string(),
        "/dev/null".to_string(),
    ];
    if let Some(r2) = r2 {
        command.extend([
            "-fastq2".to_string(),
            r2.display().to_string(),
            "-out_good2".to_string(),
            "/dev/null".to_string(),
            "-out_bad2".to_string(),
            "/dev/null".to_string(),
            "-out_single".to_string(),
            "/dev/null".to_string(),
            "-out_single2".to_string(),
            "/dev/null".to_string(),
        ]);
    }
    command
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::plan_with_options;
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

    fn stats_tool(tool_id: &'static str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::from_static(tool_id),
            tool_version: ToolVersion::from("2.8.0"),
            image: ContainerImageRefV1 { image: format!("bijuxdna/{tool_id}"), digest: None },
            command: CommandSpecV1 { template: vec![tool_id.to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 2,
            },
        }
    }

    #[test]
    fn profile_read_lengths_plan_renders_seqkit_thread_template() {
        let plan = plan_with_options(
            &seqkit_tool(),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            Some(6),
            Some(64),
        )
        .expect("plan");

        assert_eq!(plan.resources.threads, 6);
        assert_eq!(
            plan.command.template,
            vec!["seqkit_stats", "-a", "-T", "-j", "6", "reads_R1.fastq.gz", "reads_R2.fastq.gz",]
        );
    }

    #[test]
    fn profile_read_lengths_plan_renders_seqfu_stats_command() {
        let plan = plan_with_options(
            &stats_tool("seqfu"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            Some(5),
            Some(64),
        )
        .expect("plan");

        assert_eq!(plan.resources.threads, 5);
        assert_eq!(
            plan.command.template,
            vec!["seqfu", "stats", "-a", "-T", "-j", "5", "reads_R1.fastq.gz", "reads_R2.fastq.gz",]
        );
    }

    #[test]
    fn profile_read_lengths_plan_renders_fastp_report_only_command() {
        let plan = plan_with_options(
            &stats_tool("fastp"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            Some(4),
            Some(64),
        )
        .expect("plan");

        assert_eq!(plan.resources.threads, 4);
        assert_eq!(
            plan.command.template,
            vec![
                "fastp",
                "--in1",
                "reads_R1.fastq.gz",
                "--out1",
                "/dev/null",
                "--thread",
                "4",
                "--json",
                "/dev/null",
                "--disable_adapter_trimming",
                "--disable_quality_filtering",
                "--disable_length_filtering",
                "--disable_trim_poly_g",
                "--dont_eval_duplication",
                "--in2",
                "reads_R2.fastq.gz",
                "--out2",
                "/dev/null",
            ]
        );
    }

    #[test]
    fn profile_read_lengths_plan_renders_prinseq_null_output_command() {
        let plan = plan_with_options(
            &stats_tool("prinseq"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            Some(3),
            Some(64),
        )
        .expect("plan");

        assert_eq!(plan.resources.threads, 3);
        assert_eq!(
            plan.command.template,
            vec![
                "prinseq++",
                "-threads",
                "3",
                "-fastq",
                "reads_R1.fastq.gz",
                "-out_good",
                "/dev/null",
                "-out_bad",
                "/dev/null",
                "-fastq2",
                "reads_R2.fastq.gz",
                "-out_good2",
                "/dev/null",
                "-out_bad2",
                "/dev/null",
                "-out_single",
                "/dev/null",
                "-out_single2",
                "/dev/null",
            ]
        );
    }
}
