use super::identity::{
    execution_pipeline_identity, execution_sample_identity, hash_path,
    infer_tool_version_from_image, runtime_platform_identity,
};
use super::inputs::container_input_mapping;
use super::observer::build_observer_command_args;
use super::runtime_policy::stage_workdir_for_value;
use super::{build_apptainer_exec_args, container_command_template, hash_inputs};
use anyhow::anyhow;
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_environment::api::RuntimeKind;
use serde_json::json;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

type StepFixture = ExecutionStep;

#[test]
fn observer_args_use_docker_mounts_for_docker_runner() {
    let (bin, args) = build_observer_command_args(
        "bijuxdna/seqkit:latest-pinned-amd64",
        Path::new("/artifacts/runtime/input"),
        &["stats".to_string(), "/data/reads.fq.gz".to_string()],
        RuntimeKind::Docker,
    );

    assert_eq!(bin, "docker");
    assert!(args.starts_with(&["run".to_string(), "--rm".to_string()]));
    assert!(args.contains(&"/artifacts/runtime/input:/data:ro".to_string()));
}

#[test]
fn execution_identity_defaults_to_stage_and_step_ids() {
    let step = execution_step_fixture(
        "sample-0001.reads.trim.fastp",
        "stage.trim",
        &["fastp".to_string()],
        "fastp:0.23.4",
        Vec::new(),
        Vec::new(),
        "/artifacts/runtime/out",
    );

    assert_eq!(execution_pipeline_identity(&step), "stage.trim");
    assert_eq!(execution_sample_identity(&step), "sample-0001.reads.trim.fastp");
}

#[test]
fn image_version_inference_distinguishes_registry_ports_from_tags() {
    assert_eq!(infer_tool_version_from_image("bijux/toolx:0.12.1"), "0.12.1");
    assert_eq!(infer_tool_version_from_image("localhost:5000/bijux/toolx:0.12.1"), "0.12.1");
    assert_eq!(infer_tool_version_from_image("localhost:5000/bijux/toolx"), "unknown");
    assert_eq!(infer_tool_version_from_image("bijux/toolx:latest@sha256:abc123"), "unknown");
}

#[test]
fn runtime_platform_identity_defaults_to_runner_name() {
    assert_eq!(runtime_platform_identity(RuntimeKind::Local), "local");
    assert_eq!(runtime_platform_identity(RuntimeKind::Docker), "docker");
    assert_eq!(runtime_platform_identity(RuntimeKind::Apptainer), "apptainer");
    assert_eq!(runtime_platform_identity(RuntimeKind::Singularity), "singularity");
}

#[test]
fn stage_workdir_rejects_paths_that_escape_output_root() {
    let out_dir = Path::new("/artifacts/runtime/out");

    assert_eq!(
        stage_workdir_for_value(out_dir, "/artifacts/runtime/out/nested/work"),
        "/data/output/nested/work"
    );
    assert_eq!(
        stage_workdir_for_value(out_dir, "/artifacts/runtime/out/../outside"),
        "/data/output"
    );
    assert_eq!(stage_workdir_for_value(out_dir, "/artifacts/runtime/outside/work"), "/data/output");
}

#[test]
fn observer_args_use_apptainer_exec_for_apptainer_runner() {
    let (bin, args) = build_observer_command_args(
        "/containers/seqkit.sif",
        Path::new("/artifacts/runtime/input"),
        &["stats".to_string(), "/data/reads.fq.gz".to_string()],
        RuntimeKind::Apptainer,
    );

    assert_eq!(bin, "apptainer");
    assert!(args.starts_with(&[
        "exec".to_string(),
        "--cleanenv".to_string(),
        "--no-home".to_string(),
        "--containall".to_string()
    ]));
    assert!(args.contains(&"--bind".to_string()));
    assert!(args.contains(&"/artifacts/runtime/input:/data:ro".to_string()));
    assert!(args.contains(&"/containers/seqkit.sif".to_string()));
}

#[test]
fn apptainer_exec_defaults_workdir_to_output_mount() -> anyhow::Result<()> {
    let step = execution_step_fixture(
        "step.trim_reads.tool.seqkit",
        "stage.trim",
        &["seqkit".to_string(), "stats".to_string()],
        "/containers/seqkit.sif",
        vec![json!({
            "artifact_id": "reads",
            "path": "/artifacts/runtime/input/sample.fq.gz",
            "role": "reads",
            "required": true
        })],
        vec![json!({
            "artifact_id": "report",
            "path": "/artifacts/runtime/out/report.json",
            "role": "report_json",
            "required": true
        })],
        "/artifacts/runtime/out",
    );

    let args = build_apptainer_exec_args(
        &step,
        &[PathBuf::from("/artifacts/runtime/input/sample.fq.gz")],
        Path::new("/artifacts/runtime/input"),
        Path::new("/artifacts/runtime/out"),
        RuntimeKind::Apptainer,
    )?;

    let pwd_index = args
        .iter()
        .position(|value| value == "--pwd")
        .ok_or_else(|| anyhow!("missing --pwd flag in apptainer args"))?;
    assert_eq!(args[pwd_index + 1], "/data/output");
    assert!(args.contains(&"/artifacts/runtime/out:/data/output:rw".to_string()));
    Ok(())
}

#[test]
fn container_command_template_rewrites_mounted_input_and_output_paths() {
    let template = vec![
        "sh".to_string(),
        "-lc".to_string(),
        "validator /artifacts/runtime/corpus/sample_0004_R1.fq.gz > /artifacts/runtime/out/validation_r1.log && printf '%s' /artifacts/runtime/out/validation.json"
            .to_string(),
    ];

    let rewritten = container_command_template(
        &template,
        Path::new("/artifacts/runtime/corpus"),
        Path::new("/artifacts/runtime/out"),
        false,
    );

    assert_eq!(rewritten[0], "sh");
    assert!(rewritten[2].contains("/data/input/sample_0004_R1.fq.gz"));
    assert!(rewritten[2].contains("/data/output/validation_r1.log"));
    assert!(rewritten[2].contains("/data/output/validation.json"));
}

#[test]
fn container_command_template_rewrites_single_file_mounts_inside_shell_scripts(
) -> anyhow::Result<()> {
    let temp = tempdir()?;
    let input = temp.path().join("sample_0004_R1.fq.gz");
    bijux_dna_infra::write_bytes(&input, b"@read\nACGT\n+\n!!!!\n")?;
    let out_dir = temp.path().join("out");
    bijux_dna_infra::ensure_dir(&out_dir)?;
    let template = vec![
        "sh".to_string(),
        "-lc".to_string(),
        format!(
            "seqkit fx2tab -j 1 -n -s {} > {}",
            input.display(),
            out_dir.join("reads.tsv").display()
        ),
    ];

    let rewritten = container_command_template(&template, &input, &out_dir, false);

    assert_eq!(rewritten[0], "sh");
    assert!(rewritten[2].contains("seqkit fx2tab -j 1 -n -s /data/input/sample_0004_R1.fq.gz"));
    assert!(rewritten[2].contains("> /data/output/reads.tsv"));
    Ok(())
}

#[test]
fn container_command_template_rewrites_exact_output_root_inside_shell_scripts() {
    let template = vec![
        "sh".to_string(),
        "-lc".to_string(),
        "flash2 -o flash2 -d /artifacts/runtime/out -t 1 /artifacts/runtime/corpus/sample_0004_R1.fq.gz /artifacts/runtime/corpus/sample_0004_R2.fq.gz"
            .to_string(),
    ];

    let rewritten = container_command_template(
        &template,
        Path::new("/artifacts/runtime/corpus"),
        Path::new("/artifacts/runtime/out"),
        false,
    );

    assert_eq!(rewritten[0], "sh");
    assert!(rewritten[2].contains("-d /data/output"));
    assert!(rewritten[2].contains("/data/input/sample_0004_R1.fq.gz"));
    assert!(rewritten[2].contains("/data/input/sample_0004_R2.fq.gz"));
}

#[test]
fn container_command_template_does_not_rewrite_sibling_prefix_paths() {
    let template = vec![
        "sh".to_string(),
        "-lc".to_string(),
        "cat /artifacts/runtime/input-old/sample.fq > /artifacts/runtime/out-old/report.txt"
            .to_string(),
    ];

    let rewritten = container_command_template(
        &template,
        Path::new("/artifacts/runtime/input"),
        Path::new("/artifacts/runtime/out"),
        false,
    );

    assert!(rewritten[2].contains("/artifacts/runtime/input-old/sample.fq"));
    assert!(rewritten[2].contains("/artifacts/runtime/out-old/report.txt"));
}

#[test]
fn container_command_template_keeps_output_paths_writable_when_out_dir_is_under_input_root() {
    let template = vec![
        "sh".to_string(),
        "-lc".to_string(),
        "printf '%s' /artifacts/runtime/results/benchmark_corpus/reads.report_summary/cluster/bench/report_summary/sample_0001/tools/multiqc/report_summary.json > /artifacts/runtime/results/benchmark_corpus/reads.report_summary/cluster/bench/report_summary/sample_0001/tools/multiqc/report_summary.json".to_string(),
    ];

    let rewritten = container_command_template(
        &template,
        Path::new("/artifacts/runtime/results/benchmark_corpus"),
        Path::new(
            "/artifacts/runtime/results/benchmark_corpus/reads.report_summary/cluster/bench/report_summary/sample_0001/tools/multiqc",
        ),
        false,
    );

    assert_eq!(rewritten[0], "sh");
    assert!(rewritten[2].contains("/data/output/report_summary.json"));
    assert!(!rewritten[2].contains("/data/input/reads.report_summary"));
}

#[test]
fn container_input_mapping_preserves_single_file_basename() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let input = temp.path().join("sample_0004_R1.fq.gz");
    bijux_dna_infra::write_bytes(&input, b"@read\nACGT\n+\n!!!!\n")?;

    let (mount_root, container_root) = container_input_mapping(&input);

    assert_eq!(mount_root, temp.path());
    assert_eq!(container_root, "/data/input/sample_0004_R1.fq.gz");
    Ok(())
}

#[test]
fn container_command_template_preserves_absolute_inputs_for_mixed_roots() {
    let template = vec![
        "bowtie2".to_string(),
        "-x".to_string(),
        "/opt/reference/panel/phix/bowtie2/reference".to_string(),
        "-S".to_string(),
        "/dev/null".to_string(),
        "-1".to_string(),
        "/data/benchmark_corpus/normalized/sample_0001_R1.fq.gz".to_string(),
        "--met-file".to_string(),
        "/artifacts/runtime/out/bowtie2.metrics.txt".to_string(),
    ];

    let rewritten = container_command_template(
        &template,
        Path::new("/"),
        Path::new("/artifacts/runtime/out"),
        true,
    );

    assert_eq!(rewritten[2], template[2]);
    assert_eq!(rewritten[4], "/dev/null");
    assert_eq!(rewritten[6], template[6]);
    assert_eq!(rewritten[8], "/data/output/bowtie2.metrics.txt");
}

#[test]
fn hash_path_supports_directory_outputs() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let root = temp.path().join("summary_bundle");
    bijux_dna_infra::ensure_dir(root.join("nested"))?;
    bijux_dna_infra::write_bytes(root.join("nested").join("summary.txt"), b"bundle-summary")?;

    let digest = hash_path(&root)?;

    assert_eq!(digest.len(), 64);
    Ok(())
}

#[test]
fn hash_path_supports_directory_symlinks() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let root = temp.path().join("taxonomy");
    bijux_dna_infra::ensure_dir(root.join("kraken2"))?;
    bijux_dna_infra::write_bytes(root.join("kraken2").join("hash.k2d"), b"kraken-hash")?;
    std::os::unix::fs::symlink(root.join("kraken2"), root.join("krakenuniq"))?;

    let digest = hash_path(&root)?;

    assert_eq!(digest.len(), 64);
    Ok(())
}

#[test]
fn hash_inputs_rejects_missing_paths() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let missing = temp.path().join("missing.txt");

    let err = match hash_inputs(&[missing]) {
        Ok(_) => panic!("missing input must fail"),
        Err(err) => err,
    };

    assert!(err.to_string().contains("declared input path does not exist"));
    Ok(())
}

#[test]
fn hash_inputs_hashes_directories() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let root = temp.path().join("summary_bundle");
    bijux_dna_infra::ensure_dir(&root)?;
    bijux_dna_infra::write_bytes(root.join("summary.txt"), b"bundle-summary")?;

    let hashes = hash_inputs(&[root])?;

    assert_eq!(hashes.len(), 1);
    assert_eq!(hashes[0].len(), 64);
    Ok(())
}

fn execution_step_fixture(
    step_id: &str,
    stage_id: &str,
    command_template: &[String],
    image: &str,
    inputs: Vec<serde_json::Value>,
    outputs: Vec<serde_json::Value>,
    out_dir: &str,
) -> StepFixture {
    let normalize_io = |items: Vec<serde_json::Value>| {
        items
            .into_iter()
            .map(|item| {
                let name = item
                    .get("name")
                    .or_else(|| item.get("artifact_id"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("artifact")
                    .to_string();
                let path = item
                    .get("path")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let role = item
                    .get("role")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown")
                    .to_string();
                let optional = item
                    .get("optional")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or_else(|| {
                        !item.get("required").and_then(serde_json::Value::as_bool).unwrap_or(true)
                    });
                json!({
                    "name": name,
                    "path": path,
                    "role": role,
                    "optional": optional,
                })
            })
            .collect::<Vec<_>>()
    };

    serde_json::from_value(json!({
        "step_id": step_id,
        "stage_id": stage_id,
        "command": { "template": command_template },
        "image": { "image": image, "digest": null },
        "resources": {
            "runtime": "local",
            "mem_gb": 1,
            "tmp_gb": 1,
            "threads": 1
        },
        "io": {
            "inputs": normalize_io(inputs),
            "outputs": normalize_io(outputs)
        },
        "out_dir": out_dir,
        "aux_images": {},
        "expected_artifact_ids": [],
        "metrics_schema_ids": []
    }))
    .unwrap_or_else(|err| panic!("execution step fixture must deserialize: {err}"))
}
