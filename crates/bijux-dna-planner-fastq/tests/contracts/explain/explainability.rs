//! Snapshot contract for deterministic FASTQ stage decision reasons.

use std::collections::BTreeMap;

use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_planner_fastq::{compose_fastq_stage_bindings, FastqStageBinding};
use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind};

#[test]
/// Ensures FASTQ default stage reasons include defaults diff and contract hash fields.
fn tool_reasons_carry_defaults_and_contract_hash() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-reasons")?;
    let r1 = temp.path().join("r1.fastq");
    bijux_dna_infra::write_bytes(&r1, "@r1\nA\n+\n#\n")?;

    let tool = ToolExecutionSpecV1 {
        tool_id: ToolId::new("fastqvalidator"),
        tool_version: "1.0".to_string(),
        image: ContainerImageRefV1 { image: "fastqvalidator".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["fastqvalidator".to_string()] },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    };

    let stage = bijux_dna_domain_fastq::STAGE_VALIDATE_READS.as_str().to_string();
    let contract_hash = bijux_dna_domain_fastq::stage_contract_hash(&stage)
        .and_then(Result::ok)
        .unwrap_or_else(|| "missing".to_string());
    let reason = PlanDecisionReason {
        kind: PlanReasonKind::Default,
        summary: "default selection".to_string(),
        details: serde_json::json!({
            "defaults_diff": {},
            "contract_hash": contract_hash,
        }),
    };

    let binding = FastqStageBinding {
        stage_id: stage.clone(),
        stage_instance_id: None,
        tool,
        reason: Some(reason.clone()),
        params: None,
    };
    let plans = compose_fastq_stage_bindings(
        std::slice::from_ref(&binding),
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        &r1,
        None,
        None,
        None,
        |binding, _r1, _r2| {
            Ok(temp.path().join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )?;

    let plan_reason = &plans[0].reason;
    assert_eq!(plan_reason.kind, reason.kind);
    assert_eq!(plan_reason.summary, reason.summary);
    assert!(plan_reason.details.get("defaults_diff").is_some_and(serde_json::Value::is_object));
    assert!(plan_reason.details.get("contract_hash").is_some_and(|value| value.as_str().is_some()));
    Ok(())
}

#[test]
/// Captures stable reason payloads for FASTQ trim/stats/screen planner decisions.
fn stage_reasons_are_deterministic_for_new_fastq_stage_set() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-reasons-deterministic")?;
    let r1 = temp.path().join("r1.fastq");
    bijux_dna_infra::write_bytes(&r1, "@r1\nA\n+\n#\n")?;

    let tool_for = |tool: &str| ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "1.0".to_string(),
        image: ContainerImageRefV1 { image: tool.to_string(), digest: None },
        command: CommandSpecV1 { template: vec![tool.to_string()] },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    };

    let stages = [
        (bijux_dna_domain_fastq::STAGE_TRIM_READS.as_str().to_string(), tool_for("fastp")),
        (
            bijux_dna_domain_fastq::STAGE_PROFILE_READS.as_str().to_string(),
            tool_for("seqkit_stats"),
        ),
        (bijux_dna_domain_fastq::STAGE_SCREEN_TAXONOMY.as_str().to_string(), tool_for("kraken2")),
    ];
    let bindings: Vec<FastqStageBinding> = stages
        .iter()
        .map(|(stage_id, tool)| FastqStageBinding {
            stage_id: stage_id.clone(),
            stage_instance_id: None,
            tool: tool.clone(),
            reason: Some(PlanDecisionReason {
                kind: PlanReasonKind::Default,
                summary: format!("default selection for {stage_id}"),
                details: serde_json::json!({
                    "defaults_diff": {},
                    "contract_hash": bijux_dna_domain_fastq::stage_contract_hash(stage_id)
                        .and_then(Result::ok)
                        .unwrap_or_else(|| "missing".to_string()),
                }),
            }),
            params: None,
        })
        .collect();

    let mk_plan = || {
        compose_fastq_stage_bindings(
            &bindings,
            &BTreeMap::new(),
            None,
            None,
            None,
            false,
            &r1,
            None,
            None,
            None,
            |binding, _r1, _r2| {
                Ok(temp
                    .path()
                    .join(binding.stage_id.replace('.', "_"))
                    .join(binding.tool.tool_id.as_str()))
            },
        )
    };

    let first = mk_plan()?;
    let second = mk_plan()?;
    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.reason.summary, b.reason.summary);
        assert_eq!(a.reason.kind, b.reason.kind);
        assert!(a.reason.details.get("defaults_diff").is_some());
        assert!(a.reason.details.get("contract_hash").is_some());
        let reason_json =
            bijux_dna_testkit::snapshot_normalize_json(&serde_json::to_value(&a.reason)?);
        let snapshot_name = format!("bijux-dna-planner-fastq__contracts__explain__{}", a.stage_id);
        insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
            insta::assert_json_snapshot!(snapshot_name, reason_json);
        });
    }
    Ok(())
}
