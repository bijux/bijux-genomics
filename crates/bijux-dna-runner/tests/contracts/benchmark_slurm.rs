use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
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
    BenchmarkExecutionSpec, BenchmarkRunIdentity, BenchmarkStagePlanManifest,
    BenchmarkStagePlanReference, BENCHMARK_EXECUTION_SPEC_SCHEMA_V1,
    BENCHMARK_STAGE_PLAN_MANIFEST_SCHEMA_V1,
};
use bijux_dna_containers::api::{
    freeze_repository, ContainerGovernance, ContainerPathPolicy, ContainerPreflightPolicy,
    ContainerRuntimePolicy, ContainerSitePolicy, ContainerSlurmPolicy, RepositoryFreezeRequest,
    RepositoryFreezeScope, CONTAINER_GOVERNANCE_SCHEMA_V1,
};
use bijux_dna_runner::api::{
    load_benchmark_slurm_job_handle, prepare_benchmark_slurm_job, submit_prepared_benchmark_job,
    SlurmClientPaths,
};

#[cfg(unix)]
use crate::scheduler_executable::write_executable;

struct Fixture {
    root: tempfile::TempDir,
    spec: BenchmarkExecutionSpec,
    input_path: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for entry in walkdir::WalkDir::new(self.root.path())
            .contents_first(true)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_symlink() {
                continue;
            }
            if let Ok(metadata) = entry.metadata() {
                let mut permissions = metadata.permissions();
                permissions.set_mode(permissions.mode() | 0o200);
                let _ = std::fs::set_permissions(entry.path(), permissions);
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
fn fixture() -> anyhow::Result<Fixture> {
    let root = tempfile::tempdir()?;
    let canonical_root = root.path().canonicalize()?;
    let active_repository = canonical_root.join("active-source");
    std::fs::create_dir(&active_repository)?;
    run_git(&active_repository, &["init", "--quiet"])?;
    run_git(&active_repository, &["config", "user.name", "Bijux Test"])?;
    run_git(&active_repository, &["config", "user.email", "test@bijux.invalid"])?;
    std::fs::write(active_repository.join("README.md"), b"benchmark source\n")?;
    run_git(&active_repository, &["add", "README.md"])?;
    run_git(&active_repository, &["commit", "--quiet", "-m", "fixture"])?;
    let repository_commit = run_git(&active_repository, &["rev-parse", "HEAD"])?;
    let study_root = canonical_root.join("study");
    let governance = ContainerGovernance {
        schema_version: CONTAINER_GOVERNANCE_SCHEMA_V1.to_string(),
        site: ContainerSitePolicy {
            id: "test-site".to_string(),
            hostname_regex: "^test-host$".to_string(),
        },
        paths: ContainerPathPolicy {
            study_root: study_root.clone(),
            containers_root: study_root.join("containers"),
            data_root: study_root.join("data"),
            results_root: study_root.join("results"),
            finalization_root: study_root.join("evidence/finalization"),
            active_repository,
            frozen_repositories: study_root.join("data/evidence/source-checkouts"),
        },
        runtime: ContainerRuntimePolicy { apptainer_architecture: "linux/amd64".to_string() },
        slurm: ContainerSlurmPolicy {
            account: "science".to_string(),
            scratch_environment: Some("SNIC_TMP".to_string()),
            scratch_root: None,
            partitions: vec!["compute".to_string()],
            quota_command: vec!["projinfo".to_string()],
        },
        preflight: ContainerPreflightPolicy {
            minimum_available_bytes: 1,
            minimum_available_inodes: 1,
            maximum_clock_skew_seconds: 30,
            required_commands: vec!["git".to_string()],
            required_contracts: vec!["containers/contracts/governance.yml".to_string()],
        },
    };
    let repository_path = freeze_repository(&RepositoryFreezeRequest {
        governance,
        hostname: "test-host".to_string(),
        commit: repository_commit.clone(),
        scope: RepositoryFreezeScope::Complete,
        checkout_paths: Vec::new(),
        source_namespace: "source-freezes".to_string(),
    })?;
    let image_path = canonical_root.join("seqkit.sif");
    let input_path = canonical_root.join("ERR1.fastq.gz");
    let result_root = canonical_root.join(
        "benchmarks/run-2042-fastq-profile-read-lengths/seqkit/attempts/human-short-read-length-comparison/attempt-001",
    );
    std::fs::create_dir_all(&result_root)?;
    std::fs::write(&image_path, b"seqkit image")?;
    std::fs::write(&input_path, b"fastq input")?;
    let mut required_files = vec![
        "repetitions/repetition-1.metrics.json".to_string(),
        "repetitions/repetition-1.resource.tsv".to_string(),
        "repetitions/repetition-2.resource.tsv".to_string(),
        "repetitions/repetition-3.resource.tsv".to_string(),
    ];
    for repetition in 1..=3 {
        let instance = format!("measured-repetition-{repetition}--sample-a");
        required_files.extend([
            format!("repetitions/{instance}/raw.tsv"),
            format!("repetitions/{instance}/resource.tsv"),
        ]);
    }

    let spec = BenchmarkExecutionSpec {
        schema_version: BENCHMARK_EXECUTION_SPEC_SCHEMA_V1.to_string(),
        identity: BenchmarkAttemptIdentity {
            campaign_id: "human-short-read-length-comparison".to_string(),
            run: BenchmarkRunIdentity {
                run_id: "run-2042".to_string(),
                domain: "fastq".to_string(),
                stage_id: "fastq.profile_read_lengths".to_string(),
            },
            case_id: None,
            attempt_id: "attempt-001".to_string(),
            tool_id: "seqkit".to_string(),
        },
        operator: "bijan".to_string(),
        prepared_at_utc: "2026-08-05T02:00:00Z".to_string(),
        source: BenchmarkExecutionSourceLock {
            repository_commit,
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
            parameter_schema_version: "test.parameter.schema.v1".to_string(),
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
            required_files,
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
    let execution_spec_path = result_root.join("execution-spec.json");
    std::fs::write(&execution_spec_path, serde_json::to_vec_pretty(&spec)?)?;
    let mut plans = Vec::new();
    for repetition in 1..=3 {
        let instance = format!("measured-repetition-{repetition}--sample-a");
        let relative = format!("stage-plans/repetition-{repetition}/sample-a.json");
        let path = result_root.join(&relative);
        std::fs::create_dir_all(path.parent().ok_or_else(|| anyhow::anyhow!("plan parent"))?)?;
        std::fs::write(
            &path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "stage_instance_id": instance,
                "command": {
                    "template": [
                        "sh",
                        "-c",
                        format!(
                            "printf '%s\\n' raw > /result/repetitions/{instance}/raw.tsv"
                        )
                    ]
                },
                "io": {
                    "outputs": [{
                        "path": format!("/result/repetitions/{instance}/raw.tsv")
                    }]
                },
                "out_dir": result_root.join("repetitions").join(&instance)
            }))?,
        )?;
        plans.push(BenchmarkStagePlanReference {
            repetition,
            execution_unit_id: "sample-a".to_owned(),
            stage_instance_id: instance,
            path: relative,
            sha256: bijux_dna_infra::hash_file_sha256(&path)?,
        });
    }
    let manifest = BenchmarkStagePlanManifest {
        schema_version: BENCHMARK_STAGE_PLAN_MANIFEST_SCHEMA_V1.to_owned(),
        identity: spec.identity.clone(),
        execution_spec_sha256: bijux_dna_infra::hash_file_sha256(&execution_spec_path)?,
        plans,
    };
    std::fs::write(
        result_root.join("stage-plan-manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(Fixture { root, spec, input_path })
}

fn run_git(repository: &std::path::Path, arguments: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git").arg("-C").arg(repository).args(arguments).output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
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
    assert!(script.contains("'--env' 'TMPDIR=/scratch'"));
    assert!(script.contains(":/repository:ro"));
    assert!(script.contains(":/input/ERR1.fastq.gz:ro"));
    assert!(script.contains("repetition-3.resource.tsv.partial"));
    assert!(script.contains(
        "'/usr/bin/time' '-o' \"${RESULT_ROOT}/\"'repetitions/repetition-1.resource.tsv.partial' '-f'"
    ));
    assert!(script.contains("test -x /usr/bin/time"));
    assert!(script.contains("command -v find >/dev/null"));
    assert!(script.contains("command -v sort >/dev/null"));
    assert!(!script.contains("'--quiet'"));
    assert!(!script.contains("'--output="));
    assert!(!script.contains("'--format="));
    let measured_line = script
        .lines()
        .find(|line| {
            line.starts_with("'/usr/bin/time'")
                && line.contains("repetition-1.resource.tsv.partial")
        })
        .ok_or_else(|| anyhow::anyhow!("missing measured benchmark command"))?;
    assert!(measured_line.starts_with("'/usr/bin/time'"));
    assert!(script.contains("%e\t%U\t%S\t%M\t%I\t%O\t%x' 'apptainer' 'exec' "));
    assert!(!script.contains("{repetition}"));
    assert!(script.contains("output-sha256.tsv"));
    assert!(script.contains("elif test -d"));
    assert!(script.contains("-type f -print0 | LC_ALL=C sort -z"));
    assert!(script.contains("required benchmark output is absent"));
    assert!(!script.lines().any(|line| line.contains("test -f") && line.contains("&&")));
    assert!(!script.lines().any(|line| line.contains("test ! -e") && line.contains("&&")));
    assert!(script.contains("stage-plan-manifest.json"));
    assert!(script.contains("raw > /result/repetitions/measured-repetition-3--sample-a/raw.tsv"));
    assert!(script.contains("repetitions/measured-repetition-3--sample-a/resource.tsv.partial"));
    let stage_plan_position = script
        .find("measured-repetition-3--sample-a/raw.tsv")
        .ok_or_else(|| anyhow::anyhow!("missing stage-plan command"))?;
    let aggregate_position = script
        .find(
            "'/usr/bin/time' '-o' \"${RESULT_ROOT}/\"'repetitions/repetition-1.resource.tsv.partial'",
        )
        .ok_or_else(|| anyhow::anyhow!("missing aggregate command"))?;
    assert!(stage_plan_position < aggregate_position);
    assert!(script.contains("completed.json.partial"));
    assert!(Command::new("bash").arg("-n").arg(&first.request.script_path).status()?.success());
    Ok(())
}

#[test]
fn benchmark_slurm_material_rejects_stage_plan_outputs_outside_required_closure(
) -> anyhow::Result<()> {
    let fixture = fixture()?;
    let result_root = PathBuf::from(&fixture.spec.outputs.result_root);
    let plan_path = result_root.join("stage-plans/repetition-1/sample-a.json");
    let mut plan: serde_json::Value = serde_json::from_slice(&std::fs::read(&plan_path)?)?;
    plan["io"]["outputs"][0]["path"] = serde_json::Value::String(
        "/result/repetitions/measured-repetition-1--sample-a/undeclared.tsv".to_owned(),
    );
    std::fs::write(&plan_path, serde_json::to_vec_pretty(&plan)?)?;
    let manifest_path = result_root.join("stage-plan-manifest.json");
    let mut manifest: BenchmarkStagePlanManifest =
        serde_json::from_slice(&std::fs::read(&manifest_path)?)?;
    manifest.plans[0].sha256 = bijux_dna_infra::hash_file_sha256(&plan_path)?;
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    assert!(prepare_benchmark_slurm_job(&fixture.spec, "2026-08-05T02:10:00Z")
        .is_err_and(|error| error.to_string().contains("required output authority")));
    Ok(())
}

#[test]
fn benchmark_slurm_material_rejects_missing_or_mutated_stage_plan_authority() -> anyhow::Result<()>
{
    let mutated = fixture()?;
    let result_root = PathBuf::from(&mutated.spec.outputs.result_root);
    std::fs::write(result_root.join("stage-plans/repetition-2/sample-a.json"), b"mutated")?;
    assert!(prepare_benchmark_slurm_job(&mutated.spec, "2026-08-05T02:10:00Z")
        .is_err_and(|error| error.to_string().contains("digest differs")));

    let missing = fixture()?;
    std::fs::remove_file(
        PathBuf::from(&missing.spec.outputs.result_root).join("stage-plan-manifest.json"),
    )?;
    assert!(prepare_benchmark_slurm_job(&missing.spec, "2026-08-05T02:10:00Z")
        .is_err_and(|error| error.to_string().contains("immutable JSON")));
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

#[test]
fn benchmark_slurm_material_rejects_mutable_source_snapshots() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let repository = PathBuf::from(&fixture.spec.source.repository_path);
    let mut permissions = std::fs::metadata(&repository)?.permissions();
    permissions.set_mode(permissions.mode() | 0o200);
    std::fs::set_permissions(&repository, permissions)?;

    let error = prepare_benchmark_slurm_job(&fixture.spec, "2026-08-05T02:10:00Z")
        .err()
        .ok_or_else(|| anyhow::anyhow!("mutable benchmark source was accepted"))?;
    assert!(format!("{error:#}").contains("remains writable"));
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

    write_executable(&paths.sbatch, "#!/bin/sh\nprintf '999999\\n'\n")?;
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
    write_executable(&paths.sbatch, &format!("#!/bin/sh\n{sbatch_body}"))?;
    Ok(paths)
}

fn shell_literal(path: &std::path::Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}
