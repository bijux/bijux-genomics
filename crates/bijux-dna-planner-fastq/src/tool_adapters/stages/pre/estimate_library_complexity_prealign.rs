use std::path::Path;

use anyhow::Result;
use bijux_dna_core::contract::StageOperatingMode;
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// # Errors
/// Returns an error if the governed report parameters cannot be serialized.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    kmer_size: Option<u32>,
) -> Result<StagePlanV1> {
    let library_complexity_report = out_dir.join("library_complexity_report.json");

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
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs,
            outputs: vec![ArtifactRef::required(
                ArtifactId::from_static("library_complexity_report"),
                library_complexity_report.clone(),
                ArtifactRole::ReportJson,
            )],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.as_str(),
            "input_r1": r1,
            "input_r2": r2,
            "out_dir": out_dir,
            "library_complexity_report": library_complexity_report,
            "complexity_policy": "prealign_kmer",
            "estimate_method": "kmer_redundancy",
            "kmer_size": kmer_size,
        }),
        effective_params: serde_json::json!({
            "paired_mode": if r2.is_some() { "paired_end" } else { "single_end" },
            "complexity_policy": "prealign_kmer",
            "estimate_method": "kmer_redundancy",
            "modifies_reads": false,
            "advisory_only": true,
            "kmer_size": kmer_size,
        }),
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: StageOperatingMode::Advisory,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::plan;
    use anyhow::Result;
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, StageOperatingMode, ToolConstraints,
        ToolExecutionSpecV1, ToolId,
    };

    fn tool() -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("bijux_dna"),
            tool_version: "workspace".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux-dna".to_string(),
                digest: None,
            },
            command: CommandSpecV1 { template: vec!["bijux-dna".to_string()] },
            resources: ToolConstraints {
                runtime: "local".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        }
    }

    #[test]
    fn estimate_library_complexity_plan_emits_report_only_contract() -> Result<()> {
        let temp = std::env::temp_dir().join("bijux-estimate-library-complexity-plan");
        bijux_dna_infra::ensure_dir(&temp)?;
        let r1 = temp.join("reads_R1.fastq");
        let r2 = temp.join("reads_R2.fastq");
        let out_dir = temp.join("out");

        let plan = plan(&tool(), &r1, Some(&r2), &out_dir, Some(4))?;

        assert_eq!(plan.stage_id.as_str(), "fastq.estimate_library_complexity_prealign");
        assert_eq!(plan.tool_id.as_str(), "bijux_dna");
        assert_eq!(plan.io.outputs.len(), 1);
        assert_eq!(plan.io.outputs[0].name.as_str(), "library_complexity_report");
        assert_eq!(plan.operating_mode, StageOperatingMode::Advisory);
        assert_eq!(plan.effective_params["advisory_only"], serde_json::json!(true));
        assert_eq!(plan.effective_params["kmer_size"], serde_json::json!(4));
        Ok(())
    }
}
