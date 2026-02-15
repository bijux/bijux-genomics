
use super::*;
use bijux_dna_core::contract::{ArtifactRole, ArtifactSpec, StageIO, ToolConstraints};
use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
use tempfile::TempDir;

#[test]
fn tool_exec_bcftools_version_in_container() -> Result<()> {
    if std::env::var("BIJUX_E2E").is_err() {
        return Ok(());
    }
    let tmp = TempDir::new()?;
    let stage_root = tmp.path().join("artifacts").join("stage");
    let out_root = tmp.path().join("out");
    let in_root = tmp.path().join("in");
    bijux_dna_infra::ensure_dir(&stage_root)?;
    bijux_dna_infra::ensure_dir(&out_root)?;
    bijux_dna_infra::ensure_dir(&in_root)?;
    let out_file = out_root.join("bcftools.version.txt");
    let step = ExecutionStep {
        step_id: StepId::new("vcf.stats.bcftools_version"),
        stage_id: StageId::new("vcf.stats"),
        command: CommandSpecV1 {
            template: vec![
                "sh".to_string(),
                "-lc".to_string(),
                "bcftools --version > /data/output/bcftools.version.txt".to_string(),
            ],
        },
        image: ContainerImageRefV1 {
            image: "quay.io/biocontainers/bcftools:1.20--h8b25389_0".to_string(),
            digest: Some(
                "sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6"
                    .to_string(),
            ),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![],
            outputs: vec![ArtifactSpec::required(
                ArtifactId::new("version"),
                out_file.clone(),
                ArtifactRole::Log,
            )],
        },
        out_dir: out_root.clone(),
        aux_images: std::collections::BTreeMap::new(),
        expected_artifact_ids: vec![],
        metrics_schema_ids: vec![],
    };
    let req = ToolInvocationRequest {
        step,
        runner: RuntimeKind::Docker,
        context: ToolContext {
            run_id: "run-e2e-tool-exec".to_string(),
            stage_id: "vcf.stats".to_string(),
            tool_id: "bcftools".to_string(),
            sample_id: None,
            stage_root: stage_root.clone(),
            input_root: in_root,
            output_root: out_root.clone(),
            tmp_root: stage_root.join("tmp"),
            threads: 1,
            memory_hint_mb: Some(512),
            compression_threads: Some(1),
            seed: Some(7),
            network_policy: NetworkPolicy::Forbid,
        },
        timeout: None,
        mode: ToolExecMode::Execute,
    };
    let result = ToolExec::invoke(&req)?;
    assert_eq!(result.stage_result.exit_code, 0);
    let version = std::fs::read_to_string(out_file)?;
    assert!(
        version.to_ascii_lowercase().contains("bcftools"),
        "unexpected bcftools --version output: {version}"
    );
    Ok(())
}

#[test]
fn dry_run_explain_emits_plan_and_resource_details() -> Result<()> {
    let tmp = TempDir::new()?;
    let stage_root = tmp.path().join("artifacts").join("stage");
    let out_root = tmp.path().join("out");
    let in_root = tmp.path().join("in");
    bijux_dna_infra::ensure_dir(&stage_root)?;
    bijux_dna_infra::ensure_dir(&out_root)?;
    bijux_dna_infra::ensure_dir(&in_root)?;
    let step = ExecutionStep {
        step_id: StepId::new("vcf.qc.dry_run_explain"),
        stage_id: StageId::new("vcf.qc"),
        command: CommandSpecV1 {
            template: vec!["bcftools".to_string(), "--version".to_string()],
        },
        image: ContainerImageRefV1 {
            image: "quay.io/biocontainers/bcftools:1.20--h8b25389_0".to_string(),
            digest: Some(
                "sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6"
                    .to_string(),
            ),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![],
            outputs: vec![],
        },
        out_dir: out_root.clone(),
        aux_images: std::collections::BTreeMap::new(),
        expected_artifact_ids: vec![],
        metrics_schema_ids: vec![],
    };
    let req = ToolInvocationRequest {
        step,
        runner: RuntimeKind::Docker,
        context: ToolContext {
            run_id: "run-dry-run-explain".to_string(),
            stage_id: "vcf.qc".to_string(),
            tool_id: "bcftools".to_string(),
            sample_id: None,
            stage_root: stage_root.clone(),
            input_root: in_root,
            output_root: out_root.clone(),
            tmp_root: stage_root.join("tmp"),
            threads: 1,
            memory_hint_mb: Some(256),
            compression_threads: Some(1),
            seed: Some(11),
            network_policy: NetworkPolicy::Forbid,
        },
        timeout: None,
        mode: ToolExecMode::DryRunExplain,
    };
    let result = ToolExec::invoke(&req)?;
    assert_eq!(result.stage_result.exit_code, 0);
    let explain_path = stage_root.join("dry_run_explain.json");
    assert!(explain_path.exists());
    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(explain_path)?)?;
    assert_eq!(
        payload
            .get("schema_version")
            .and_then(serde_json::Value::as_str),
        Some("bijux.dry_run_explain.v1")
    );
    Ok(())
}
