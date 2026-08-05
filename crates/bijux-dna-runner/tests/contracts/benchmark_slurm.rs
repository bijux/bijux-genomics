use std::collections::BTreeMap;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use std::process::Command;

use bijux_dna_bench_model::{
    BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator,
    BenchmarkAttemptIdentity, BenchmarkExecutionAcceptance, BenchmarkExecutionCachePolicy,
    BenchmarkExecutionCommands, BenchmarkExecutionDeterminism, BenchmarkExecutionDownstreamProbe,
    BenchmarkExecutionImage, BenchmarkExecutionInput, BenchmarkExecutionInputKind,
    BenchmarkExecutionInterface, BenchmarkExecutionMeasurement, BenchmarkExecutionOutputs,
    BenchmarkExecutionParameterEvidence, BenchmarkExecutionResources,
    BenchmarkExecutionRetryPolicy, BenchmarkExecutionRuntimePolicy, BenchmarkExecutionSourceLock,
    BenchmarkExecutionSpec, BenchmarkRunIdentity, BENCHMARK_EXECUTION_SPEC_SCHEMA_V1,
};
use bijux_dna_runner::api::{
    load_benchmark_slurm_job_handle, prepare_benchmark_slurm_job, submit_prepared_benchmark_job,
    SlurmClientPaths,
};

struct Fixture {
    root: tempfile::TempDir,
    spec: BenchmarkExecutionSpec,
    input_path: PathBuf,
}

#[allow(clippy::too_many_lines)]
fn fixture() -> anyhow::Result<Fixture> {
    let root = tempfile::tempdir()?;
    let canonical_root = root.path().canonicalize()?;
    let repository_path = canonical_root.join("source");
    let image_path = canonical_root.join("seqkit.sif");
    let input_path = canonical_root.join("ERR1.fastq.gz");
    let result_root = canonical_root.join(
        "benchmarks/run-2042-fastq-profile-read-lengths/seqkit/attempts/human-short-read-length-comparison/attempt-001",
    );
    std::fs::create_dir(&repository_path)?;
    std::fs::create_dir_all(&result_root)?;
    std::fs::write(&image_path, b"seqkit image")?;
    std::fs::write(&input_path, b"fastq input")?;

    let spec = BenchmarkExecutionSpec {
        schema_version: BENCHMARK_EXECUTION_SPEC_SCHEMA_V1.to_string(),
        identity: BenchmarkAttemptIdentity {
            campaign_id: "human-short-read-length-comparison".to_string(),
            run: BenchmarkRunIdentity {
                run_id: "run-2042".to_string(),
                domain: "fastq".to_string(),
                stage_id: "fastq.profile_read_lengths".to_string(),
            },
            attempt_id: "attempt-001".to_string(),
            tool_id: "seqkit".to_string(),
        },
        operator: "bijan".to_string(),
        prepared_at_utc: "2026-08-05T02:00:00Z".to_string(),
        source: BenchmarkExecutionSourceLock {
            repository_commit: "1".repeat(40),
            repository_path: repository_path.display().to_string(),
            contract_sha256: BTreeMap::from([
                ("campaign_registry".to_string(), "2".repeat(64)),
                ("container_catalog".to_string(), "3".repeat(64)),
                ("image_inventory".to_string(), "4".repeat(64)),
                ("stage_interface_registry".to_string(), "5".repeat(64)),
            ]),
        },
        image: BenchmarkExecutionImage {
            path: image_path.display().to_string(),
            sha256: bijux_dna_infra::hash_file_sha256(&image_path)?,
        },
        inputs: vec![BenchmarkExecutionInput {
            id: "fastq_err1".to_string(),
            kind: BenchmarkExecutionInputKind::Data,
            path: input_path.display().to_string(),
            mount_path: "/input/ERR1.fastq.gz".to_string(),
            sha256: bijux_dna_infra::hash_file_sha256(&input_path)?,
        }],
        commands: BenchmarkExecutionCommands {
            version: vec!["seqkit".to_string(), "version".to_string()],
            help: vec!["seqkit".to_string(), "--help".to_string()],
            warmup: vec!["sh".to_string(), "-c".to_string(), "true".to_string()],
            measured: vec![
                "sh".to_string(),
                "-c".to_string(),
                "printf '%s' \"measured\" > /result/repetitions/repetition-{repetition}.metrics.json"
                    .to_string(),
            ],
            negative: vec!["seqkit".to_string(), "missing.fastq".to_string()],
        },
        parameters: BenchmarkExecutionParameterEvidence {
            parameter_schema_version: "bijux.fastq.params.read_length_profile.v1".to_string(),
            defaults_provider: "bijux_dna_domain_fastq::read_length_profile_defaults".to_string(),
            overrides_path: "config-pipeline-parameters.yml".to_string(),
            overrides_sha256: "6".repeat(64),
            effective_path: "effective-parameters.json".to_string(),
            effective_sha256: "7".repeat(64),
        },
        interface: BenchmarkExecutionInterface {
            accepted_input_contracts: vec!["bijux.fastq.artifact.single_end.v1".to_string()],
            output_contract: "bijux.benchmark.fastq.profile_read_lengths.bundle.v1".to_string(),
            normalized_result_contract: "benchmarks/contracts/schemas/result.json".to_string(),
            adapter: "bijux_dna_planner_fastq::profile_read_lengths".to_string(),
            downstream_probe: BenchmarkExecutionDownstreamProbe {
                stage_id: "benchmark.fastq.profile_read_lengths.compare".to_string(),
                input_contract: "bijux.benchmark.fastq.profile_read_lengths.bundle.v1".to_string(),
                source: "benchmarks/execution/fastq_read_lengths.awk".to_string(),
                artifact: "repetitions/repetition-1.metrics.json".to_string(),
            },
        },
        measurement: BenchmarkExecutionMeasurement {
            warmup_repetitions: 1,
            measured_repetitions: 3,
            resource_file_template: "repetitions/repetition-{repetition}.resource.tsv".to_string(),
            runtime_scope: "containerized_stage_end_to_end".to_string(),
        },
        outputs: BenchmarkExecutionOutputs {
            result_root: result_root.display().to_string(),
            mount_path: "/result".to_string(),
            required_files: vec![
                "repetitions/repetition-1.metrics.json".to_string(),
                "repetitions/repetition-1.resource.tsv".to_string(),
                "repetitions/repetition-2.resource.tsv".to_string(),
                "repetitions/repetition-3.resource.tsv".to_string(),
            ],
        },
        resources: BenchmarkExecutionResources {
            account: "lu2024-12-24".to_string(),
            partition: "compute".to_string(),
            cpus: 4,
            memory_mb: 8192,
            walltime: "01:00:00".to_string(),
            scratch_mb: 4096,
            retry: BenchmarkExecutionRetryPolicy {
                max_attempts: 1,
                backoff_seconds: 0,
                retryable_exit_codes: Vec::new(),
            },
        },
        runtime: BenchmarkExecutionRuntimePolicy {
            locale: "C".to_string(),
            timezone: "UTC".to_string(),
            runtime_network: false,
            threads: 4,
            random_seed: None,
            cache_policy: BenchmarkExecutionCachePolicy::Warm,
            determinism: BenchmarkExecutionDeterminism::Deterministic,
        },
        acceptance: BenchmarkExecutionAcceptance {
            version_regex: "^[0-9]+[.][0-9]+".to_string(),
            negative_exit_nonzero: true,
            metrics: vec![BenchmarkAcceptanceMetric {
                id: "read_count".to_string(),
                path: "repetitions/repetition-1.metrics.json".to_string(),
                json_pointer: "/read_count".to_string(),
                operator: BenchmarkAcceptanceOperator::GreaterThan,
                expected: BenchmarkAcceptanceExpectedValue::Integer(0),
                unit: "reads".to_string(),
            }],
        },
    };
    std::fs::write(result_root.join("execution-spec.json"), serde_json::to_vec_pretty(&spec)?)?;
    Ok(Fixture { root, spec, input_path })
}

#[test]
fn benchmark_slurm_material_is_isolated_complete_and_restart_safe() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let first = prepare_benchmark_slurm_job(&fixture.spec, "2026-08-05T02:10:00Z")?;
    let recovered = prepare_benchmark_slurm_job(&fixture.spec, "2026-08-05T02:10:00Z")?;
    assert_eq!(first, recovered);

    let script = std::fs::read_to_string(&first.request.script_path)?;
    assert!(script.contains("#SBATCH --account=lu2024-12-24"));
    assert!(script.contains("--containall' '--cleanenv' '--net' '--network' 'none'"));
    assert!(script.contains(":/repository:ro"));
    assert!(script.contains(":/input/ERR1.fastq.gz:ro"));
    assert!(script.contains("repetition-3.resource.tsv.partial"));
    assert!(script.contains(
        "'/usr/bin/time' '-o' '/result/repetitions/repetition-1.resource.tsv.partial' '-f'"
    ));
    assert!(!script.contains("'--quiet'"));
    assert!(!script.contains("'--output="));
    assert!(!script.contains("'--format="));
    assert!(!script.lines().any(|line| line.starts_with("/usr/bin/time ")));
    assert!(!script.contains("{repetition}"));
    assert!(script.contains("output-sha256.tsv"));
    assert!(script.contains("completed.json.partial"));
    assert!(Command::new("bash").arg("-n").arg(&first.request.script_path).status()?.success());
    Ok(())
}

#[test]
fn benchmark_slurm_material_rejects_authority_drift_and_script_conflicts() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let prepared = prepare_benchmark_slurm_job(&fixture.spec, "2026-08-05T02:10:00Z")?;
    std::fs::write(&fixture.input_path, b"mutated")?;
    assert!(prepare_benchmark_slurm_job(&fixture.spec, "2026-08-05T02:10:00Z")
        .is_err_and(|error| error.to_string().contains("input digest differs")));

    std::fs::write(&fixture.input_path, b"fastq input")?;
    std::fs::write(&prepared.request.script_path, b"conflicting script")?;
    assert!(prepare_benchmark_slurm_job(&fixture.spec, "2026-08-05T02:10:00Z")
        .is_err_and(|error| error.to_string().contains("script conflicts")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn benchmark_slurm_material_rejects_output_symlinks_and_directive_injection() -> anyhow::Result<()>
{
    let mut injected = fixture()?;
    injected.spec.resources.partition = "compute\n#SBATCH --exclusive".to_string();
    assert!(prepare_benchmark_slurm_job(&injected.spec, "2026-08-05T02:10:00Z")
        .is_err_and(|error| error.to_string().contains("unsupported characters")));

    let symlinked = fixture()?;
    let result_root = PathBuf::from(&symlinked.spec.outputs.result_root);
    let external =
        result_root.parent().ok_or_else(|| anyhow::anyhow!("result parent"))?.join("external");
    std::fs::create_dir(&external)?;
    std::os::unix::fs::symlink(&external, result_root.join("repetitions"))?;
    assert!(prepare_benchmark_slurm_job(&symlinked.spec, "2026-08-05T02:10:00Z")
        .is_err_and(|error| error.to_string().contains("non-symlink directory")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn benchmark_submission_persists_and_recovers_one_scheduler_effect() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let prepared = prepare_benchmark_slurm_job(&fixture.spec, "2026-08-05T02:10:00Z")?;
    let calls = fixture.root.path().join("sbatch-calls");
    let paths = scheduler_client(
        fixture.root.path(),
        &format!("printf x >> {}\nprintf '456789\\n'\n", shell_literal(&calls)),
    )?;

    let first = submit_prepared_benchmark_job(&paths, &prepared)?;
    let recovered = submit_prepared_benchmark_job(&paths, &prepared)?;
    std::fs::write(
        PathBuf::from(&fixture.spec.outputs.result_root)
            .join("repetitions/repetition-1.metrics.json"),
        b"{}",
    )?;
    let loaded = load_benchmark_slurm_job_handle(&fixture.spec)?;

    assert_eq!(first, recovered);
    assert_eq!(first, loaded);
    assert_eq!(std::fs::read_to_string(calls)?, "x");
    let scheduler_root =
        prepared.request.script_path.parent().ok_or_else(|| anyhow::anyhow!("script parent"))?;
    assert!(scheduler_root.join("submission-intent.json").is_file());
    assert!(scheduler_root.join("job-handle.json").is_file());
    Ok(())
}

#[cfg(unix)]
#[test]
fn unresolved_submission_intent_blocks_duplicate_scheduler_effects() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let prepared = prepare_benchmark_slurm_job(&fixture.spec, "2026-08-05T02:10:00Z")?;
    let paths = scheduler_client(fixture.root.path(), "echo rejected >&2\nexit 1\n")?;
    assert!(submit_prepared_benchmark_job(&paths, &prepared).is_err());

    write_executable(&paths.sbatch, "printf '999999\\n'\n")?;
    assert!(submit_prepared_benchmark_job(&paths, &prepared)
        .is_err_and(|error| error.to_string().contains("reconcile scheduler state")));
    let scheduler_root =
        prepared.request.script_path.parent().ok_or_else(|| anyhow::anyhow!("script parent"))?;
    assert!(scheduler_root.join("submission-failure.json").is_file());
    assert!(!scheduler_root.join("job-handle.json").exists());
    Ok(())
}

#[cfg(unix)]
fn scheduler_client(root: &std::path::Path, sbatch_body: &str) -> anyhow::Result<SlurmClientPaths> {
    let paths = SlurmClientPaths {
        sbatch: root.join("sbatch"),
        squeue: root.join("squeue"),
        sacct: root.join("sacct"),
        scancel: root.join("scancel"),
    };
    write_executable(&paths.sbatch, sbatch_body)?;
    Ok(paths)
}

#[cfg(unix)]
fn write_executable(path: &std::path::Path, body: &str) -> anyhow::Result<()> {
    std::fs::write(path, format!("#!/bin/sh\n{body}"))?;
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

fn shell_literal(path: &std::path::Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}
