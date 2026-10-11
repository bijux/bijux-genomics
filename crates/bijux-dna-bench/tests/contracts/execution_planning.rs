use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Context;
use bijux_dna_bench::{
    build_execution_spec, ensure_execution_spec, ensure_stage_campaign_attempts,
    load_checked_in_benchmark_planning_contracts, load_execution_authorities, load_execution_spec,
    plan_stage_campaign_attempts, BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric,
    BenchmarkAcceptanceOperator, BenchmarkExecutionAcceptance, BenchmarkExecutionDeterminism,
    BenchmarkExecutionInput, BenchmarkExecutionInputKind, BenchmarkExecutionMaterial,
    BenchmarkExecutionParameterEvidence, BenchmarkExecutionSpec,
};
use bijux_dna_environment::api::{
    load_container_catalog, load_image_inventory_publication, ContainerCatalog,
    CurrentApptainerImage,
};

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

fn source_locked_plan(
    repository_commit: &str,
    campaign_registry_sha256: &str,
) -> anyhow::Result<bijux_dna_bench::BenchmarkCampaignPlan> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    plan_stage_campaign_attempts(
        &registry,
        repository_commit,
        campaign_registry_sha256,
        "run-2042",
        "human-short-read-length-comparison",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )
}

#[test]
fn execution_authorities_bind_exact_source_files_and_reject_stale_inventory() -> anyhow::Result<()>
{
    let repository_root = super::support::repo_root()?;
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-execution-authorities");
    let inventory_path = temporary.path().join("image-inventory.json");
    let mut inventory = load_image_inventory_publication(
        &repository_root.join("containers/reports/image-inventory.json"),
    )?;
    let repository_commit = "a".repeat(40);
    let campaign_sha256 = bijux_dna_infra::hash_file_sha256(
        &repository_root.join("benchmarks/contracts/stage-campaigns.toml"),
    )?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &repository_root.join("containers/contracts/catalog.toml"),
    )?;
    inventory.repository_commit.clone_from(&repository_commit);
    inventory.catalog_sha256.clone_from(&catalog_sha256);
    std::fs::write(&inventory_path, serde_json::to_vec_pretty(&inventory)?)?;
    let plan = source_locked_plan(&repository_commit, &campaign_sha256)?;

    let authorities =
        load_execution_authorities(&repository_root, &inventory_path, &repository_commit, &plan)?;
    assert_eq!(authorities.source_contract_sha256()["container_catalog"], catalog_sha256);
    assert_eq!(
        authorities.source_contract_sha256()["image_inventory"],
        bijux_dna_infra::hash_file_sha256(&inventory_path)?
    );

    inventory.repository_commit = "9".repeat(40);
    let stale_path = temporary.path().join("stale-image-inventory.json");
    std::fs::write(&stale_path, serde_json::to_vec_pretty(&inventory)?)?;
    assert!(load_execution_authorities(&repository_root, &stale_path, &repository_commit, &plan,)
        .is_err_and(|error| error.to_string().contains("repository commit differs")));
    Ok(())
}

#[test]
fn execution_authorities_recheck_selected_sif_bytes() -> anyhow::Result<()> {
    let repository_root = super::support::repo_root()?;
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-execution-image");
    let study_root = temporary.path().join("STUDY");
    let image_path = study_root.join("containers/apptainer/seqkit.sif");
    std::fs::create_dir_all(image_path.parent().context("image parent")?)?;
    std::fs::write(&image_path, b"source-locked seqkit image")?;
    let image_sha256 = bijux_dna_infra::hash_file_sha256(&image_path)?;

    let mut inventory = load_image_inventory_publication(
        &repository_root.join("containers/reports/image-inventory.json"),
    )?;
    let repository_commit = "a".repeat(40);
    let campaign_sha256 = bijux_dna_infra::hash_file_sha256(
        &repository_root.join("benchmarks/contracts/stage-campaigns.toml"),
    )?;
    inventory.repository_commit.clone_from(&repository_commit);
    inventory.catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &repository_root.join("containers/contracts/catalog.toml"),
    )?;
    let seqkit = inventory
        .tools
        .iter_mut()
        .find(|tool| tool.tool == "seqkit")
        .context("inventory omits seqkit")?;
    let selected_sha256 = seqkit.selected_image_sha256.clone().context("seqkit selection")?;
    let selected = seqkit
        .candidates
        .iter_mut()
        .find(|candidate| candidate.sha256 == selected_sha256)
        .context("inventory omits selected seqkit candidate")?;
    selected.path = "study://containers/apptainer/seqkit.sif".to_string();
    selected.sha256.clone_from(&image_sha256);
    selected.size_bytes = std::fs::metadata(&image_path)?.len();
    seqkit.selected_image_sha256 = Some(image_sha256.clone());
    let inventory_path = temporary.path().join("image-inventory.json");
    std::fs::write(&inventory_path, serde_json::to_vec_pretty(&inventory)?)?;
    let plan = source_locked_plan(&repository_commit, &campaign_sha256)?;

    let authorities =
        load_execution_authorities(&repository_root, &inventory_path, &repository_commit, &plan)?;
    let image = authorities.resolve_current_image(&study_root, "seqkit")?;
    assert_eq!(image.path, image_path.canonicalize()?);
    assert_eq!(image.sha256, image_sha256);

    let identity = plan
        .cycle
        .attempts
        .iter()
        .find(|attempt| attempt.identity.tool_id == "seqkit")
        .context("run-2042 plan omits seqkit")?
        .identity
        .clone();
    let mut execution_material = material();
    execution_material.repository_path = repository_root.clone();
    execution_material.benchmark_root = temporary.path().join("results/benchmarks");
    execution_material.source_contract_sha256 = authorities.source_contract_sha256().clone();
    let spec = authorities.build_execution_spec(&plan, &identity, &image, execution_material)?;
    assert_eq!(spec.image.sha256, image_sha256);

    let mut wrong_material = material();
    wrong_material.repository_path = temporary.path().to_path_buf();
    wrong_material.benchmark_root = temporary.path().join("results/benchmarks");
    wrong_material.source_contract_sha256 = authorities.source_contract_sha256().clone();
    assert!(authorities
        .build_execution_spec(&plan, &identity, &image, wrong_material)
        .is_err_and(|error| error.to_string().contains("repository path differs")));

    std::fs::write(&image_path, b"mutated")?;
    assert!(authorities.resolve_current_image(&study_root, "seqkit").is_err());
    Ok(())
}

fn build(
    catalog: &ContainerCatalog,
    selected_image: &CurrentApptainerImage,
) -> anyhow::Result<BenchmarkExecutionSpec> {
    build_at(catalog, selected_image, std::path::Path::new("/study/results/benchmarks"))
}

fn build_at(
    catalog: &ContainerCatalog,
    selected_image: &CurrentApptainerImage,
    benchmark_root: &std::path::Path,
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
    let mut execution_material = material();
    execution_material.benchmark_root = benchmark_root.to_path_buf();
    build_execution_spec(
        &registry,
        &interfaces,
        catalog,
        &plan,
        &identity,
        selected_image,
        execution_material,
    )
}

#[test]
fn execution_spec_storage_is_restart_safe_and_refuses_conflicts() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-execution-spec-storage");
    let plan = plan()?;
    ensure_stage_campaign_attempts(temporary.path(), &plan)?;
    let (_, _, catalog) = authorities()?;
    let spec = build_at(&catalog, &image(), temporary.path())?;

    let first = ensure_execution_spec(temporary.path(), &spec)?;
    let recovered = ensure_execution_spec(temporary.path(), &spec)?;
    assert_eq!(first, recovered);
    let persisted: BenchmarkExecutionSpec = serde_json::from_slice(&std::fs::read(first)?)?;
    assert_eq!(persisted, spec);
    assert_eq!(load_execution_spec(temporary.path(), &spec.identity)?, spec);

    let mut conflicting = spec;
    conflicting.operator = "different-operator".to_string();
    assert!(ensure_execution_spec(temporary.path(), &conflicting).is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn execution_spec_storage_rejects_symlink_substitution() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-execution-spec-symlink");
    let plan = plan()?;
    ensure_stage_campaign_attempts(temporary.path(), &plan)?;
    let (_, _, catalog) = authorities()?;
    let spec = build_at(&catalog, &image(), temporary.path())?;
    let external = temporary.path().join("external.json");
    std::fs::write(&external, serde_json::to_vec_pretty(&spec)?)?;
    std::os::unix::fs::symlink(
        &external,
        std::path::Path::new(&spec.outputs.result_root).join("execution-spec.json"),
    )?;

    assert!(ensure_execution_spec(temporary.path(), &spec).is_err());
    assert_eq!(std::fs::read(external)?, serde_json::to_vec_pretty(&spec)?);
    Ok(())
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
