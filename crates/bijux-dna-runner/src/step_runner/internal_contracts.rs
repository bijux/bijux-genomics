use super::identity::{
    execution_pipeline_identity, execution_sample_identity, hash_path, runtime_platform_identity,
};
use super::inputs::container_input_mapping;
use super::observer::build_observer_command_args;
use super::{build_apptainer_exec_args, container_command_template, hash_inputs};
use anyhow::anyhow;
use bijux_dna_core::contract::{ExecutionStep, StageIO, ToolConstraints};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, StageId, StepId,
};
use bijux_dna_environment::api::RuntimeKind;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn observer_args_use_docker_mounts_for_docker_runner() {
    let (bin, args) = build_observer_command_args(
        "bijuxdna/seqkit:latest-pinned-amd64",
        Path::new("/tmp/input"),
        &["stats".to_string(), "/data/reads.fq.gz".to_string()],
        RuntimeKind::Docker,
    );

    assert_eq!(bin, "docker");
    assert!(args.starts_with(&["run".to_string(), "--rm".to_string()]));
    assert!(args.contains(&"/tmp/input:/data:ro".to_string()));
}

#[test]
fn execution_identity_defaults_to_stage_and_step_ids() {
    let step = ExecutionStep {
        step_id: StepId::from_static("sample-0001.reads.trim.fastp"),
        stage_id: StageId::from_static("stage.trim"),
        command: CommandSpecV1 { template: vec!["fastp".to_string()] },
        image: ContainerImageRefV1 { image: "fastp:0.23.4".to_string(), digest: None },
        resources: ToolConstraints::default(),
        io: StageIO { inputs: Vec::new(), outputs: Vec::new() },
        out_dir: PathBuf::from("/tmp/out"),
        aux_images: std::collections::BTreeMap::default(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };

    assert_eq!(execution_pipeline_identity(&step), "stage.trim");
    assert_eq!(execution_sample_identity(&step), "sample-0001.reads.trim.fastp");
}

#[test]
fn runtime_platform_identity_defaults_to_runner_name() {
    assert_eq!(runtime_platform_identity(RuntimeKind::Docker), "docker");
    assert_eq!(runtime_platform_identity(RuntimeKind::Apptainer), "apptainer");
    assert_eq!(runtime_platform_identity(RuntimeKind::Singularity), "singularity");
}

#[test]
fn observer_args_use_apptainer_exec_for_apptainer_runner() {
    let (bin, args) = build_observer_command_args(
        "/containers/seqkit.sif",
        Path::new("/tmp/input"),
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
    assert!(args.contains(&"/tmp/input:/data:ro".to_string()));
    assert!(args.contains(&"/containers/seqkit.sif".to_string()));
}

#[test]
fn apptainer_exec_defaults_workdir_to_output_mount() -> anyhow::Result<()> {
    let step = ExecutionStep {
        step_id: StepId::from_static("step.trim_reads.tool.seqkit"),
        stage_id: StageId::from_static("stage.trim"),
        command: CommandSpecV1 { template: vec!["seqkit".to_string(), "stats".to_string()] },
        image: ContainerImageRefV1 { image: "/containers/seqkit.sif".to_string(), digest: None },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reads"),
                Path::new("/tmp/input/sample.fq.gz").to_path_buf(),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::from_static("report"),
                Path::new("/tmp/out/report.json").to_path_buf(),
                ArtifactRole::ReportJson,
            )],
        },
        out_dir: Path::new("/tmp/out").to_path_buf(),
        aux_images: std::collections::BTreeMap::default(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };

    let args = build_apptainer_exec_args(
        &step,
        &[PathBuf::from("/tmp/input/sample.fq.gz")],
        Path::new("/tmp/input"),
        Path::new("/tmp/out"),
        RuntimeKind::Apptainer,
    )?;

    let pwd_index = args
        .iter()
        .position(|value| value == "--pwd")
        .ok_or_else(|| anyhow!("missing --pwd flag in apptainer args"))?;
    assert_eq!(args[pwd_index + 1], "/data/output");
    Ok(())
}

#[test]
fn container_command_template_rewrites_mounted_input_and_output_paths() {
    let template = vec![
        "sh".to_string(),
        "-lc".to_string(),
        "validator /tmp/corpus/sample_0004_R1.fq.gz > /tmp/out/validation_r1.log && printf '%s' /tmp/out/validation.json"
            .to_string(),
    ];

    let rewritten = container_command_template(
        &template,
        Path::new("/tmp/corpus"),
        Path::new("/tmp/out"),
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
        "flash2 -o flash2 -d /tmp/out -t 1 /tmp/corpus/sample_0004_R1.fq.gz /tmp/corpus/sample_0004_R2.fq.gz"
            .to_string(),
    ];

    let rewritten = container_command_template(
        &template,
        Path::new("/tmp/corpus"),
        Path::new("/tmp/out"),
        false,
    );

    assert_eq!(rewritten[0], "sh");
    assert!(rewritten[2].contains("-d /data/output"));
    assert!(rewritten[2].contains("/data/input/sample_0004_R1.fq.gz"));
    assert!(rewritten[2].contains("/data/input/sample_0004_R2.fq.gz"));
}

#[test]
fn container_command_template_keeps_output_paths_writable_when_out_dir_is_under_input_root() {
    let template = vec![
        "sh".to_string(),
        "-lc".to_string(),
        "printf '%s' /tmp/results/benchmark_corpus/reads.report_summary/cluster/bench/report_summary/sample_0001/tools/multiqc/report_summary.json > /tmp/results/benchmark_corpus/reads.report_summary/cluster/bench/report_summary/sample_0001/tools/multiqc/report_summary.json".to_string(),
    ];

    let rewritten = container_command_template(
        &template,
        Path::new("/tmp/results/benchmark_corpus"),
        Path::new(
            "/tmp/results/benchmark_corpus/reads.report_summary/cluster/bench/report_summary/sample_0001/tools/multiqc",
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
        "/tmp/out/bowtie2.metrics.txt".to_string(),
    ];

    let rewritten =
        container_command_template(&template, Path::new("/"), Path::new("/tmp/out"), true);

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
fn hash_inputs_ignores_missing_paths_and_hashes_directories() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let root = temp.path().join("summary_bundle");
    bijux_dna_infra::ensure_dir(&root)?;
    bijux_dna_infra::write_bytes(root.join("summary.txt"), b"bundle-summary")?;

    let hashes = hash_inputs(&[root, temp.path().join("missing.txt")])?;

    assert_eq!(hashes.len(), 1);
    assert_eq!(hashes[0].len(), 64);
    Ok(())
}
