#![cfg(test)]

use super::identity::{
    execution_pipeline_identity, execution_sample_identity, hash_path,
    infer_tool_version_from_image, runtime_platform_identity,
};
use super::inputs::container_input_mapping;
use super::observer::build_observer_command_args;
use super::runtime_policy::stage_workdir_for_value;
use super::{build_apptainer_exec_args, container_command_template, execute_step, hash_inputs};
use anyhow::anyhow;
use bijux_dna_containers::api::RuntimeKind;
use bijux_dna_core::contract::ExecutionStep;
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
fn ordinary_runner_refuses_external_artifact_plans_before_execution() -> anyhow::Result<()> {
    let step = execution_step_fixture(
        "sample-0001.vcf.call-gl.angsd",
        "vcf.call_gl",
        &["angsd".to_string(), "-i".to_string(), "sample.input".to_string()],
        "external-artifact://angsd",
        Vec::new(),
        Vec::new(),
        "/artifacts/runtime/out",
    );

    let error = execute_step(&step, RuntimeKind::Local, None)
        .err()
        .ok_or_else(|| anyhow!("ordinary runner accepted restricted-tool execution"))?;

    assert!(error.to_string().contains("execute_external_artifact_step"));
    Ok(())
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
        None,
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

    assert_eq!(digest, bijux_dna_infra::hash_path_sha256(&root)?);
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

    let Err(err) = hash_inputs(&[missing]) else {
        panic!("missing input must fail");
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

    let expected = bijux_dna_infra::hash_path_sha256(&root)?;
    let hashes = hash_inputs(&[root])?;

    assert_eq!(hashes.len(), 1);
    assert_eq!(hashes[0], expected);
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

#[cfg(unix)]
#[test]
fn container_stage_deadlines_refuse_live_providers_and_inherited_pipes() -> anyhow::Result<()> {
    use super::runtime_policy::StepRuntimeContext;
    use crate::command_runner::{
        CommandExecutionBindingV1, ExecutionBindingManifestV1, RUNNER_EXECUTION_BINDINGS_ENV,
        RUNNER_EXECUTION_BINDINGS_SCHEMA_V1,
    };
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::time::Duration;

    let root = bijux_dna_testkit::tempdir_for("container-stage-deadline");
    let provider = root.path().join("runtime-provider");
    let binding_path = root.path().join("provider-bindings.json");
    let provider_name = provider.to_str().ok_or_else(|| anyhow!("provider is not UTF-8"))?;
    // Invoke the existing shell directly; a newly written executable's admission is
    // unrelated to the runner's one-second lifetime and inherited-pipe contract.
    let bindings = ExecutionBindingManifestV1 {
        schema_version: RUNNER_EXECUTION_BINDINGS_SCHEMA_V1.to_string(),
        bindings: vec![CommandExecutionBindingV1 {
            command: provider_name.to_string(),
            program: "/bin/sh".to_string(),
            prefix_args: vec![
                "-c".to_string(),
                "for last do :; done\nprintf provider-ready >&2\ncase \"$last\" in hold) exec /bin/sleep 2;; inherited) /bin/sleep 2 & exit 0;; *) printf completed; exit 0;; esac\n".to_string(),
                provider_name.to_string(),
            ],
            required_files: Vec::new(),
        }],
    };
    std::fs::write(&binding_path, serde_json::to_vec(&bindings)?)?;
    let environment = BTreeMap::from([(
        OsString::from(RUNNER_EXECUTION_BINDINGS_ENV),
        binding_path.into_os_string(),
    )]);
    for runtime in [RuntimeKind::Apptainer, RuntimeKind::Singularity] {
        for mode in ["hold", "inherited"] {
            let out_dir = root.path().join(format!("{runtime:?}-{mode}"));
            let step = execution_step_fixture(
                "deadline-probe",
                "fastq.trim_reads",
                &[mode.to_string()],
                "private-provider-fixture.sif",
                Vec::new(),
                Vec::new(),
                out_dir.to_str().ok_or_else(|| anyhow!("fixture root is not UTF-8"))?,
            );
            // Keep the deadline below the finite provider lifetime while allowing process startup.
            let error = super::execute_verified_step_in_context(
                &step,
                runtime,
                StepRuntimeContext {
                    timeout: Some(Duration::from_secs(1)),
                    environment: Some(&environment),
                    executable: Some(&provider),
                },
            )
            .err()
            .ok_or_else(|| anyhow!("stage deadline accepted {runtime:?} {mode}"))?;
            let message = format!("{error:#}");
            assert!(message.contains("[runner_effect:container_lifecycle]"), "{message}");
            assert!(message.contains("provider-ready"), "{message}");
            assert!(message.contains("local command exceeded timeout"), "{message}");
            assert!(!out_dir.join("run_artifacts").exists());
        }
        let out_dir = root.path().join(format!("{runtime:?}-completed"));
        let step = execution_step_fixture(
            "completed-probe",
            "fastq.trim_reads",
            &["complete".to_string()],
            "private-provider-fixture.sif",
            Vec::new(),
            Vec::new(),
            out_dir.to_str().ok_or_else(|| anyhow!("fixture root is not UTF-8"))?,
        );
        for timeout in [None, Some(Duration::from_secs(5))] {
            let completed = super::execute_verified_step_in_context(
                &step,
                runtime,
                StepRuntimeContext {
                    timeout,
                    environment: Some(&environment),
                    executable: Some(&provider),
                },
            )?;
            assert!(completed.command.starts_with("/bin/sh -c "));
            assert_eq!(completed.exit_code, 0);
            assert_eq!(completed.stdout, "completed");
            assert_eq!(completed.stderr, "provider-ready");
            assert!(completed
                .command
                .contains(provider.to_str().ok_or_else(|| anyhow!("provider is not UTF-8"))?));
        }
    }
    Ok(())
}

#[test]
fn owned_environment_drives_container_policy_without_global_overrides() -> anyhow::Result<()> {
    use super::runtime_policy::{
        configured_memory_mb, network_allowed_with_context, runtime_env_exports,
    };
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    let mut environment = BTreeMap::from([
        ("BIJUX_STAGE_WORKDIR".to_string(), "/artifacts/runtime/out/private/work".to_string()),
        ("BIJUX_STAGE_MEMORY_MB".to_string(), "1536".to_string()),
        ("BIJUX_ALLOW_NETWORK".to_string(), "0".to_string()),
        ("LC_ALL".to_string(), "C".to_string()),
    ])
    .into_iter()
    .map(|(key, value)| (OsString::from(key), OsString::from(value)))
    .collect::<BTreeMap<_, _>>();
    let step = execution_step_fixture(
        "context",
        "vcf.qc",
        &["true".to_string()],
        "context.sif",
        Vec::new(),
        Vec::new(),
        "/artifacts/runtime/out",
    );
    let args = build_apptainer_exec_args(
        &step,
        &[],
        Path::new("/input"),
        Path::new("/artifacts/runtime/out"),
        RuntimeKind::Apptainer,
        Some(&environment),
    )?;
    assert!(args.windows(2).any(|values| values == ["--pwd", "/data/output/private/work"]));
    assert!(
        runtime_env_exports(Some(&environment)).contains(&("LC_ALL".to_string(), "C".to_string()))
    );
    assert!(!network_allowed_with_context(Some(&environment)));
    assert!((configured_memory_mb(&step, Some(&environment)) - 1536.0).abs() < f64::EPSILON);
    for value in ["NaN", "inf", "-1", "0", "invalid"] {
        environment.insert(OsString::from("BIJUX_STAGE_MEMORY_MB"), OsString::from(value));
        assert!((configured_memory_mb(&step, Some(&environment)) - 1024.0).abs() < f64::EPSILON);
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn local_context_reaches_child_and_provenance_without_binding_resolution() -> anyhow::Result<()> {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    let root = bijux_dna_testkit::tempdir_for("local-owned-environment");
    let temporary = root.path().join("owned-temp");
    let environment = BTreeMap::from([
        ("TMPDIR".to_string(), temporary.display().to_string()),
        ("BIJUX_STAGE_THREADS".to_string(), "3".to_string()),
        (
            "BIJUX_DNA_RUNNER_BINDINGS".to_string(),
            root.path().join("absent-bindings.json").display().to_string(),
        ),
    ])
    .into_iter()
    .map(|(key, value)| (OsString::from(key), OsString::from(value)))
    .collect::<BTreeMap<_, _>>();
    let step = execution_step_fixture(
        "context",
        "vcf.qc",
        &[
            "sh".to_string(),
            "-c".to_string(),
            "printf '%s|%s' \"$TMPDIR\" \"$BIJUX_STAGE_THREADS\"".to_string(),
        ],
        "bijux/context-provider:1",
        Vec::new(),
        Vec::new(),
        root.path().to_str().ok_or_else(|| anyhow!("fixture root is not UTF-8"))?,
    );
    let result =
        super::execute_step_with_context(&step, RuntimeKind::Local, None, Some(&environment))?;
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, format!("{}|3", temporary.display()));
    let invocation: serde_json::Value = serde_json::from_slice(&std::fs::read(
        root.path().join("run_artifacts/tool_invocation.json"),
    )?)?;
    assert_eq!(invocation["environment"]["TMPDIR"], temporary.display().to_string());
    Ok(())
}

#[cfg(unix)]
#[test]
fn docker_lifecycle_uses_owned_host_environment_without_new_binding_admission() -> anyhow::Result<()>
{
    use super::super::backend::docker::executor::{
        docker_logs_with_context, docker_wait_timeout_with_context, docker_wait_with_context,
    };
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::os::unix::fs::PermissionsExt as _;
    let root = bijux_dna_testkit::tempdir_for("docker-lifecycle-environment");
    let executable = root.path().join("docker");
    std::fs::write(&executable, b"#!/bin/sh\ncase \"$1\" in wait) printf '0\\n';; logs) printf '%s' \"$BIJUX_STAGE_THREADS\";; *) exit 64;; esac\n")?;
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700))?;
    let environment = BTreeMap::from([
        ("PATH".to_string(), root.path().display().to_string()),
        ("BIJUX_STAGE_THREADS".to_string(), "5".to_string()),
        (
            "BIJUX_DNA_RUNNER_BINDINGS".to_string(),
            root.path().join("absent-bindings.json").display().to_string(),
        ),
    ])
    .into_iter()
    .map(|(key, value)| (OsString::from(key), OsString::from(value)))
    .collect::<BTreeMap<_, _>>();
    assert_eq!(docker_wait_with_context("owned", Some(&environment))?, 0);
    assert_eq!(
        docker_wait_timeout_with_context(
            "owned",
            std::time::Duration::from_secs(2),
            Some(&environment)
        )?,
        0
    );
    assert_eq!(docker_logs_with_context("owned", Some(&environment))?, "5");
    Ok(())
}

#[cfg(unix)]
#[test]
fn native_environment_bytes_reach_local_children_without_text_fallback() -> anyhow::Result<()> {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt as _;
    let root = bijux_dna_testkit::tempdir_for("local-native-environment");
    let observation = root.path().join("native-env.bytes");
    let raw_home = OsString::from_vec(b"owned-home-\xff".to_vec());
    let raw_temporary = OsString::from_vec(b"owned-temporary-\xfe".to_vec());
    let environment = BTreeMap::from([
        (OsString::from("HOME"), raw_home.clone()),
        (OsString::from("TMPDIR"), raw_temporary.clone()),
    ]);
    let step = execution_step_fixture(
        "native-context",
        "vcf.qc",
        &[
            "sh".to_string(),
            "-c".to_string(),
            "printf '%s\\000%s' \"$HOME\" \"$TMPDIR\" > \"$1\"".to_string(),
            "bijux-native-env".to_string(),
            observation.display().to_string(),
        ],
        "bijux/context-provider:1",
        Vec::new(),
        Vec::new(),
        root.path().to_str().ok_or_else(|| anyhow!("ASCII fixture root required"))?,
    );
    let result =
        super::execute_step_with_context(&step, RuntimeKind::Local, None, Some(&environment))?;
    assert_eq!(result.exit_code, 0);
    let mut expected = raw_home.into_vec();
    expected.push(0);
    expected.extend(raw_temporary.into_vec());
    assert_eq!(std::fs::read(observation)?, expected);
    let invocation: serde_json::Value = serde_json::from_slice(&std::fs::read(
        root.path().join("run_artifacts/tool_invocation.json"),
    )?)?;
    assert!(invocation["environment"].get("TMPDIR").is_none());
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn inherited_local_entrypoint_keeps_native_output_directory_bytes() -> anyhow::Result<()> {
    use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
    let root = bijux_dna_testkit::tempdir_for("native-local-output");
    let output = root.path().join(std::ffi::OsString::from_vec(b"owned-out-\xff".to_vec()));
    std::fs::create_dir(&output)?;
    let observation = root.path().join("working-directory.bytes");
    let mut step = execution_step_fixture(
        "native-output",
        "vcf.qc",
        &[
            "sh".to_string(),
            "-c".to_string(),
            "pwd -P > \"$1\"".to_string(),
            "bijux-native-output".to_string(),
            observation.display().to_string(),
        ],
        "bijux/context-provider:1",
        Vec::new(),
        Vec::new(),
        root.path().to_str().ok_or_else(|| anyhow!("ASCII fixture parent required"))?,
    );
    step.out_dir.clone_from(&output);
    let result = execute_step(&step, RuntimeKind::Local, None)?;
    assert_eq!(result.exit_code, 0);
    let mut expected = output.canonicalize()?.as_os_str().as_bytes().to_vec();
    expected.push(b'\n');
    assert_eq!(std::fs::read(observation)?, expected);
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("run_artifacts/stage_report.json"))?)?;
    assert_eq!(
        report["log_paths"],
        serde_json::json!([
            output.join("run_artifacts/stdout.log").display().to_string(),
            output.join("run_artifacts/stderr.log").display().to_string(),
        ])
    );
    Ok(())
}
