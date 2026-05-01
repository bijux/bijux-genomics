use std::path::PathBuf;

use bijux_dna_core::contract::ExecutionContract;
use bijux_dna_environment::api::{
    resolve_image, select_best_runner, PlatformSpec, RuntimeKind, ToolImageSpec,
};
use bijux_dna_environment_qa::image_qa::{
    hash_file_sha256, image_qa_base_dir, image_qa_jsonl_path, image_qa_sqlite_path,
    validate_execution_outputs,
};

#[test]
fn image_qa_paths_are_stable() {
    let cwd = PathBuf::from("/tmp/bijux");
    let base = image_qa_base_dir(&cwd, "docker");
    assert!(base.ends_with("bijux/artifacts/image-qa/docker"));
    let jsonl = image_qa_jsonl_path(&cwd, "docker");
    assert!(jsonl.ends_with("bijux/artifacts/image-qa/docker/qa.jsonl"));
    let sqlite = image_qa_sqlite_path(&cwd, "docker");
    assert!(sqlite.ends_with("bijux/artifacts/image-qa/docker/qa.sqlite"));
}

#[test]
fn hash_file_sha256_matches_content() -> Result<(), Box<dyn std::error::Error>> {
    let dir = bijux_dna_testkit::tempdir_for("environment-qa-hash-file");
    let path = dir.path().join("sample.txt");
    bijux_dna_infra::write_bytes(&path, "hello")?;
    let hash = hash_file_sha256(&path)?;
    assert_eq!(hash, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    Ok(())
}

#[test]
fn validate_execution_outputs_enforces_contract() -> Result<(), Box<dyn std::error::Error>> {
    let dir = bijux_dna_testkit::tempdir_for("environment-qa-output-contract");
    let out_dir = dir.path();
    let expected = out_dir.join("out.fastq.gz");
    bijux_dna_infra::write_bytes(&expected, "data")?;

    let contract = ExecutionContract {
        required_inputs: vec![],
        optional_inputs: vec![],
        expected_outputs: vec!["out.fastq.gz".to_string()],
        optional_outputs: vec![],
        forbidden_outputs: vec!["bad.fastq".to_string()],
        forbid_unexpected_outputs: true,
        requires_provenance: false,
    };

    validate_execution_outputs(&contract, out_dir)?;

    let unexpected = out_dir.join("extra.txt");
    bijux_dna_infra::write_bytes(&unexpected, "extra")?;
    assert!(validate_execution_outputs(&contract, out_dir).is_err());
    Ok(())
}

#[test]
fn validate_execution_outputs_allows_optional_outputs_in_strict_mode(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = bijux_dna_testkit::tempdir_for("environment-qa-optional-output-contract");
    let out_dir = dir.path();
    bijux_dna_infra::write_bytes(out_dir.join("out.fastq.gz"), "data")?;
    bijux_dna_infra::write_bytes(out_dir.join("metrics.json"), "{}")?;

    let contract = ExecutionContract {
        required_inputs: vec![],
        optional_inputs: vec![],
        expected_outputs: vec!["out.fastq.gz".to_string()],
        optional_outputs: vec!["metrics.json".to_string()],
        forbidden_outputs: vec![],
        forbid_unexpected_outputs: true,
        requires_provenance: false,
    };

    validate_execution_outputs(&contract, out_dir)?;
    Ok(())
}

#[test]
fn runner_selection_falls_back_in_stable_order() -> Result<(), Box<dyn std::error::Error>> {
    let preferred = RuntimeKind::Docker;
    let selected = select_best_runner(preferred, &[RuntimeKind::Singularity, RuntimeKind::Docker])?;
    assert_eq!(selected, RuntimeKind::Docker);

    let selected = select_best_runner(preferred, &[RuntimeKind::Apptainer])?;
    assert_eq!(selected, RuntimeKind::Apptainer);
    Ok(())
}

#[test]
fn image_resolution_prefers_digest_and_rejects_base_name() -> Result<(), Box<dyn std::error::Error>>
{
    let platform = PlatformSpec {
        name: "test".to_string(),
        runner: RuntimeKind::Docker,
        container_dir: PathBuf::from("containers"),
        image_prefix: "local".to_string(),
        arch: "x86_64".to_string(),
    };
    let pinned = resolve_image(
        &ToolImageSpec {
            tool: "fastp".to_string(),
            version: "0.23.4".to_string(),
            digest: Some("sha256:abc".to_string()),
            enabled: None,
            shipping_policy: None,
        },
        &platform,
    )?;
    assert_eq!(pinned.full_name, "local/fastp@sha256:abc");

    let err = match resolve_image(
        &ToolImageSpec {
            tool: "base-image".to_string(),
            version: "1.0".to_string(),
            digest: None,
            enabled: None,
            shipping_policy: None,
        },
        &platform,
    ) {
        Ok(image) => {
            return Err(format!("expected error, got image {}", image.full_name).into());
        }
        Err(err) => err,
    };
    assert!(err.to_string().contains("must not reference base"));
    Ok(())
}
