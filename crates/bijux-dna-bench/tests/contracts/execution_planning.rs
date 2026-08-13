use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use bijux_dna_bench::{
    build_benchmark_stage_plans, build_execution_spec, ensure_execution_spec,
    ensure_stage_campaign_attempts, ensure_stage_plan_manifest,
    load_checked_in_benchmark_planning_contracts, load_execution_authorities, load_execution_spec,
    load_stage_plan_manifest, plan_stage_campaign_attempts, BenchmarkAcceptanceExpectedValue,
    BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator, BenchmarkExecutionAcceptance,
    BenchmarkExecutionDeterminism, BenchmarkExecutionInput, BenchmarkExecutionInputKind,
    BenchmarkExecutionMaterial, BenchmarkExecutionParameterEvidence, BenchmarkExecutionSpec,
    BenchmarkStagePlanInputMaterial, BenchmarkStagePlanOutputMaterial,
    BenchmarkStagePlanUnitMaterial,
};
use bijux_dna_containers::api::{
    load_container_catalog, load_image_inventory_publication, ContainerCatalog,
    CurrentApptainerImage,
};
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, ToolConstraints};
use bijux_dna_core::ids::{ArtifactId, StageId, StepId, ToolId};
use bijux_dna_core::prelude::ContainerImageRefV1;
use bijux_dna_domain_fastq::params::stats::READ_LENGTH_PROFILE_SCHEMA_VERSION;
use bijux_dna_stage_contract::{
    build_binding_stage_plan, BindingInputV1, BindingOutputV1, BindingStagePlanRequest,
    EffectiveToolParameterSourceV1, EffectiveToolParameterV1, EffectiveToolParametersV1,
    OutputDiscoveryV1, ToolArtifactSourceV1, ToolBindingV1, EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1,
    TOOL_BINDING_SCHEMA_V1,
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
            parameter_schema_version: READ_LENGTH_PROFILE_SCHEMA_VERSION.to_string(),
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
        stage_plan_units: Some(vec![BenchmarkStagePlanUnitMaterial {
            execution_unit_id: "sample-a".to_owned(),
            inputs: vec![BenchmarkStagePlanInputMaterial {
                binding_name: "reads_r1".to_owned(),
                path: PathBuf::from("/input/ERR1.fastq.gz"),
                role: ArtifactRole::Reads,
            }],
            outputs: vec![BenchmarkStagePlanOutputMaterial {
                binding_name: "length_distribution_tsv".to_owned(),
                relative_path: "length-distribution.tsv".into(),
                role: ArtifactRole::SummaryTsv,
            }],
            scientific_defaults: BTreeMap::from([("threads".to_owned(), "4".to_owned())]),
            input_derived: BTreeMap::new(),
            operator_overrides: BTreeMap::new(),
            immutable_parameters: ["threads".to_owned()].into_iter().collect(),
        }]),
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
    source_locked_plan_for(
        repository_commit,
        campaign_registry_sha256,
        "run-2042",
        "human-short-read-length-comparison",
    )
}

fn source_locked_plan_for(
    repository_commit: &str,
    campaign_registry_sha256: &str,
    run_id: &str,
    campaign_id: &str,
) -> anyhow::Result<bijux_dna_bench::BenchmarkCampaignPlan> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    plan_stage_campaign_attempts(
        &registry,
        repository_commit,
        campaign_registry_sha256,
        run_id,
        campaign_id,
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
    assert_eq!(
        authorities.source_contract_sha256()["planner_contract"],
        bijux_dna_infra::hash_file_sha256(&repository_root.join(
            "benchmarks/contracts/planner/fastq-profile-read-lengths.planner-contract.json"
        ))?
    );

    inventory.repository_commit = "9".repeat(40);
    let stale_path = temporary.path().join("stale-image-inventory.json");
    std::fs::write(&stale_path, serde_json::to_vec_pretty(&inventory)?)?;
    assert!(load_execution_authorities(&repository_root, &stale_path, &repository_commit, &plan,)
        .is_err_and(|error| error.to_string().contains("repository commit differs")));
    Ok(())
}

#[test]
fn execution_authorities_own_validation_planner_contract_lock() -> anyhow::Result<()> {
    let repository_root = super::support::repo_root()?;
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-validation-authorities");
    let inventory_path = temporary.path().join("image-inventory.json");
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
    std::fs::write(&inventory_path, serde_json::to_vec_pretty(&inventory)?)?;
    let plan = source_locked_plan_for(
        &repository_commit,
        &campaign_sha256,
        "run-2051",
        "human-short-read-validation-comparison",
    )?;

    let authorities =
        load_execution_authorities(&repository_root, &inventory_path, &repository_commit, &plan)?;
    assert_eq!(
        authorities.source_contract_sha256()["planner_contract"],
        bijux_dna_infra::hash_file_sha256(
            &repository_root
                .join("benchmarks/contracts/planner/fastq-validate-reads.planner-contract.json")
        )?
    );
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
    assert!(spec.outputs.required_files.contains(
        &"repetitions/measured-repetition-3--sample-a/length-distribution.tsv".to_owned()
    ));
    assert!(spec
        .outputs
        .required_files
        .contains(&"repetitions/measured-repetition-3--sample-a/resource.tsv".to_owned()));
    let units = [BenchmarkStagePlanUnitMaterial {
        execution_unit_id: "sample-a".to_owned(),
        inputs: vec![BenchmarkStagePlanInputMaterial {
            binding_name: "reads_r1".to_owned(),
            path: PathBuf::from(&spec.inputs[0].mount_path),
            role: ArtifactRole::Reads,
        }],
        outputs: vec![BenchmarkStagePlanOutputMaterial {
            binding_name: "length_distribution_tsv".to_owned(),
            relative_path: "length-distribution.tsv".into(),
            role: ArtifactRole::SummaryTsv,
        }],
        scientific_defaults: BTreeMap::from([("threads".to_owned(), "4".to_owned())]),
        input_derived: BTreeMap::new(),
        operator_overrides: BTreeMap::new(),
        immutable_parameters: ["threads".to_owned()].into_iter().collect(),
    }];
    let plans = authorities.build_stage_plans(&spec, &units)?;
    assert_eq!(plans.len(), 3);
    assert!(plans[0].io.outputs[0].path.starts_with("/result/repetitions"));

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

#[test]
fn stage_plan_storage_is_restart_safe_and_checksum_bound() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-stage-plan-storage");
    let campaign = plan()?;
    ensure_stage_campaign_attempts(temporary.path(), &campaign)?;
    let (_, _, catalog) = authorities()?;
    let spec = build_at(&catalog, &image(), temporary.path())?;
    ensure_execution_spec(temporary.path(), &spec)?;
    let plans = (1..=spec.measurement.measured_repetitions)
        .map(|repetition| stage_plan(&spec, repetition))
        .collect::<anyhow::Result<Vec<_>>>()?;

    let first = ensure_stage_plan_manifest(temporary.path(), &spec, &plans)?;
    let recovered = ensure_stage_plan_manifest(temporary.path(), &spec, &plans)?;
    assert_eq!(recovered, first);
    let (loaded, loaded_plans) = load_stage_plan_manifest(temporary.path(), &spec.identity)?;
    assert_eq!(loaded, first);
    assert_eq!(loaded_plans.len(), 3);
    assert_eq!(
        loaded_plans[2].stage_instance_id.as_ref().map(StepId::as_str),
        Some("measured-repetition-3--sample-a")
    );

    let plan_path =
        Path::new(&spec.outputs.result_root).join("stage-plans/repetition-2/sample-a.json");
    std::fs::write(&plan_path, b"{}")?;
    assert!(load_stage_plan_manifest(temporary.path(), &spec.identity)
        .is_err_and(|error| error.to_string().contains("bytes differ")));
    Ok(())
}

#[test]
fn stage_plan_builder_closes_every_repetition_and_execution_unit() -> anyhow::Result<()> {
    let (_, _, catalog) = authorities()?;
    let spec = build(&catalog, &image())?;
    let binding = stage_plan_binding(&spec);
    let tool = catalog
        .tools
        .iter()
        .find(|tool| tool.id == "seqkit")
        .context("container catalog omits seqkit")?;
    let unit = BenchmarkStagePlanUnitMaterial {
        execution_unit_id: "sample-a".to_owned(),
        inputs: vec![BenchmarkStagePlanInputMaterial {
            binding_name: "reads_r1".to_owned(),
            path: PathBuf::from(&spec.inputs[0].mount_path),
            role: ArtifactRole::Reads,
        }],
        outputs: vec![BenchmarkStagePlanOutputMaterial {
            binding_name: "length_distribution_tsv".to_owned(),
            relative_path: "length-distribution.tsv".into(),
            role: ArtifactRole::SummaryTsv,
        }],
        scientific_defaults: BTreeMap::from([(
            "threads".to_owned(),
            spec.resources.cpus.to_string(),
        )]),
        input_derived: BTreeMap::new(),
        operator_overrides: BTreeMap::new(),
        immutable_parameters: ["threads".to_owned()].into_iter().collect(),
    };

    let mut later_unit = unit.clone();
    later_unit.execution_unit_id = "sample-b".to_owned();
    let plans = build_benchmark_stage_plans(&spec, &binding, tool, &[later_unit, unit.clone()])?;
    assert_eq!(plans.len(), 6);
    assert_eq!(
        plans[0].stage_instance_id.as_ref().map(StepId::as_str),
        Some("measured-repetition-1--sample-a")
    );
    assert_eq!(
        plans[1].stage_instance_id.as_ref().map(StepId::as_str),
        Some("measured-repetition-1--sample-b")
    );
    assert_eq!(
        plans[4].stage_instance_id.as_ref().map(StepId::as_str),
        Some("measured-repetition-3--sample-a")
    );
    assert_eq!(plans[0].command.template[0], "seqkit");
    assert!(plans[0].io.outputs[0]
        .path
        .ends_with("measured-repetition-1--sample-a/length-distribution.tsv"));
    assert!(plans[0].io.outputs[0].path.starts_with("/result/repetitions"));
    assert!(plans[0].out_dir.starts_with(&spec.outputs.result_root));

    let mut escaping = unit.clone();
    escaping.outputs[0].relative_path = "../foreign.tsv".into();
    let error = build_benchmark_stage_plans(&spec, &binding, tool, &[escaping])
        .err()
        .context("escaping stage-plan output was accepted")?;
    assert!(error.to_string().contains("escapes"));

    let mut resource_drift = binding.clone();
    resource_drift.resources.threads = 2;
    let error =
        build_benchmark_stage_plans(&spec, &resource_drift, tool, std::slice::from_ref(&unit))
            .err()
            .context("binding resource drift was accepted")?;
    assert!(format!("{error:#}").contains("binding threads"));

    let error = build_benchmark_stage_plans(&spec, &binding, tool, &[unit.clone(), unit])
        .err()
        .context("duplicate stage-plan unit was accepted")?;
    assert!(error.to_string().contains("duplicated"));
    Ok(())
}

fn stage_plan(
    spec: &BenchmarkExecutionSpec,
    repetition: u32,
) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> {
    let binding = stage_plan_binding(spec);
    let effective = EffectiveToolParametersV1 {
        schema_version: EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1.to_owned(),
        binding_id: binding.binding_id.clone(),
        stage_id: binding.stage_id.clone(),
        tool_id: binding.tool_id.clone(),
        allocated_cpus: spec.resources.cpus,
        resources: binding.resources.clone(),
        parameters: BTreeMap::from([(
            "threads".to_owned(),
            EffectiveToolParameterV1 {
                value: spec.resources.cpus.to_string(),
                source: EffectiveToolParameterSourceV1::ScientificDefault,
            },
        )]),
    };
    let instance = format!("measured-repetition-{repetition}--sample-a");
    let out_dir = Path::new(&spec.outputs.result_root).join("repetitions").join(&instance);
    build_binding_stage_plan(BindingStagePlanRequest {
        binding: &binding,
        effective_parameters: &effective,
        image: ContainerImageRefV1 {
            image: spec.image.path.clone(),
            digest: Some(format!("sha256:{}", spec.image.sha256)),
        },
        executable: "seqkit".to_owned(),
        tool_version: "2.10.1".to_owned(),
        stage_instance_id: Some(StepId::new(instance)),
        inputs: vec![ArtifactRef {
            name: ArtifactId::new("reads_r1"),
            path: PathBuf::from(&spec.inputs[0].mount_path),
            role: ArtifactRole::Reads,
            artifact_contract_id: Some("bijux.fastq.artifact.paired_end.v1".to_owned()),
            optional: false,
        }],
        outputs: vec![ArtifactRef {
            name: ArtifactId::new("length_distribution_tsv"),
            path: out_dir.join("length-distribution.tsv"),
            role: ArtifactRole::SummaryTsv,
            artifact_contract_id: Some("bijux.auxiliary.artifact.table.v1".to_owned()),
            optional: false,
        }],
        out_dir,
    })
}

fn stage_plan_binding(spec: &BenchmarkExecutionSpec) -> ToolBindingV1 {
    ToolBindingV1 {
        schema_version: TOOL_BINDING_SCHEMA_V1.to_owned(),
        binding_id: "fastq.profile_read_lengths__seqkit".to_owned(),
        stage_id: StageId::new("fastq.profile_read_lengths"),
        tool_id: ToolId::new("seqkit"),
        artifact_source: ToolArtifactSourceV1::CatalogImage {
            catalog_tool_id: ToolId::new("seqkit"),
        },
        capabilities: vec!["single_end_reads".to_owned()],
        inputs: vec![BindingInputV1 {
            name: "reads_r1".to_owned(),
            artifact_contract_id: "bijux.fastq.artifact.paired_end.v1".to_owned(),
            required: true,
        }],
        parameters: vec![bijux_dna_stage_contract::BindingParameterV1 {
            name: "threads".to_owned(),
            required: true,
        }],
        outputs: vec![BindingOutputV1 {
            name: "length_distribution_tsv".to_owned(),
            artifact_contract_id: "bijux.auxiliary.artifact.table.v1".to_owned(),
            discovery: OutputDiscoveryV1::ExactPath,
        }],
        normalized_output_contract_id: "bijux.benchmark.fastq.profile_read_lengths.bundle.v1"
            .to_owned(),
        command_template: vec![
            "${executable}".to_owned(),
            "${input:reads_r1}".to_owned(),
            "${output:length_distribution_tsv}".to_owned(),
            "${parameter:threads}".to_owned(),
        ],
        resources: ToolConstraints {
            runtime: "apptainer".to_owned(),
            mem_gb: 4,
            tmp_gb: 4,
            threads: spec.resources.cpus,
        },
        normalizer_id: "bijux_dna_planner_fastq::tool_adapters::stages::pre::profile_read_lengths"
            .to_owned(),
        downstream_probe_id: "benchmark.fastq.profile_read_lengths.compare".to_owned(),
        limitations: Vec::new(),
    }
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

#[test]
fn duplication_normalizer_closes_both_backend_shapes() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("bam-duplication-normalizer")?;
    let picard = temporary.path().join("picard.metrics.txt");
    let samtools = temporary.path().join("samtools.stats.txt");
    std::fs::write(
        &picard,
        b"## METRICS CLASS\nLIBRARY\tUNPAIRED_READS_EXAMINED\tREAD_PAIRS_EXAMINED\tUNPAIRED_READ_DUPLICATES\tREAD_PAIR_DUPLICATES\tPERCENT_DUPLICATION\nlib\t2\t4\t0\t1\t0.200000\n",
    )?;
    std::fs::write(&samtools, b"EXAMINED: 10\nDUPLICATE TOTAL: 2\n")?;
    let expected = "metric\tvalue\nexamined_reads\t10\nduplicate_reads\t2\nduplication_fraction\t0.200000000000\n";
    assert_eq!(normalize_duplication("picard", &picard)?, expected);
    assert_eq!(normalize_duplication("samtools", &samtools)?, expected);
    Ok(())
}

fn normalize_duplication(source: &str, input: &Path) -> anyhow::Result<String> {
    let script =
        super::support::repo_root()?.join("benchmarks/execution/bam_duplication_metrics.awk");
    let output = Command::new("awk")
        .args(["-v", "mode=summary", "-v", &format!("source={source}"), "-f"])
        .arg(script)
        .arg(input)
        .output()?;
    anyhow::ensure!(output.status.success(), "duplication AWK failed");
    Ok(String::from_utf8(output.stdout)?)
}

#[test]
fn alignment_normalizer_scores_primary_records_against_independent_truth() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("bam-align-normalizer")?;
    let truth = temporary.path().join("alignment-truth.tsv");
    let sam = temporary.path().join("align.sam");
    std::fs::write(
        &truth,
        b"read_id\tread_end\treference_name\tposition\nread-a\t1\tchr1\t10\nread-a\t2\tchr1\t40\nread-b\t1\tchr1\t70\nread-b\t2\t*\t0\n",
    )?;
    std::fs::write(
        &sam,
        b"@HD\tVN:1.6\tSO:unsorted\n@SQ\tSN:chr1\tLN:1000\nread-a\t99\tchr1\t10\t60\t20M\t=\t40\t50\tAAAAAAAAAAAAAAAAAAAA\t*\nread-a\t147\tchr1\t40\t60\t20M\t=\t10\t-50\tAAAAAAAAAAAAAAAAAAAA\t*\nread-b\t65\tchr1\t71\t60\t20M\t*\t0\t0\tAAAAAAAAAAAAAAAAAAAA\t*\nread-b\t133\t*\t0\t0\t*\t*\t0\t0\tAAAAAAAAAAAAAAAAAAAA\t*\n",
    )?;
    let script = super::support::repo_root()?.join("benchmarks/execution/bam_align.awk");
    let output = Command::new("awk")
        .args(["-v", "mode=summary", "-v"])
        .arg(format!("truth_path={}", truth.display()))
        .arg("-f")
        .arg(script)
        .arg(sam)
        .output()?;
    anyhow::ensure!(output.status.success(), "alignment AWK failed");
    assert_eq!(
        String::from_utf8(output.stdout)?,
        "metric\tvalue\ntotal_reads\t4\nmapped_reads\t3\nalignment_rate\t0.750000000000\nexpected_mapped_reads\t3\nexact_placement_reads\t2\nplacement_precision\t0.666666666667\nplacement_recall\t0.666666666667\n"
    );
    Ok(())
}

#[test]
fn insert_size_normalizer_extracts_picard_fr_distribution() -> anyhow::Result<()> {
    let repository_root = super::support::repo_root()?;
    let fixture = repository_root.join(
        "benchmarks/tests/fixtures/bench/parsers/bam/bam.insert_size/picard/raw.insert-size.metrics.txt",
    );
    let script = repository_root.join("benchmarks/execution/bam_insert_size.awk");
    let output =
        Command::new("awk").args(["-v", "mode=summary", "-f"]).arg(script).arg(fixture).output()?;
    anyhow::ensure!(output.status.success(), "insert-size AWK failed");
    assert_eq!(
        String::from_utf8(output.stdout)?,
        "metric\tvalue\nmedian_insert_size\t62.000000000000\nmean_insert_size\t64.200000000000\nstandard_deviation\t12.400000000000\nmedian_absolute_deviation\t8.000000000000\nminimum_insert_size\t30\nmaximum_insert_size\t180\nread_pairs\t1000\npair_orientation_fr_fraction\t1.000000000000\n"
    );
    Ok(())
}

#[test]
fn gc_bias_normalizer_extracts_picard_all_reads_summary() -> anyhow::Result<()> {
    let repository_root = super::support::repo_root()?;
    let fixture = repository_root.join(
        "benchmarks/tests/fixtures/bench/parsers/bam/bam.gc_bias/picard/raw.gc-bias.metrics.txt",
    );
    let script = repository_root.join("benchmarks/execution/bam_gc_bias.awk");
    let output =
        Command::new("awk").args(["-v", "mode=summary", "-f"]).arg(script).arg(fixture).output()?;
    anyhow::ensure!(output.status.success(), "GC-bias AWK failed");
    assert_eq!(
        String::from_utf8(output.stdout)?,
        "metric\tvalue\ngc_bias_score\t0.054000000000\nat_dropout\t3.200000000000\ngc_dropout\t5.400000000000\ntotal_clusters\t200000\naligned_reads\t180000\nwindows\t1200\nread_starts\t180000\n"
    );
    Ok(())
}

#[test]
fn genotyping_normalizer_derives_candidate_site_metrics() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("bam-genotyping-normalizer")?;
    let genotypes = temporary.path().join("genotypes.tsv");
    std::fs::write(
        &genotypes,
        b"chr1\t1\tA\tG\t0/0\t3\nchr1\t2\tC\tT\t0/1\t2\nchr1\t3\tG\tA\t./.\t0\n",
    )?;
    let script = super::support::repo_root()?.join("benchmarks/execution/bam_genotyping.awk");
    let output = Command::new("awk")
        .args(["-v", "mode=summary", "-f"])
        .arg(&script)
        .arg(genotypes)
        .output()?;
    anyhow::ensure!(output.status.success(), "genotyping AWK failed");
    assert_eq!(
        String::from_utf8(output.stdout)?,
        "metric\tvalue\ntotal_records\t3\ncovered_sites\t2\ncalled_genotypes\t2\nalternate_genotypes\t1\nmean_depth_at_covered_sites\t2.500000000000\n"
    );

    let summary = temporary.path().join("summary.tsv");
    std::fs::write(
        &summary,
        b"metric\tvalue\ntotal_records\t3\ncovered_sites\t2\ncalled_genotypes\t2\nalternate_genotypes\t1\nmean_depth_at_covered_sites\t2.500000000000\n",
    )?;
    let output = Command::new("awk")
        .args([
            "-v",
            "mode=metrics",
            "-v",
            "bam_input_bytes=26592222",
            "-v",
            "reference_input_bytes=2539168129",
            "-v",
            "database_input_bytes=8534981",
            "-v",
            "input_bytes=2574295332",
            "-v",
            "output_bytes=4484572",
            "-f",
        ])
        .arg(script)
        .arg(summary)
        .output()?;
    anyhow::ensure!(output.status.success(), "genotyping metrics AWK failed");
    let metrics: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(metrics["reference_input_bytes"], 2_539_168_129_u64);
    assert_eq!(metrics["input_bytes"], 2_574_295_332_u64);
    Ok(())
}
