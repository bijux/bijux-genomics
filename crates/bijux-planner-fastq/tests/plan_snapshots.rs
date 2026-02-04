use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_core::execution_plan::PlanPolicy;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_planner_fastq::{FastqPlanConfig, FastqPlanner};

#[test]
fn fastq_plan_snapshot() {
    let tool_trim = ToolExecutionSpecV1 {
        tool_id: ToolId("fastp".to_string()),
        tool_version: "0.23.4".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/fastp".to_string(),
            digest: Some("sha256:fastp".to_string()),
        },
        command: CommandSpecV1 {
            template: vec!["fastp".to_string()],
        },
        resources: ToolConstraints {
            runtime: "short".to_string(),
            mem_gb: 2,
            tmp_gb: 1,
            threads: 2,
        },
    };
    let tool_validate = ToolExecutionSpecV1 {
        tool_id: ToolId("fastqvalidator_official".to_string()),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/fastqvalidator".to_string(),
            digest: Some("sha256:fastqvalidator".to_string()),
        },
        command: CommandSpecV1 {
            template: vec!["fastqvalidator".to_string()],
        },
        resources: ToolConstraints {
            runtime: "short".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    };
    let config = FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__default__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        stages: vec!["fastq.validate_pre".to_string(), "fastq.trim".to_string()],
        tools: vec![tool_validate, tool_trim],
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: PathBuf::from("reads_R1.fastq.gz"),
        r2: None,
        out_dir: PathBuf::from("out"),
    };
    let plan = FastqPlanner::plan(&config).expect("plan");
    insta::assert_json_snapshot!(plan);
}
