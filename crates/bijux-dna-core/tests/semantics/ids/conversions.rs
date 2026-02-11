use std::convert::TryFrom;

use bijux_dna_core::ids::{
    parse_pipeline_id, parse_stage_id, parse_tool_id, validate_pipeline_id,
    validate_pipeline_id_str, validate_stage_id, validate_stage_id_str, validate_tool_id,
    validate_tool_id_str, ArtifactId, PipelineId, ProfileId, RunId, StageId, StepId, ToolId,
};

#[test]
fn id_new_from_static_display_and_as_str_cover_all_types() {
    let stage = StageId::from_static("fastq.trim");
    let step = StepId::from_static("fastq.trim");
    let tool = ToolId::from_static("fastp");
    let artifact = ArtifactId::from_static("reads_out");
    let profile = ProfileId::from_static("default");
    let pipeline = PipelineId::from_static("fastq-to-fastq__default__v1");
    let run = RunId("run-001".to_string());

    assert_eq!(stage.as_str(), "fastq.trim");
    assert_eq!(step.as_str(), "fastq.trim");
    assert_eq!(tool.as_str(), "fastp");
    assert_eq!(artifact.as_str(), "reads_out");
    assert_eq!(profile.as_str(), "default");
    assert_eq!(pipeline.as_str(), "fastq-to-fastq__default__v1");
    assert_eq!(run.as_str(), "run-001");

    assert_eq!(stage.to_string(), "fastq.trim");
    assert_eq!(step.to_string(), "fastq.trim");
    assert_eq!(tool.to_string(), "fastp");
    assert_eq!(artifact.to_string(), "reads_out");
    assert_eq!(profile.to_string(), "default");
    assert_eq!(pipeline.to_string(), "fastq-to-fastq__default__v1");
    assert_eq!(run.to_string(), "run-001");

    let stage_owned = StageId::new("fastq.qc_post");
    let step_owned = StepId::new("fastq.qc_post");
    let tool_owned = ToolId::new("fastqc");
    let artifact_owned = ArtifactId::new("qc_report");
    let profile_owned = ProfileId::new("strict");
    let pipeline_owned = PipelineId::new("fastq-to-fastq__strict__v2");

    assert_eq!(stage_owned.as_str(), "fastq.qc_post");
    assert_eq!(step_owned.as_str(), "fastq.qc_post");
    assert_eq!(tool_owned.as_str(), "fastqc");
    assert_eq!(artifact_owned.as_str(), "qc_report");
    assert_eq!(profile_owned.as_str(), "strict");
    assert_eq!(pipeline_owned.as_str(), "fastq-to-fastq__strict__v2");
}

#[test]
fn id_try_from_and_parse_validate_paths_cover_success_and_failures() {
    let stage = StageId::try_from("fastq.trim");
    let step = StepId::try_from("fastq.trim");
    let tool = ToolId::try_from("fastp");
    let artifact = ArtifactId::try_from("reads_out");
    let profile = ProfileId::try_from("default");
    let pipeline = PipelineId::try_from("fastq-to-fastq__default__v1");

    assert!(stage.as_ref().is_ok_and(|id| validate_stage_id(id).is_ok()));
    assert!(step
        .as_ref()
        .is_ok_and(|id| validate_stage_id_str(id.as_str()).is_ok()));
    assert!(tool.as_ref().is_ok_and(|id| validate_tool_id(id).is_ok()));
    assert!(artifact
        .as_ref()
        .is_ok_and(|id| validate_tool_id_str(id.as_str()).is_ok()));
    assert!(profile
        .as_ref()
        .is_ok_and(|id| validate_tool_id_str(id.as_str()).is_ok()));
    assert!(pipeline
        .as_ref()
        .is_ok_and(|id| validate_pipeline_id(id).is_ok()));

    assert!(parse_stage_id("fastq.trim").is_ok());
    assert!(parse_tool_id("fastp").is_ok());
    assert!(parse_pipeline_id("fastq-to-fastq__default__v1").is_ok());

    assert!(StageId::try_from("").is_err());
    assert!(StepId::try_from("noperiod").is_err());
    assert!(ToolId::try_from("FASTP").is_err());
    assert!(ArtifactId::try_from("bad*name").is_err());
    assert!(ProfileId::try_from("Bad").is_err());
    assert!(PipelineId::try_from("fastq-to-fastq__default").is_err());

    assert!(validate_pipeline_id_str("fastq-to-fastq__default__vx").is_err());
    assert!(validate_pipeline_id_str("fastq-to-fastq__bad!flavor__v1").is_err());
    assert!(validate_pipeline_id_str("badgraph__default__v1").is_err());
    assert!(validate_pipeline_id_str("fastq-to-fastq__default__v1").is_ok());
}
