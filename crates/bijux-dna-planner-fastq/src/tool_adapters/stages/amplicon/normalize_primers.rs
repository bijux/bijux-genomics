use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_PRIMERS;
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_NORMALIZE_PRIMERS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let output_r1 = if r2.is_some() {
        out_dir.join("R1.primer_normalized.fastq.gz")
    } else {
        out_dir.join("primer_normalized.fastq.gz")
    };
    let output_r2 = r2.map(|_| out_dir.join("R2.primer_normalized.fastq.gz"));
    let orientation_report = out_dir.join("primer_orientation.tsv");
    let primer_stats = out_dir.join("primer_stats.json");
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
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("normalized_reads_r1"),
        output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("normalized_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("primer_orientation_report"),
        orientation_report.clone(),
        ArtifactRole::SummaryTsv,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("primer_stats_json"),
        primer_stats.clone(),
        ArtifactRole::MetricsJson,
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
            template: normalize_primers_command(
                &tool.tool_id.0,
                r1,
                r2,
                &output_r1,
                output_r2.as_deref(),
                &orientation_report,
                &primer_stats,
            )?,
        },
        resources: tool.resources.clone(),
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({
            "orientation_policy": "normalize_to_forward_primer",
            "primer_set_id": "default",
            "paired_mode": if r2.is_some() { "paired_end" } else { "single_end" },
            "mismatch_policy": {
                "max_mismatches": 2,
                "allow_iupac_codes": true,
                "strict_5p_anchor": true
            }
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "amplicon primer normalization"),
    })
}

fn normalize_primers_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    orientation_report: &Path,
    primer_stats: &Path,
) -> Result<Vec<String>> {
    match tool_id {
        "cutadapt" => {
            let mut command = vec![
                "cutadapt".to_string(),
                "-g".to_string(),
                "file:primers.fa".to_string(),
                "--overlap".to_string(),
                "8".to_string(),
                "--error-rate".to_string(),
                "0.12".to_string(),
                "--revcomp".to_string(),
                "--info-file".to_string(),
                orientation_report.display().to_string(),
                "--json".to_string(),
                primer_stats.display().to_string(),
                "-o".to_string(),
                output_r1.display().to_string(),
            ];
            if let Some(output_r2) = output_r2 {
                command.push("-p".to_string());
                command.push(output_r2.display().to_string());
            }
            command.push(r1.display().to_string());
            if let Some(r2) = r2 {
                command.push(r2.display().to_string());
            }
            Ok(command)
        }
        "seqkit" => {
            if r2.is_some() {
                return Err(anyhow!(
                    "seqkit primer normalization planning requires a single merged or single-end input"
                ));
            }
            Ok(vec![
                "seqkit".to_string(),
                "grep".to_string(),
                "-r".to_string(),
                "-p".to_string(),
                "PRIMER".to_string(),
                "-o".to_string(),
                output_r1.display().to_string(),
                r1.display().to_string(),
            ])
        }
        _ => Err(anyhow!(
            "unsupported primer normalization tool for stage planning: {tool_id}"
        )),
    }
}
