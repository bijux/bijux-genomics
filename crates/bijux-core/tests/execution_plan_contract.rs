use bijux_core::contract::{ArtifactRef, StageIO, ToolConstraints};
use bijux_core::plan::execution_graph::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_core::plan::PlanPolicy;
use bijux_core::{CommandSpecV1, ContainerImageRefV1, StageId};

#[test]
#[allow(clippy::too_many_lines)]
fn execution_plan_roundtrip_is_canonical() -> anyhow::Result<()> {
    let plan = ExecutionGraph::new(
        "fastq-to-bam__default__v1",
        "planner-fastq@1",
        PlanPolicy::PreferAccuracy,
        vec![ExecutionStep {
            step_id: StageId::from_static("fastq.trim"),
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
            aux_images: std::collections::BTreeMap::new(),
            expected_artifact_ids: Vec::new(),
            metrics_schema_ids: Vec::new(),
        }],
        vec![ExecutionEdge::new(
            StageId::from_static("fastq.trim"),
            StageId::from_static("fastq.trim"),
        )],
    );
    assert!(plan.is_err(), "self-loop should be rejected");

    let plan = ExecutionGraph::new(
        "fastq-to-bam__default__v1",
        "planner-fastq@1",
        PlanPolicy::PreferAccuracy,
        vec![
            ExecutionStep {
                step_id: StageId::from_static("fastq.filter"),
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
                aux_images: std::collections::BTreeMap::new(),
                expected_artifact_ids: Vec::new(),
                metrics_schema_ids: Vec::new(),
            },
            ExecutionStep {
                step_id: StageId::from_static("fastq.trim"),
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
                aux_images: std::collections::BTreeMap::new(),
                expected_artifact_ids: Vec::new(),
                metrics_schema_ids: Vec::new(),
            },
        ],
        vec![ExecutionEdge::new(
            StageId::from_static("fastq.trim"),
            StageId::from_static("fastq.filter"),
        )],
    )?;

    let encoded = serde_json::to_string_pretty(&plan)?;
    let decoded: ExecutionGraph = serde_json::from_str(&encoded)?;
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
