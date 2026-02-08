use anyhow::Result;
use bijux_core::contract::{ContractVersion, ExecutionManifest};
use bijux_runner::replay_run;

#[test]
fn replay_is_deterministic() -> Result<()> {
    let temp = bijux_infra::temp_dir("bijux-replay-determinism")?;
    let root = temp.path();
    let input = root.join("input.txt");
    bijux_infra::write_bytes(&input, "ACGT")?;
    let out_dir = root.join("run");
    bijux_infra::ensure_dir(&out_dir)?;
    let output = out_dir.join("output.txt");

    let manifest = ExecutionManifest {
        contract_version: ContractVersion::v1(),
        run_id: "run-1".to_string(),
        stage: "fastq.trim".to_string(),
        tool: "tool".to_string(),
        tool_version: "0.0.0".to_string(),
        image_digest: "sha256:img".to_string(),
        command: format!("cat {} > {}", input.display(), output.display()),
        input_hashes: vec![bijux_infra::hash_file_sha256(&input)?],
        input_files: vec![input.display().to_string()],
        output_dir: out_dir.display().to_string(),
        runner: "local".to_string(),
        platform: "test".to_string(),
        arch: "x86_64".to_string(),
    };
    bijux_infra::atomic_write_json(&out_dir.join("manifest.json"), &manifest)?;

    replay_run("run-1", root)?;
    let hash_first = bijux_infra::hash_file_sha256(&output)?;
    std::fs::remove_file(&output)?;
    replay_run("run-1", root)?;
    let hash_second = bijux_infra::hash_file_sha256(&output)?;

    assert_eq!(hash_first, hash_second);
    Ok(())
}
