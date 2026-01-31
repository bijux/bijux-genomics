use std::fs;
use std::path::PathBuf;

use bijux_core::ExecutionContract;
use bijux_env_builder::image_qa::{
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
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("sample.txt");
    fs::write(&path, "hello")?;
    let hash = hash_file_sha256(&path)?;
    assert_eq!(
        hash,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    Ok(())
}

#[test]
fn validate_execution_outputs_enforces_contract() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::TempDir::new()?;
    let out_dir = dir.path();
    let expected = out_dir.join("out.fastq.gz");
    fs::write(&expected, "data")?;

    let contract = ExecutionContract {
        required_inputs: vec![],
        expected_outputs: vec!["out.fastq.gz".to_string()],
        forbidden_outputs: vec!["bad.fastq".to_string()],
        forbid_unexpected_outputs: true,
    };

    validate_execution_outputs(&contract, out_dir)?;

    let unexpected = out_dir.join("extra.txt");
    fs::write(&unexpected, "extra")?;
    assert!(validate_execution_outputs(&contract, out_dir).is_err());
    Ok(())
}
