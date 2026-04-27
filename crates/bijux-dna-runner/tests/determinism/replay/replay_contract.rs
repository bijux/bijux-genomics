use anyhow::Result;
use bijux_dna_core::contract::{ContractVersion, ExecutionManifest};
use bijux_dna_runner::backend::replay_run;

#[test]
fn replay_verifies_manifest_without_executing_command() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bijux-dna-replay")?;
    let root = temp.path();
    let input = root.join("input.txt");
    bijux_dna_infra::write_bytes(&input, "ACGT")?;
    let out_dir = root.join("run");
    bijux_dna_infra::ensure_dir(&out_dir)?;
    let output = out_dir.join("output.txt");
    let command_marker = out_dir.join("command-ran.txt");
    bijux_dna_infra::write_bytes(&output, "recorded-output")?;

    let manifest = ExecutionManifest {
        contract_version: ContractVersion::v1(),
        run_id: "run-1".to_string(),
        stage: "fastq.trim_reads".to_string(),
        tool: "tool".to_string(),
        tool_version: "0.0.1-test".to_string(),
        image_digest: "sha256:synthetic-image".to_string(),
        command: format!(
            "printf changed > {} && touch {}",
            output.display(),
            command_marker.display()
        ),
        input_hashes: vec![bijux_dna_infra::hash_file_sha256(&input)?],
        input_files: vec![input.display().to_string()],
        output_dir: out_dir.display().to_string(),
        runner: "docker".to_string(),
        platform: "test".to_string(),
        arch: "x86_64".to_string(),
    };
    bijux_dna_infra::atomic_write_json(&out_dir.join("manifest.json"), &manifest)?;

    replay_run("run-1", root)?;
    assert_eq!(std::fs::read_to_string(&output)?, "recorded-output");
    assert!(!command_marker.exists(), "replay must not execute manifest.command");
    Ok(())
}

#[test]
fn replay_rejects_changed_input_hashes() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bijux-dna-replay-input-hash")?;
    let root = temp.path();
    let input = root.join("input.txt");
    bijux_dna_infra::write_bytes(&input, "ACGT")?;
    let out_dir = root.join("run");
    bijux_dna_infra::ensure_dir(&out_dir)?;

    let manifest = ExecutionManifest {
        contract_version: ContractVersion::v1(),
        run_id: "run-1".to_string(),
        stage: "fastq.trim_reads".to_string(),
        tool: "tool".to_string(),
        tool_version: "0.0.1-test".to_string(),
        image_digest: "sha256:synthetic-image".to_string(),
        command: "printf should-not-run".to_string(),
        input_hashes: vec![bijux_dna_infra::hash_file_sha256(&input)?],
        input_files: vec![input.display().to_string()],
        output_dir: out_dir.display().to_string(),
        runner: "docker".to_string(),
        platform: "test".to_string(),
        arch: "x86_64".to_string(),
    };
    bijux_dna_infra::atomic_write_json(&out_dir.join("manifest.json"), &manifest)?;
    bijux_dna_infra::write_bytes(&input, "TGCA")?;

    let err = match replay_run("run-1", root) {
        Ok(()) => panic!("changed input hash must fail replay"),
        Err(error) => error,
    };
    assert!(err.to_string().contains("replay input hash mismatch"));
    Ok(())
}
