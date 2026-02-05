use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_core::plan::execution_plan::PlanPolicy;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_planner_fastq::{
    apply_tool_overrides, plan_fastq_to_fastq__default__v1, DefaultPipelineOptions,
    FastqPipelineInputs, FastqPlanConfig, FastqPlanner,
};

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
        tool_reasons: None,
    };
    let plan = FastqPlanner::plan(&config).expect("plan");
    insta::assert_json_snapshot!(plan);
}

#[test]
fn tool_override_precedence_is_stable() {
    let mut base = BTreeMap::new();
    base.insert("fastq.trim".to_string(), "fastp".to_string());
    let mut profile = BTreeMap::new();
    profile.insert("fastq.trim".to_string(), "cutadapt".to_string());
    let mut cli = BTreeMap::new();
    cli.insert("fastq.trim".to_string(), "bbduk".to_string());
    let mut forced = BTreeMap::new();
    forced.insert("fastq.trim".to_string(), "trimmomatic".to_string());
    let merged = apply_tool_overrides(base, profile, cli, forced);
    assert_eq!(
        merged.get("fastq.trim").map(String::as_str),
        Some("trimmomatic")
    );
}

#[test]
fn default_pipeline_plan_snapshot_is_stable() {
    let stages = [
        "fastq.validate_pre",
        "fastq.detect_adapters",
        "fastq.trim",
        "fastq.filter",
        "fastq.stats_neutral",
        "fastq.qc_post",
    ];
    let tool_id_for_stage = |stage: &str| -> &'static str {
        match stage {
            "fastq.validate_pre" => "fastqvalidator_official",
            "fastq.detect_adapters" => "fastqc",
            "fastq.trim" => "fastp",
            "fastq.filter" => "fastp",
            "fastq.stats_neutral" => "seqkit_stats",
            "fastq.qc_post" => "multiqc",
            _ => "echo",
        }
    };
    let tools: Vec<ToolExecutionSpecV1> = stages
        .iter()
        .map(|stage| ToolExecutionSpecV1 {
            tool_id: ToolId(tool_id_for_stage(stage).to_string()),
            tool_version: "0.0.0".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test".to_string(),
                digest: Some("sha256:plan".to_string()),
            },
            command: CommandSpecV1 {
                template: vec!["echo".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        })
        .collect();
    let inputs = FastqPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tools,
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: PathBuf::from("reads_R1.fastq.gz"),
        r2: None,
        out_dir: PathBuf::from("out"),
        tool_reasons: None,
    };
    let plan =
        plan_fastq_to_fastq__default__v1(&inputs, DefaultPipelineOptions::default()).expect("plan");
    insta::assert_json_snapshot!("fastq_default_pipeline_plan", plan);
}
