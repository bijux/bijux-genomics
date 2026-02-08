use anyhow::Result;
use bijux_dna_core::contract::{ContractVersion, ExecutionManifest};
use bijux_dna_runner::backend::replay_run;

#[test]
fn replay_regenerates_outputs_from_manifest() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bijux-replay")?;
    let root = temp.path();
    let input = root.join("input.txt");
    bijux_dna_infra::write_bytes(&input, "ACGT")?;
    let out_dir = root.join("run");
    bijux_dna_infra::ensure_dir(&out_dir)?;
    let output = out_dir.join("output.txt");

    let manifest = ExecutionManifest {
        contract_version: ContractVersion::v1(),
        run_id: "run-1".to_string(),
        stage: "fastq.trim".to_string(),
        tool: "tool".to_string(),
        tool_version: "0.0.0".to_string(),
        image_digest: "sha256:img".to_string(),
        command: format!("cat {} > {}", input.display(), output.display()),
        input_hashes: vec![bijux_dna_infra::hash_file_sha256(&input)?],
        input_files: vec![input.display().to_string()],
        output_dir: out_dir.display().to_string(),
        runner: "docker".to_string(),
        platform: "test".to_string(),
        arch: "x86_64".to_string(),
    };
    bijux_dna_infra::atomic_write_json(&out_dir.join("manifest.json"), &manifest)?;

    replay_run("run-1", root)?;
    let hash_a = bijux_dna_infra::hash_file_sha256(&output)?;
    std::fs::remove_file(&output)?;
    replay_run("run-1", root)?;
    let hash_b = bijux_dna_infra::hash_file_sha256(&output)?;
    assert_eq!(hash_a, hash_b);
    Ok(())
}
