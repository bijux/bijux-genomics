#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[cfg(not(feature = "bam_downstream"))]
#[test]
fn bench_local_render_slurm_scripts_bam_requires_bam_downstream_feature() {
    let output = run_cli(&["bench", "local", "render-slurm-scripts", "--domain", "bam", "--json"]);
    assert!(!output.status.success(), "command should fail without bam_downstream");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("requires the `bam_downstream` feature"),
        "stderr must explain the BAM feature gate, got:\n{stderr}"
    );
}

#[test]
fn bench_local_render_slurm_scripts_fastq_json_reports_governed_27_stage_slice() {
    let payload =
        run_cli_json(&["bench", "local", "render-slurm-scripts", "--domain", "fastq", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_slurm_dry_run.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("fastq"));
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("target/slurm-dry-run/fastq")
    );
    assert_eq!(payload.get("script_count").and_then(serde_json::Value::as_u64), Some(27));
    let scripts =
        payload.get("scripts").and_then(serde_json::Value::as_array).expect("scripts array");
    assert_eq!(scripts.len(), 27);
    assert!(scripts.iter().all(|entry| {
        entry.get("stage_id").and_then(serde_json::Value::as_str).is_some()
            && entry.get("tool_id").and_then(serde_json::Value::as_str).is_some()
            && entry
                .get("script_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| path.starts_with("target/slurm-dry-run/fastq/") && path.ends_with(".sbatch"))
            && entry
                .get("command")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|command| command.contains("bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq."))
    }));
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_render_slurm_scripts_bam_json_reports_governed_24_stage_slice() {
    let payload =
        run_cli_json(&["bench", "local", "render-slurm-scripts", "--domain", "bam", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_slurm_dry_run.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("bam"));
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("target/slurm-dry-run/bam")
    );
    assert_eq!(payload.get("script_count").and_then(serde_json::Value::as_u64), Some(24));
    let scripts =
        payload.get("scripts").and_then(serde_json::Value::as_array).expect("scripts array");
    assert_eq!(scripts.len(), 24);
    assert!(scripts.iter().all(|entry| {
        entry.get("stage_id").and_then(serde_json::Value::as_str).is_some()
            && entry.get("tool_id").and_then(serde_json::Value::as_str).is_some()
            && entry
                .get("script_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| path.starts_with("target/slurm-dry-run/bam/") && path.ends_with(".sbatch"))
            && entry
                .get("command")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|command| command.contains("bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam."))
    }));
}

#[test]
fn bench_local_render_slurm_scripts_fastq_writes_bash_parseable_27_script_slice() {
    let output = run_cli(&["bench", "local", "render-slurm-scripts", "--domain", "fastq"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let repo_root = support::repo_root().expect("repo root");
    let output_root = repo_root.join("target/slurm-dry-run/fastq");
    assert!(output_root.is_dir(), "FASTQ slurm dry-run root must exist");

    let mut scripts = std::fs::read_dir(&output_root)
        .expect("read output root")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sbatch"))
        .collect::<Vec<_>>();
    scripts.sort();

    assert_eq!(scripts.len(), 27, "FASTQ dry-run root must contain 27 sbatch scripts");
    for script in &scripts {
        let syntax = Command::new("bash").arg("-n").arg(script).output().expect("run bash -n");
        assert!(
            syntax.status.success(),
            "bash -n failed for {}: {}",
            script.display(),
            String::from_utf8_lossy(&syntax.stderr)
        );
    }

    let validate_reads_script = output_root.join("fastq.validate_reads.sbatch");
    let script_body =
        std::fs::read_to_string(&validate_reads_script).expect("read validate script");
    assert!(script_body.contains("#SBATCH --job-name=fastq-validate_reads"));
    assert!(
        script_body.contains(
            "cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.validate_reads"
        )
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_render_slurm_scripts_bam_writes_bash_parseable_24_script_slice() {
    let output = run_cli(&["bench", "local", "render-slurm-scripts", "--domain", "bam"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let repo_root = support::repo_root().expect("repo root");
    let output_root = repo_root.join("target/slurm-dry-run/bam");
    assert!(output_root.is_dir(), "BAM slurm dry-run root must exist");

    let mut scripts = std::fs::read_dir(&output_root)
        .expect("read output root")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sbatch"))
        .collect::<Vec<_>>();
    scripts.sort();

    assert_eq!(scripts.len(), 24, "BAM dry-run root must contain 24 sbatch scripts");
    for script in &scripts {
        let syntax = Command::new("bash").arg("-n").arg(script).output().expect("run bash -n");
        assert!(
            syntax.status.success(),
            "bash -n failed for {}: {}",
            script.display(),
            String::from_utf8_lossy(&syntax.stderr)
        );
    }

    let genotyping_script = output_root.join("bam.genotyping.sbatch");
    let script_body = std::fs::read_to_string(&genotyping_script).expect("read genotyping script");
    assert!(script_body.contains("#SBATCH --job-name=bam-genotyping"));
    assert!(
        script_body.contains(
            "cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.genotyping"
        )
    );
}
