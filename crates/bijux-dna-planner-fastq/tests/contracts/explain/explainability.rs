use std::collections::BTreeMap;

use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_planner_fastq::compose_fastq_pipeline_steps;
use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind};

#[test]
fn tool_reasons_carry_defaults_and_contract_hash() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-reasons")?;
    let r1 = temp.path().join("r1.fastq");
    bijux_dna_infra::write_bytes(&r1, "@r1\nA\n+\n#\n")?;

    let tool = ToolExecutionSpecV1 {
        tool_id: ToolId::new("fastqvalidator_official"),
        tool_version: "1.0".to_string(),
        image: ContainerImageRefV1 {
            image: "fastqvalidator".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["fastqvalidator".to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    };

    let stage = bijux_dna_domain_fastq::STAGE_VALIDATE_PRE
        .as_str()
        .to_string();
    let contract_hash = bijux_dna_domain_fastq::stage_contract_hash(&stage)
        .and_then(|hash| hash.ok())
        .unwrap_or_else(|| "missing".to_string());
    let reason = PlanDecisionReason {
        kind: PlanReasonKind::Default,
        summary: "default selection".to_string(),
        details: serde_json::json!({
            "defaults_diff": {},
            "contract_hash": contract_hash,
        }),
    };

    let plans = compose_fastq_pipeline_steps(
        std::slice::from_ref(&stage),
        &[tool],
        &BTreeMap::new(),
        Some(std::slice::from_ref(&reason)),
        None,
        None,
        None,
        false,
        &r1,
        None,
        |stage_id, tool, _r1, _r2| Ok(temp.path().join(stage_id).join(tool.tool_id.as_str())),
    )?;

    let plan_reason = &plans[0].reason;
    assert_eq!(plan_reason.kind, reason.kind);
    assert_eq!(plan_reason.summary, reason.summary);
    assert!(plan_reason
        .details
        .get("defaults_diff")
        .is_some_and(|value| value.is_object()));
    assert!(plan_reason
        .details
        .get("contract_hash")
        .is_some_and(|value| value.as_str().is_some()));
    Ok(())
}
