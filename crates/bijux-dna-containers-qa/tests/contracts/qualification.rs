use std::fs;

use anyhow::Result;
use bijux_dna_containers_qa::api::{
    evaluate_container_qualification, plan_container_qualification, ContainerPipelineContract,
    ContainerPipelineV2, QualificationFailureClass, QualificationProbeKind,
    QualificationProbeObservation,
};
use serde_json::json;

#[test]
fn plans_and_accepts_complete_pipeline_evidence() -> Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("qualification-assessment-pass");
    let result_root = temporary.path().join("result");
    let output_root = result_root.join("outputs");
    fs::create_dir_all(&output_root)?;
    fs::write(output_root.join("result.json"), b"{\"records\":2}\n")?;
    let plan = plan_container_qualification(current_pipeline(&result_root, false)?)?;

    assert_eq!(plan.probes.len(), 4);
    assert_eq!(plan.output_mount.to_string_lossy(), "/result");
    assert_eq!(plan.metrics.len(), 1);
    let assessment = evaluate_container_qualification(
        &plan,
        &passing_observations(),
        &result_root,
        &output_root,
    )?;

    assert!(assessment.passed());
    assert!(assessment.failures.is_empty());
    assert_eq!(assessment.outputs.len(), 1);
    assert_eq!(assessment.outputs[0].path, "outputs/result.json");
    assert_eq!(assessment.metrics[0].status, "passed");
    assert_eq!(assessment.metrics[0].observed, json!(2));
    Ok(())
}

#[test]
fn accepts_checksum_bound_directory_output_evidence() -> Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("qualification-directory-output");
    let result_root = temporary.path().join("result");
    let output_root = result_root.join("outputs");
    let evidence_dir = output_root.join("fastqc/sample_fastqc");
    let evidence_payload = b">>Basic Statistics\tpass\n";
    fs::create_dir_all(&evidence_dir)?;
    fs::write(output_root.join("result.json"), b"{\"records\":2}\n")?;
    fs::write(evidence_dir.join("fastqc_data.txt"), evidence_payload)?;
    let plan = plan_container_qualification(current_pipeline_with_required_files(
        &result_root,
        false,
        &["result.json", "fastqc"],
    )?)?;

    let assessment = evaluate_container_qualification(
        &plan,
        &passing_observations(),
        &result_root,
        &output_root,
    )?;

    assert!(assessment.passed());
    let evidence = assessment
        .outputs
        .iter()
        .find(|output| output.path == "outputs/fastqc")
        .ok_or_else(|| anyhow::anyhow!("directory output evidence is missing"))?;
    assert_eq!(evidence.size_bytes, evidence_payload.len() as u64);
    assert_eq!(evidence.sha256, bijux_dna_infra::hash_path_sha256(&output_root.join("fastqc"))?);
    Ok(())
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_nested_in_directory_output_evidence() -> Result<()> {
    use std::os::unix::fs::symlink;

    let temporary = bijux_dna_testkit::tempdir_for("qualification-directory-output-symlink");
    let result_root = temporary.path().join("result");
    let output_root = result_root.join("outputs");
    let evidence_dir = output_root.join("fastqc/sample_fastqc");
    fs::create_dir_all(&evidence_dir)?;
    fs::write(output_root.join("result.json"), b"{\"records\":2}\n")?;
    fs::write(evidence_dir.join("fastqc_data.txt"), b">>Basic Statistics\tpass\n")?;
    let outside = temporary.path().join("outside.txt");
    fs::write(&outside, b"outside\n")?;
    symlink(&outside, evidence_dir.join("escaped.txt"))?;
    let plan = plan_container_qualification(current_pipeline_with_required_files(
        &result_root,
        false,
        &["result.json", "fastqc"],
    )?)?;

    let assessment = evaluate_container_qualification(
        &plan,
        &passing_observations(),
        &result_root,
        &output_root,
    )?;

    assert!(!assessment.passed());
    assert!(assessment.outputs.iter().all(|output| output.path != "outputs/fastqc"));
    assert!(assessment.failures.iter().any(|failure| {
        failure.class == QualificationFailureClass::RequiredOutput
            && failure.detail.contains("contains a symlink")
    }));
    Ok(())
}

#[test]
fn classifies_probe_output_and_metric_failures() -> Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("qualification-assessment-failures");
    let result_root = temporary.path().join("result");
    let output_root = result_root.join("outputs");
    fs::create_dir_all(&output_root)?;
    let plan = plan_container_qualification(current_pipeline(&result_root, false)?)?;
    let mut observations = passing_observations();
    observations[0].stdout = "unexpected".to_string();
    observations[3].exit_code = 0;

    let assessment =
        evaluate_container_qualification(&plan, &observations, &result_root, &output_root)?;
    let classes = assessment.failures.iter().map(|failure| failure.class).collect::<Vec<_>>();

    assert!(!assessment.passed());
    assert!(classes.contains(&QualificationFailureClass::VersionMismatch));
    assert!(classes.contains(&QualificationFailureClass::NegativeExit));
    assert!(classes.contains(&QualificationFailureClass::RequiredOutput));
    assert!(classes.contains(&QualificationFailureClass::MetricUnreadable));
    Ok(())
}

#[test]
fn qualification_planning_rejects_benchmark_measurement_authority() -> Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("qualification-plan-measurement");
    let result_root = temporary.path().join("result");
    let Err(error) = plan_container_qualification(current_pipeline(&result_root, true)?) else {
        anyhow::bail!("benchmark measurement entered qualification planning");
    };
    assert!(error.to_string().contains("benchmark measurement authority"));
    Ok(())
}

fn current_pipeline(
    result_root: &std::path::Path,
    measurement: bool,
) -> Result<ContainerPipelineContract> {
    current_pipeline_with_required_files(result_root, measurement, &["result.json"])
}

fn current_pipeline_with_required_files(
    result_root: &std::path::Path,
    measurement: bool,
    required_files: &[&str],
) -> Result<ContainerPipelineContract> {
    let measurement = measurement.then(|| {
        json!({
            "schema_version": "bijux.benchmark.measurement.v1",
            "warmup_repetitions": 1,
            "measured_repetitions": 3,
            "resource_file_template": "repetitions/repetition-{repetition}.resource.tsv",
            "runtime_scope": "containerized_stage_end_to_end"
        })
    });
    let pipeline: ContainerPipelineV2 = serde_json::from_value(json!({
        "schema_version": "bijux.container.pipeline.v2",
        "run": {
            "id": "run-1200",
            "kind": "qualification",
            "campaign": "qa-boundary",
            "corpus": "immutable-corpus",
            "sample": "sample-a",
            "domain": "fastq",
            "stage": "fastq.align_reads",
            "tool": "bwa",
            "operator": "bijux",
            "submitted_at_utc": "2026-08-07T09:45:00Z"
        },
        "repository": {"path": "/repository", "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "clean": true},
        "image": {"path": "/images/bwa.sif", "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},
        "data": {"inputs": [{
            "id": "reads",
            "kind": "data",
            "path": "/data/reads.fastq.gz",
            "mount_path": "/input/reads.fastq.gz",
            "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        }]},
        "commands": {
            "version": ["bwa", "version"],
            "help": ["bwa", "help"],
            "real": ["bwa", "mem"],
            "negative": ["bwa", "invalid"]
        },
        "outputs": {
            "result_root": result_root,
            "mount_path": "/result",
            "required_files": required_files,
            "collection_rule": "required_files_and_metrics"
        },
        "resources": {
            "account": "science",
            "partition": "compute",
            "cpus": 2,
            "memory_mb": 1024,
            "walltime": "00:05:00",
            "scratch_mb": 1024,
            "retry": {"max_attempts": 1, "backoff_seconds": 0, "retryable_exit_codes": []}
        },
        "execution": {
            "locale": "C",
            "timezone": "UTC",
            "runtime_network": false,
            "threads": 2,
            "random_seed": null,
            "cache_policy": "cold",
            "determinism": "deterministic"
        },
        "dependencies": [],
        "acceptance": {
            "version_regex": "^1[.]0$",
            "negative_exit_nonzero": true,
            "metrics": [{
                "id": "records",
                "path": "result.json",
                "json_pointer": "/records",
                "operator": "ge",
                "expected": 1,
                "unit": "records"
            }]
        },
        "measurement": measurement
    }))?;
    Ok(ContainerPipelineContract::Current(Box::new(pipeline)))
}

fn passing_observations() -> Vec<QualificationProbeObservation> {
    vec![
        observation(QualificationProbeKind::Version, 0, "1.0"),
        observation(QualificationProbeKind::Help, 0, "help"),
        observation(QualificationProbeKind::Real, 0, "complete"),
        observation(QualificationProbeKind::Negative, 2, ""),
    ]
}

fn observation(
    kind: QualificationProbeKind,
    exit_code: i32,
    stdout: &str,
) -> QualificationProbeObservation {
    QualificationProbeObservation {
        kind,
        exit_code,
        stdout: stdout.to_string(),
        stderr: String::new(),
    }
}
