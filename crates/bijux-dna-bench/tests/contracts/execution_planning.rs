use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_bench::{
    build_execution_spec, load_checked_in_benchmark_planning_contracts,
    plan_stage_campaign_attempts, BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric,
    BenchmarkAcceptanceOperator, BenchmarkExecutionAcceptance, BenchmarkExecutionDeterminism,
    BenchmarkExecutionInput, BenchmarkExecutionInputKind, BenchmarkExecutionMaterial,
    BenchmarkExecutionParameterEvidence, BenchmarkExecutionSpec,
};
use bijux_dna_environment::api::{load_container_catalog, ContainerCatalog, CurrentApptainerImage};

fn plan() -> anyhow::Result<bijux_dna_bench::BenchmarkCampaignPlan> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    plan_stage_campaign_attempts(
        &registry,
        &"a".repeat(40),
        &"b".repeat(64),
        "run-2042",
        "human-short-read-length-comparison",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )
}

fn authorities() -> anyhow::Result<(
    bijux_dna_bench::BenchmarkStageCampaignRegistry,
    bijux_dna_bench::BenchmarkStageInterfaceCatalog,
    ContainerCatalog,
)> {
    let (registry, _, interfaces) = load_checked_in_benchmark_planning_contracts()?;
    let catalog = load_container_catalog(&super::support::repo_root()?)?;
    Ok((registry, interfaces, catalog))
}

fn image() -> CurrentApptainerImage {
    CurrentApptainerImage {
        tool_id: "seqkit".to_string(),
        path: PathBuf::from("/study/containers/apptainer/seqkit.sif"),
        sha256: "c".repeat(64),
        size_bytes: 42,
        repository_commit: "a".repeat(40),
        catalog_sha256: "d".repeat(64),
    }
}

fn material() -> BenchmarkExecutionMaterial {
    BenchmarkExecutionMaterial {
        operator: "bijan".to_string(),
        prepared_at_utc: "2026-08-05T00:01:00Z".to_string(),
        repository_path: PathBuf::from("/study/worktrees/source"),
        benchmark_root: PathBuf::from("/study/results/benchmarks"),
        source_contract_sha256: BTreeMap::from([
            ("campaign_registry".to_string(), "b".repeat(64)),
            ("container_catalog".to_string(), "d".repeat(64)),
            ("image_inventory".to_string(), "e".repeat(64)),
            ("stage_interface_registry".to_string(), "f".repeat(64)),
        ]),
        inputs: vec![BenchmarkExecutionInput {
            id: "fastq_err1".to_string(),
            kind: BenchmarkExecutionInputKind::Data,
            path: "/study/data/ERR1.fastq.gz".to_string(),
            mount_path: "/input/ERR1.fastq.gz".to_string(),
            sha256: "1".repeat(64),
        }],
        warmup_command: shell("true"),
        measured_command: shell("printf measured"),
        negative_command: shell("seqkit missing.fastq"),
        parameters: BenchmarkExecutionParameterEvidence {
            parameter_schema_version: "bijux.fastq.params.read_length_profile.v1".to_string(),
            defaults_provider: "bijux_dna_domain_fastq::read_length_profile_defaults".to_string(),
            overrides_path: "config-pipeline-parameters.yml".to_string(),
            overrides_sha256: "2".repeat(64),
            effective_path: "effective-parameters.json".to_string(),
            effective_sha256: "3".repeat(64),
        },
        required_files: vec!["repetitions/repetition-1.metrics.json".to_string()],
        account: "lu2024-12-24".to_string(),
        partition: "compute".to_string(),
        memory_mb: 8192,
        walltime: "01:00:00".to_string(),
        scratch_mb: 4096,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
    }
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
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
    }
}

fn shell(command: &str) -> Vec<String> {
    vec!["sh".to_string(), "-c".to_string(), command.to_string()]
}

fn build(
    catalog: &ContainerCatalog,
    selected_image: &CurrentApptainerImage,
) -> anyhow::Result<BenchmarkExecutionSpec> {
    let (registry, interfaces, _) = authorities()?;
    let plan = plan()?;
    let identity = plan
        .cycle
        .attempts
        .iter()
        .find(|attempt| attempt.identity.tool_id == "seqkit")
        .ok_or_else(|| anyhow::anyhow!("run-2042 plan omits seqkit"))?
        .identity
        .clone();
    build_execution_spec(
        &registry,
        &interfaces,
        catalog,
        &plan,
        &identity,
        selected_image,
        material(),
    )
}

#[test]
fn execution_builder_derives_governed_tool_and_attempt_surfaces() -> anyhow::Result<()> {
    let (_, _, catalog) = authorities()?;

    let spec = build(&catalog, &image())?;

    assert_eq!(spec.resources.cpus, 4);
    assert_eq!(spec.runtime.threads, 4);
    assert_eq!(spec.measurement.warmup_repetitions, 1);
    assert_eq!(spec.measurement.measured_repetitions, 3);
    assert_eq!(spec.commands.version[..2], ["sh", "-c"]);
    assert!(spec.commands.version[2].contains("seqkit"));
    assert!(spec.interface.adapter.contains("profile_read_lengths"));
    assert!(spec.outputs.result_root.ends_with(
        "/run-2042-fastq-profile-read-lengths/seqkit/attempts/human-short-read-length-comparison/attempt-001"
    ));
    Ok(())
}

#[test]
fn execution_builder_is_deterministic_for_identical_authorities() -> anyhow::Result<()> {
    let (_, _, catalog) = authorities()?;
    let first = build(&catalog, &image())?;
    let second = build(&catalog, &image())?;
    assert_eq!(serde_json::to_vec(&first)?, serde_json::to_vec(&second)?);
    Ok(())
}

#[test]
fn execution_builder_rejects_image_and_catalog_authority_drift() -> anyhow::Result<()> {
    let (_, _, mut catalog) = authorities()?;
    let mut stale_image = image();
    stale_image.repository_commit = "9".repeat(40);
    assert!(build(&catalog, &stale_image).is_err());

    let seqkit = catalog
        .tools
        .iter_mut()
        .find(|tool| tool.id == "seqkit")
        .ok_or_else(|| anyhow::anyhow!("container catalog omits seqkit"))?;
    seqkit.stage_ids.clear();
    assert!(build(&catalog, &image()).is_err());
    Ok(())
}
