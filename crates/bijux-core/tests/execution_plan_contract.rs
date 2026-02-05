use bijux_core::plan::execution_plan::{ExecutionPlan, PlanEdge, PlanPolicy};
use bijux_core::{
    ArtifactRef, CommandSpecV1, ContainerImageRefV1, StageIO, StageId, StagePlanV1, StageVersion,
    ToolConstraints, ToolId,
};

#[test]
#[allow(clippy::too_many_lines)]
fn execution_plan_roundtrip_is_canonical() -> anyhow::Result<()> {
    let plan = ExecutionPlan::new(
        "fastq-to-bam__default__v1",
        "planner-fastq@1",
        PlanPolicy::PreferAccuracy,
        vec![StagePlanV1 {
            stage_id: StageId::from_static("fastq.trim"),
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastp"),
            tool_version: "0.23.4".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/fastp".to_string(),
                digest: Some("sha256:abc".to_string()),
            },
            command: CommandSpecV1 {
                template: vec!["fastp".to_string(), "--in1".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 4,
                tmp_gb: 4,
                threads: 2,
            },
            io: StageIO {
                inputs: vec![ArtifactRef {
                    name: "r1".to_string(),
                    path: "/data/input.fastq.gz".into(),
                }],
                outputs: vec![ArtifactRef {
                    name: "trimmed".to_string(),
                    path: "/data/trimmed.fastq.gz".into(),
                }],
            },
            out_dir: "/tmp/out".into(),
            params: serde_json::json!({"sample_id": "sample-1"}),
            effective_params: serde_json::json!({"sample_id": "sample-1"}),
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
        }],
        vec![PlanEdge::new("fastq.trim", "fastq.trim")],
    );
    assert!(plan.is_err(), "self-loop should be rejected");

    let plan = ExecutionPlan::new(
        "fastq-to-bam__default__v1",
        "planner-fastq@1",
        PlanPolicy::PreferAccuracy,
        vec![
            StagePlanV1 {
                stage_id: StageId::from_static("fastq.filter"),
                stage_version: StageVersion(1),
                tool_id: ToolId::from_static("fastp"),
                tool_version: "0.23.4".to_string(),
                image: ContainerImageRefV1 {
                    image: "bijux/fastp".to_string(),
                    digest: Some("sha256:abc".to_string()),
                },
                command: CommandSpecV1 {
                    template: vec!["fastp".to_string(), "--in1".to_string()],
                },
                resources: ToolConstraints {
                    runtime: "docker".to_string(),
                    mem_gb: 4,
                    tmp_gb: 4,
                    threads: 2,
                },
                io: StageIO {
                    inputs: vec![ArtifactRef {
                        name: "r1".to_string(),
                        path: "/data/input.fastq.gz".into(),
                    }],
                    outputs: vec![ArtifactRef {
                        name: "filtered".to_string(),
                        path: "/data/filtered.fastq.gz".into(),
                    }],
                },
                out_dir: "/tmp/out".into(),
                params: serde_json::json!({"sample_id": "sample-1"}),
                effective_params: serde_json::json!({"sample_id": "sample-1"}),
                aux_images: std::collections::BTreeMap::new(),
                reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
            },
            StagePlanV1 {
                stage_id: StageId::from_static("fastq.trim"),
                stage_version: StageVersion(1),
                tool_id: ToolId::from_static("fastp"),
                tool_version: "0.23.4".to_string(),
                image: ContainerImageRefV1 {
                    image: "bijux/fastp".to_string(),
                    digest: Some("sha256:def".to_string()),
                },
                command: CommandSpecV1 {
                    template: vec!["fastp".to_string(), "--in1".to_string()],
                },
                resources: ToolConstraints {
                    runtime: "docker".to_string(),
                    mem_gb: 4,
                    tmp_gb: 4,
                    threads: 2,
                },
                io: StageIO {
                    inputs: vec![ArtifactRef {
                        name: "r1".to_string(),
                        path: "/data/input.fastq.gz".into(),
                    }],
                    outputs: vec![ArtifactRef {
                        name: "trimmed".to_string(),
                        path: "/data/trimmed.fastq.gz".into(),
                    }],
                },
                out_dir: "/tmp/out".into(),
                params: serde_json::json!({"sample_id": "sample-1"}),
                effective_params: serde_json::json!({"sample_id": "sample-1"}),
                aux_images: std::collections::BTreeMap::new(),
                reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
            },
        ],
        vec![PlanEdge::new("fastq.trim", "fastq.filter")],
    )?;

    let encoded = serde_json::to_string_pretty(&plan)?;
    let decoded: ExecutionPlan = serde_json::from_str(&encoded)?;
    let reencoded = serde_json::to_string_pretty(&decoded)?;
    assert_eq!(
        encoded, reencoded,
        "execution plan roundtrip must be canonical"
    );
    let hash_before = plan.plan_hash()?;
    let hash_after = decoded.plan_hash()?;
    assert_eq!(hash_before, hash_after, "plan_hash must be stable");
    Ok(())
}
