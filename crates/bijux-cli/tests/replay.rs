use assert_cmd::Command;
use bijux_api::v1::run::ExecutionManifest;
use std::path::Path;

fn tempdir_in_repo() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let cwd = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let base = cwd.join("target").join("test-tmp");
    bijux_infra::ensure_dir(&base)?;
    Ok(bijux_infra::temp_dir_in(base, "bijux")?)
}

#[test]
fn replay_runs_manifest_command_and_reproduces_outputs() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir_in_repo()?;
    let run_dir = tmp.path().join("run").join("abc");
    bijux_infra::ensure_dir(&run_dir)?;
    let output_dir = tmp.path().join("out");
    bijux_infra::ensure_dir(&output_dir)?;

    let run_id = "run-123".to_string();
    let output_path = output_dir.join("output.txt");
    let metrics_path = output_dir.join("metrics.json");
    let command = format!(
        "printf 'ok' > '{}' && printf '{{\"metric\":1}}' > '{}'",
        output_path.display(),
        metrics_path.display()
    );

    let manifest = ExecutionManifest {
        run_id: run_id.clone(),
        stage: "fastq.trim".to_string(),
        tool: "fastp".to_string(),
        tool_version: "0.0.0".to_string(),
        image_digest: "sha256:abc".to_string(),
        command: command.clone(),
        input_hashes: vec!["sha256:deadbeef".to_string()],
        input_files: vec!["reads.fastq.gz".to_string()],
        output_dir: output_dir.to_string_lossy().to_string(),
        runner: "docker".to_string(),
        platform: "local".to_string(),
        arch: "arm64".to_string(),
    };
    let manifest_path = run_dir.join("manifest.json");
    bijux_infra::write_bytes(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bijux"));
    cmd.args([
        "replay",
        &run_id,
        "--search-root",
        tmp.path().to_string_lossy().as_ref(),
    ]);
    cmd.assert().success();

    let output = std::fs::read_to_string(&output_path)?;
    let metrics = std::fs::read_to_string(&metrics_path)?;
    assert_eq!(output, "ok");
    assert_eq!(metrics, "{\"metric\":1}");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bijux"));
    cmd.args([
        "replay",
        &run_id,
        "--search-root",
        tmp.path().to_string_lossy().as_ref(),
    ]);
    cmd.assert().success();
    let output2 = std::fs::read_to_string(&output_path)?;
    let metrics2 = std::fs::read_to_string(&metrics_path)?;
    assert_eq!(output2, output);
    assert_eq!(metrics2, metrics);
    Ok(())
}
