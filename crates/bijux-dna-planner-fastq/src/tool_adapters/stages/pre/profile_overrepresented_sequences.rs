use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::{
    params::stats::OVERREPRESENTED_PROFILE_SCHEMA_VERSION,
    stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES, FastqOverrepresentedProfileParams,
    PairedMode,
};
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_PROFILE_OVERREPRESENTED_SEQUENCES;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// Build an overrepresented-sequence analysis plan.
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

/// Build an overrepresented-sequence analysis plan with governed option overrides.
///
/// # Errors
/// Returns an error if plan serialization fails.
pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    threads_override: Option<u32>,
    top_k_override: Option<u32>,
) -> Result<StagePlanV1> {
    let report_tsv = out_dir.join("overrepresented_sequences.tsv");
    let summary_json = out_dir.join("overrepresented_sequences.json");
    let report_json = out_dir.join("overrepresented_report.json");
    let fastqc_dir = out_dir.join("fastqc_overrepresented");
    let threads = threads_override.unwrap_or(tool.resources.threads).max(1);
    let top_k = top_k_override.unwrap_or(50).max(1);
    let command_template =
        profile_overrepresented_command(&tool.tool_id.0, r1, r2, &fastqc_dir, threads)?;
    let effective_params = FastqOverrepresentedProfileParams {
        schema_version: OVERREPRESENTED_PROFILE_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads,
        top_k,
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
                    ArtifactId::from_static("overrepresented_sequences_tsv"),
                    report_tsv.clone(),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("overrepresented_sequences_json"),
                    summary_json.clone(),
                    ArtifactRole::MetricsJson,
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
            "input_r1": r1,
            "input_r2": r2,
            "threads": threads,
            "top_k": top_k,
            "output_tsv": report_tsv,
            "output_json": summary_json,
            "report_json": report_json,
        }),
        effective_params: serde_json::to_value(&effective_params)?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::new(
            bijux_dna_stage_contract::PlanReasonKind::Default,
            "overrepresented sequence detection",
        ),
    })
}

fn profile_overrepresented_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    fastqc_dir: &Path,
    threads: u32,
) -> Result<Vec<String>> {
    match tool_id {
        "fastqc" => {
            let mut command = vec![
                "fastqc".to_string(),
                "--outdir".to_string(),
                fastqc_dir.display().to_string(),
                "--threads".to_string(),
                threads.to_string(),
                r1.display().to_string(),
            ];
            if let Some(r2) = r2 {
                command.push(r2.display().to_string());
            }
            Ok(wrap_fastqc_command(&command, fastqc_dir))
        }
        "seqkit" => {
            let mut command = vec![
                "seqkit".to_string(),
                "fx2tab".to_string(),
                "-j".to_string(),
                threads.to_string(),
                "-n".to_string(),
                "-s".to_string(),
                r1.display().to_string(),
            ];
            if let Some(r2) = r2 {
                command.push(r2.display().to_string());
            }
            Ok(vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!("{} > /dev/null", shell_join(&command)),
            ])
        }
        "fastq_scan" => {
            let mut command = vec![tool_id.to_string(), r1.display().to_string()];
            if let Some(r2) = r2 {
                command.push(r2.display().to_string());
            }
            Ok(command)
        }
        _ => {
            Err(anyhow!("unsupported overrepresented-sequence tool for stage planning: {tool_id}"))
        }
    }
}

fn wrap_fastqc_command(command: &[String], output_dir: &Path) -> Vec<String> {
    vec![
        "sh".to_string(),
        "-lc".to_string(),
        format!("mkdir -p {}\n{}", shell_quote(output_dir), shell_join(command)),
    ]
}

fn shell_join(command: &[String]) -> String {
    command.iter().map(|part| shell_quote_str(part)).collect::<Vec<_>>().join(" ")
}

fn shell_quote(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::plan_with_options;
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
        ToolVersion,
    };
    use std::path::Path;

    fn fastqc_tool() -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("fastqc"),
            tool_version: ToolVersion::from("0.12.1"),
            image: ContainerImageRefV1 { image: "bijuxdna/fastqc".to_string(), digest: None },
            command: CommandSpecV1 {
                template: vec!["fastqc".to_string(), "{{reads_r1}}".to_string()],
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
    fn profile_overrepresented_plan_honors_thread_override() {
        let plan = plan_with_options(
            &fastqc_tool(),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            Some(6),
            Some(25),
        )
        .expect("plan");

        assert_eq!(plan.resources.threads, 6);
        assert_eq!(&plan.command.template[..2], ["sh".to_string(), "-lc".to_string()]);
        assert!(plan.command.template[2].contains("mkdir -p 'out/fastqc_overrepresented'"));
        assert!(plan.command.template[2].contains("fastqc"));
        assert_eq!(plan.params["threads"], serde_json::json!(6));
        assert_eq!(plan.params["top_k"], serde_json::json!(25));
    }

    fn seqkit_tool() -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("seqkit"),
            tool_version: ToolVersion::from("2.9.0"),
            image: ContainerImageRefV1 { image: "bijuxdna/seqkit".to_string(), digest: None },
            command: CommandSpecV1 {
                template: vec!["seqkit".to_string(), "{{reads_r1}}".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        }
    }

    #[test]
    fn profile_overrepresented_seqkit_plan_streams_sequences() {
        let plan = plan_with_options(
            &seqkit_tool(),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            Some(4),
            Some(50),
        )
        .expect("plan");

        assert_eq!(&plan.command.template[..2], ["sh".to_string(), "-lc".to_string()]);
        assert!(plan.command.template[2].contains("seqkit"));
        assert!(plan.command.template[2].contains("fx2tab"));
        assert!(plan.command.template[2].contains("'-j' '4'"));
        assert!(plan.command.template[2].contains("> /dev/null"));
    }
}
