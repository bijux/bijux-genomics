//! Snapshot contract for deterministic BAM stage decision reasons.

use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_planner_bam::{pipeline_id_catalog, plan_stage, StagePlanRequest};

fn dummy_tool(tool_id: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id),
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
/// Ensures BAM stage explain artifacts stay deterministic and operator-readable.
fn bam_plan_reasons_include_defaults_and_contract_hash() -> anyhow::Result<()> {
    let stages = pipeline_id_catalog("bam-to-bam__default__v1");
    let temp = bijux_dna_infra::temp_dir("bam-plan-reasons")?;
    let bam = temp.path().join("sample.bam");
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&bam, b"")?;
    std::fs::write(&reference, b">chrM\nACGT\n")?;

    for stage_id in stages {
        let stage = BamStage::try_from(stage_id.as_str())?;
        let tool_id = bijux_dna_planner_bam::stage_api::default_tool_for_stage(stage);
        let plan = plan_stage(StagePlanRequest {
            stage_id: &stage_id,
            tool: &dummy_tool(tool_id.as_str()),
            out_dir: temp.path(),
            bam: Some(&bam),
            bam_index: None,
            r1: None,
            r2: None,
            reference: Some(&reference),
            sample_id: Some("sample"),
            params: None,
        })?;
        assert!(!plan.reason.summary.trim().is_empty());
        assert!(plan.reason.details.get("defaults_diff").is_some_and(|value| value.is_object()));
        assert!(plan
            .reason
            .details
            .get("contract_hash")
            .is_some_and(|value| value.as_str().is_some()));
    }
    Ok(())
}

#[test]
/// Captures stable reason payloads for BAM aDNA-specific selection decisions.
fn bam_adna_plan_reasons_are_deterministic_for_new_stages() -> anyhow::Result<()> {
    let stages = pipeline_id_catalog("bam-to-bam__adna_shotgun__v1");
    let target: Vec<String> = stages
        .into_iter()
        .filter(|stage| {
            matches!(stage.as_str(), "bam.damage" | "bam.authenticity" | "bam.contamination")
        })
        .collect();
    let temp = bijux_dna_infra::temp_dir("bam-plan-reasons-adna-deterministic")?;
    let bam = temp.path().join("sample.bam");
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&bam, b"")?;
    std::fs::write(&reference, b">chr1\nA\n")?;

    let run_once = || -> anyhow::Result<Vec<bijux_dna_stage_contract::StagePlanV1>> {
        let mut out = Vec::new();
        for stage_id in &target {
            let tool_id = match stage_id.as_str() {
                "bam.damage" => "pydamage",
                "bam.authenticity" => "pmdtools",
                "bam.contamination" => "verifybamid2",
                _ => "samtools",
            };
            let params = if stage_id == "bam.contamination" {
                Some(serde_json::json!({
                    "reference_panels": ["1kg-phase3"],
                    "scope": "nuclear",
                    "prior": 0.02,
                    "sex_specific": false,
                    "assumptions": "panel-based estimate",
                    "chromosome_system": "xy",
                    "minimum_mean_coverage": 0.75,
                    "emit_confidence_caveats": true
                }))
            } else {
                None
            };
            let plan = plan_stage(StagePlanRequest {
                stage_id,
                tool: &dummy_tool(tool_id),
                out_dir: temp.path(),
                bam: Some(&bam),
                bam_index: None,
                r1: None,
                r2: None,
                reference: Some(&reference),
                sample_id: Some("sample"),
                params: params.as_ref(),
            })?;
            out.push(plan);
        }
        Ok(out)
    };

    let first = run_once()?;
    let second = run_once()?;
    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.reason.summary, b.reason.summary);
        assert_eq!(a.reason.kind, b.reason.kind);
        assert!(a.reason.details.get("defaults_diff").is_some());
        assert!(a.reason.details.get("contract_hash").is_some());
        let reason_json =
            bijux_dna_testkit::snapshot_normalize_json(&serde_json::to_value(&a.reason)?);
        let snapshot_name = format!("bijux-dna-planner-bam__contracts__explain__{}", a.stage_id);
        insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
            insta::assert_json_snapshot!(snapshot_name, reason_json);
        });
    }
    Ok(())
}
