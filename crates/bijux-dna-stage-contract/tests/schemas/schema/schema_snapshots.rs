use bijux_dna_stage_contract::{
    ExecutionPlan, StageInvocationV1, StagePlanV1, StagePluginOutputV1,
};

#[test]
fn stage_contract_schema_snapshot() {
    let plan = StagePlanV1 {
        stage_id: bijux_dna_core::ids::StageId::new("fastq.trim_reads"),
        stage_instance_id: None,
        stage_version: bijux_dna_core::ids::StageVersion(1),
        tool_id: bijux_dna_core::ids::ToolId::new("fastp"),
        tool_version: "1.0".to_string(),
        image: bijux_dna_core::prelude::ContainerImageRefV1 {
            image: "fastp".to_string(),
            digest: None,
        },
        command: bijux_dna_core::prelude::CommandSpecV1 { template: vec!["fastp".to_string()] },
        resources: bijux_dna_core::contract::ToolConstraints::default(),
        io: bijux_dna_core::contract::StageIO {
            inputs: vec![bijux_dna_core::contract::ArtifactRef::required(
                bijux_dna_core::ids::ArtifactId::new("reads_in"),
                "reads.fq".into(),
                bijux_dna_core::contract::ArtifactRole::Reads,
            )],
            outputs: vec![bijux_dna_core::contract::ArtifactRef::required(
                bijux_dna_core::ids::ArtifactId::new("reads_out"),
                "reads.trimmed.fq".into(),
                bijux_dna_core::contract::ArtifactRole::TrimmedReads,
            )],
        },
        out_dir: "out/fastq.trim_reads".into(),
        params: serde_json::json!({"quality": 20}),
        effective_params: serde_json::json!({"quality": 20}),
        aux_images: std::collections::BTreeMap::default(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    };
    let execution = ExecutionPlan::new(
        "fastq-to-fastq__default__v1",
        "planner",
        bijux_dna_core::contract::PlanPolicy::default(),
        vec![plan.clone()],
        Vec::new(),
    )
    .unwrap_or_else(|err| panic!("build execution plan: {err}"));
    let invocation = StageInvocationV1 {
        command: vec!["fastp".to_string()],
        env: std::collections::BTreeMap::new(),
        expected_outputs: Vec::new(),
    };
    let output = StagePluginOutputV1 {
        metrics: bijux_dna_core::metrics::MetricsEnvelope {
            schema_version: "bijux.metrics_envelope.v2".to_string(),
            contract_version: bijux_dna_core::contract::ContractVersion::v1(),
            stage_id: "fastq.trim_reads".to_string(),
            stage_version: 1,
            tool_id: "fastp".to_string(),
            tool_version: "1.0".to_string(),
            image_digest: "fastp".to_string(),
            parameters_fingerprint: "params-hash".to_string(),
            input_fingerprint: "input-hash".to_string(),
            parameters_json_normalized: serde_json::json!({}),
            input_hashes: Vec::new(),
            metric_provenance: None,
            metrics: serde_json::json!({}),
        },
        artifacts: Vec::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        report_parts: Vec::new(),
        warnings: Vec::new(),
        invariants: Vec::new(),
        verdict: None,
        event_hints: Vec::new(),
    };

    let expected =
        include_str!("../../fixtures/stage_contract_schema/default/stage_contract_schema.json");
    let payload = serde_json::json!({
        "plan": plan,
        "execution": execution,
        "invocation": invocation,
        "output": output,
    });
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&payload)
            .unwrap_or_else(|err| panic!("canonicalize schema payload: {err}")),
    )
    .unwrap_or_else(|err| panic!("decode schema payload as utf8: {err}"));
    assert_eq!(actual, expected);
}
