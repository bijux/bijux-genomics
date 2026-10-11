use std::path::PathBuf;

use bijux_dna_bench_model::{
    benchmark_slurm_submission_intent_schema, validate_benchmark_slurm_submission_intent,
    BenchmarkSlurmArraySpec, BenchmarkSlurmDependencyCondition, BenchmarkSlurmSubmissionIntent,
    BenchmarkSlurmSubmissionMaterial, BENCHMARK_SLURM_SUBMISSION_INTENT_SCHEMA_V1,
};

fn material() -> BenchmarkSlurmSubmissionMaterial {
    BenchmarkSlurmSubmissionMaterial {
        schema_version: BENCHMARK_SLURM_SUBMISSION_INTENT_SCHEMA_V1.to_string(),
        run_id: "run-2001:tool:attempt-001".to_string(),
        dependencies: vec!["3502947".to_string()],
        dependency_condition: BenchmarkSlurmDependencyCondition::AfterOk,
        array: Some(BenchmarkSlurmArraySpec {
            first_task_id: 1,
            last_task_id: 4,
            maximum_parallel_tasks: Some(2),
        }),
        execution_spec_sha256: "1".repeat(64),
        execution_unit_manifest_sha256: "2".repeat(64),
        script_sha256: "3".repeat(64),
        script_path: PathBuf::from("/study/attempt/scheduler/job.sbatch"),
        working_directory: PathBuf::from("/home/bijan/bijux/bijux-genomics"),
        stdout_path: PathBuf::from("/study/attempt/scheduler/stdout.log"),
        stderr_path: PathBuf::from("/study/attempt/scheduler/stderr.log"),
        submitted_at_utc: "2026-08-15T10:00:00Z".to_string(),
    }
}

#[test]
fn submission_intent_preserves_wire_identity_through_generic_material() -> anyhow::Result<()> {
    let expected = material();
    let intent = BenchmarkSlurmSubmissionIntent::from_material(expected.clone());
    validate_benchmark_slurm_submission_intent(&intent)?;
    assert_eq!(intent.material(), expected);

    let encoded = serde_json::to_value(&intent)?;
    assert_eq!(encoded["stage_plan_manifest_sha256"], "2".repeat(64));
    assert!(encoded.get("execution_unit_manifest_sha256").is_none());
    Ok(())
}

#[test]
fn submission_intent_rejects_invalid_scheduler_evidence() -> anyhow::Result<()> {
    let mut invalid_digest = BenchmarkSlurmSubmissionIntent::from_material(material());
    invalid_digest.script_sha256 = "not-a-digest".to_string();
    assert!(validate_benchmark_slurm_submission_intent(&invalid_digest).is_err());

    let mut invalid_dependency = BenchmarkSlurmSubmissionIntent::from_material(material());
    invalid_dependency.dependencies = vec!["job-name".to_string()];
    assert!(validate_benchmark_slurm_submission_intent(&invalid_dependency).is_err());

    let mut duplicate_dependency = BenchmarkSlurmSubmissionIntent::from_material(material());
    duplicate_dependency.dependencies = vec!["3502947".to_string(), "3502947".to_string()];
    assert!(validate_benchmark_slurm_submission_intent(&duplicate_dependency).is_err());

    let mut missing_after_any_dependency =
        BenchmarkSlurmSubmissionIntent::from_material(material());
    missing_after_any_dependency.dependencies.clear();
    missing_after_any_dependency.dependency_condition = BenchmarkSlurmDependencyCondition::AfterAny;
    assert!(validate_benchmark_slurm_submission_intent(&missing_after_any_dependency).is_err());

    let mut invalid_array = BenchmarkSlurmSubmissionIntent::from_material(material());
    invalid_array
        .array
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("array fixture is missing"))?
        .maximum_parallel_tasks = Some(0);
    assert!(validate_benchmark_slurm_submission_intent(&invalid_array).is_err());

    let mut oversized_parallelism = BenchmarkSlurmSubmissionIntent::from_material(material());
    oversized_parallelism
        .array
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("array fixture is missing"))?
        .maximum_parallel_tasks = Some(5);
    assert!(validate_benchmark_slurm_submission_intent(&oversized_parallelism).is_err());
    Ok(())
}

#[test]
fn submission_intent_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/benchmark-slurm-submission-intent.schema.json");
    let generated = serde_json::to_value(benchmark_slurm_submission_intent_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "submission intent schema drifted from its model owner");
    Ok(())
}
