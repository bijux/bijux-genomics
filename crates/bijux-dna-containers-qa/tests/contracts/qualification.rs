use std::collections::BTreeSet;
use std::fs;
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use anyhow::Context;
use anyhow::Result;
#[cfg(unix)]
use bijux_dna_containers_qa::api::{
    bind_qualification_source_to_frozen_repository, freeze_repository, ContainerGovernance,
    ContainerPathPolicy, ContainerPreflightPolicy, ContainerRuntimePolicy, ContainerSitePolicy,
    ContainerSlurmPolicy, QualificationSpecificationInputKindV1, QualificationSpecificationInputV1,
    QualificationSpecificationSiteResourcesV1, QualificationSpecificationSourceV1,
    RepositoryFreezeRequest, RepositoryFreezeScope, CONTAINER_GOVERNANCE_SCHEMA_V1,
    QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
};
use bijux_dna_containers_qa::api::{
    evaluate_container_qualification, plan_container_qualification, ContainerPipelineContract,
    ContainerPipelineV2, QualificationFailureClass, QualificationProbeKind,
    QualificationProbeObservation,
};
use serde_json::json;

#[test]
fn checked_merge_qualification_selections_close_their_binding_inputs_and_parameters() -> Result<()>
{
    use bijux_dna_core::ids::{StageId, ToolId};
    use bijux_dna_stage_contract::ToolBinding;

    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let selection_root = repository.join("containers/contracts/qualification-selections");
    let registry = bijux_dna_containers::api::load_qualification_run_registry(
        &repository,
        &repository.join("containers/contracts/qualification-runs.toml"),
    )?;
    let bindings = bijux_dna_stage_contract::canonical_tool_bindings()?;
    let mut observed = BTreeSet::new();
    for entry in fs::read_dir(&selection_root)? {
        let entry = entry?;
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let path = entry.path();
        let selection = bijux_dna_containers_qa::api::load_qualification_source_selection(&path)?;
        let file_tool = path.file_stem().and_then(|value| value.to_str()).unwrap_or_default();
        assert_eq!(selection.tool_id, file_tool);
        let registered = registry
            .tools
            .iter()
            .find(|tool| tool.tool_id == selection.tool_id)
            .context("selection tool is absent from the qualification registry")?;
        if registered.stage_id != "fastq.merge_pairs" {
            continue;
        }
        assert!(observed.insert(selection.tool_id.clone()));
        assert_eq!(registered.input_role, selection.corpus_role);
        let binding = bindings
            .binding(&StageId::new(&registered.stage_id), &ToolId::new(&selection.tool_id))
            .context("selection stage/tool binding is absent")?;
        let selected_inputs = selection
            .inputs
            .iter()
            .map(|input| input.binding_input.as_str())
            .collect::<BTreeSet<_>>();
        let required_inputs = binding
            .inputs()
            .iter()
            .filter(|input| input.required)
            .map(|input| input.name.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(selected_inputs, required_inputs);
        let declared_parameters = binding
            .parameters()
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<BTreeSet<_>>();
        assert!(selection
            .parameters
            .keys()
            .all(|name| declared_parameters.contains(name.as_str())));
        assert!(binding.parameters().iter().filter(|parameter| parameter.required).all(
            |parameter| {
                selection.parameters.get(&parameter.name).is_some_and(|value| !value.is_empty())
            }
        ));
        assert_eq!(selection.inputs.len(), 2);
        assert_eq!(selection.inputs[0].catalog_id, selection.inputs[1].catalog_id);
    }
    assert_eq!(
        observed,
        ["adapterremoval", "bbmerge", "flash2", "leehom", "pear", "seqprep"]
            .into_iter()
            .map(str::to_string)
            .collect()
    );
    Ok(())
}

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

#[cfg(unix)]
#[test]
fn qualification_source_binding_requires_the_governed_commit_freeze() -> Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("qualification-source-freeze");
    let active = temporary.path().join("repository");
    let study = temporary.path().join("study");
    let study_alias = temporary.path().join("study-alias");
    let frozen_repositories = study.join("data/evidence/source-checkouts");
    fs::create_dir_all(&active)?;
    for relative in ["containers", "data", "results", "finalization"] {
        fs::create_dir_all(study.join(relative))?;
    }
    std::os::unix::fs::symlink(&study, &study_alias)?;
    fs::write(study.join("data/input.fastq"), b"@read\nACGT\n+\nIIII\n")?;
    git(&active, &["init", "--quiet"])?;
    git(&active, &["config", "user.name", "Bijux Test"])?;
    git(&active, &["config", "user.email", "test@bijux.invalid"])?;
    fs::write(active.join("source.txt"), "source\n")?;
    git(&active, &["add", "source.txt"])?;
    git(&active, &["commit", "--quiet", "-m", "fixture"])?;
    let commit = git(&active, &["rev-parse", "HEAD"])?;
    let governance = qualification_governance(active, &study, frozen_repositories);
    let frozen = freeze_repository(&RepositoryFreezeRequest {
        governance: governance.clone(),
        hostname: "worker".to_string(),
        commit: commit.clone(),
        scope: RepositoryFreezeScope::Complete,
        checkout_paths: Vec::new(),
        source_namespace: "source-freezes".to_string(),
    })?;
    let source = QualificationSpecificationSourceV1 {
        schema_version: QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1.to_string(),
        catalog_sha256: "a".repeat(64),
        registry_sha256: "b".repeat(64),
        inventory_sha256: "c".repeat(64),
        governed_study_root: study_alias.clone(),
        repository_path: PathBuf::from("/mutable/repository"),
        repository_commit: commit,
        run_id: "run-1001".to_string(),
        tool_id: "tool".to_string(),
        campaign: "campaign".to_string(),
        corpus: "corpus".to_string(),
        corpus_role: "fastq".to_string(),
        sample: "sample".to_string(),
        operator: "operator".to_string(),
        submitted_at_utc: "2026-08-11T00:00:00Z".to_string(),
        result_root: study_alias.join("attempts/container-qualification/tool/legacy/results"),
        inputs: vec![QualificationSpecificationInputV1 {
            binding_input: "reads_r1".to_string(),
            artifact_contract_id: "bijux.fastq.artifact.single_end.v1".to_string(),
            id: "input-reads".to_string(),
            kind: QualificationSpecificationInputKindV1::Data,
            path: study_alias.join("data/input.fastq"),
            mount_path: PathBuf::from("/input/input.fastq"),
            sha256: "d".repeat(64),
        }],
        parameters: std::collections::BTreeMap::new(),
        outputs: Vec::new(),
        negative_command: vec!["tool".to_string(), "--invalid".to_string()],
        site: QualificationSpecificationSiteResourcesV1 {
            account: "science".to_string(),
            partition: "compute".to_string(),
            walltime: "00:30:00".to_string(),
        },
    };

    let bound = bind_qualification_source_to_frozen_repository(&source, &governance, &frozen)?;
    assert_eq!(bound.repository_path, frozen.canonicalize()?);
    assert_eq!(bound.governed_study_root, study.canonicalize()?);
    assert_eq!(
        bound.result_root,
        study.canonicalize()?.join("results/container-validation/run-1001-tool")
    );
    assert_eq!(bound.inputs[0].path, study.canonicalize()?.join("data/input.fastq"));
    let outside = temporary.path().join("outside").join(&source.repository_commit);
    fs::create_dir_all(&outside)?;
    let error = bind_qualification_source_to_frozen_repository(&source, &governance, &outside)
        .err()
        .context("qualification source accepted an unmanaged repository path")?;
    assert!(error.to_string().contains("not frozen by commit identity"));
    Ok(())
}

#[cfg(unix)]
fn qualification_governance(
    active_repository: PathBuf,
    study_root: &Path,
    frozen_repositories: PathBuf,
) -> ContainerGovernance {
    ContainerGovernance {
        schema_version: CONTAINER_GOVERNANCE_SCHEMA_V1.to_string(),
        site: ContainerSitePolicy {
            id: "test-site".to_string(),
            hostname_regex: "^worker$".to_string(),
        },
        paths: ContainerPathPolicy {
            study_root: study_root.to_path_buf(),
            containers_root: study_root.join("containers"),
            data_root: study_root.join("data"),
            results_root: study_root.join("results"),
            finalization_root: study_root.join("finalization"),
            active_repository,
            frozen_repositories,
        },
        runtime: ContainerRuntimePolicy { apptainer_architecture: "linux/amd64".to_string() },
        slurm: ContainerSlurmPolicy {
            account: "science".to_string(),
            scratch_environment: Some("TMPDIR".to_string()),
            scratch_root: None,
            partitions: vec!["compute".to_string()],
            quota_command: vec!["true".to_string()],
        },
        preflight: ContainerPreflightPolicy {
            minimum_available_bytes: 1,
            minimum_available_inodes: 1,
            maximum_clock_skew_seconds: 30,
            required_commands: vec!["git".to_string()],
            required_contracts: vec!["source.txt".to_string()],
        },
    }
}

#[cfg(unix)]
fn git(repository: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .output()
        .with_context(|| format!("run git {}", arguments.join(" ")))?;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
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
