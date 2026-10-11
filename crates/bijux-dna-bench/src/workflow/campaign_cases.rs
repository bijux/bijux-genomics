//! Owner: bijux-dna-bench
//! Generic benchmark case planning, admission, and aggregation workflows.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::{
    aggregate_benchmark_admitted_case_observation_set,
    build_benchmark_admitted_case_observation_set, ensure_benchmark_admitted_case_observation_set,
    ensure_benchmark_case_aggregation_set, ensure_benchmark_case_matrix,
    load_benchmark_admitted_case_observation_set, load_benchmark_case_matrix,
    load_stage_campaign_registry, load_stage_interface_catalog_from_repo,
    BenchmarkAdmittedCaseObservationSetV1, BenchmarkAdmittedCaseObservationV2,
    BenchmarkCaseAggregationSetV1, BenchmarkCaseMatrixRequest, BenchmarkCaseMatrixV2,
};

const MAX_CASE_INPUT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCasePlanningRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub data_index_path: PathBuf,
    pub run_id: String,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCaseAdmissionRequest {
    pub matrix_path: PathBuf,
    pub observations_path: PathBuf,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCaseAggregationRequest {
    pub matrix_path: PathBuf,
    pub observations_path: PathBuf,
    pub output_path: PathBuf,
}

/// Plan and immutably publish the complete generic case matrix for one permanent run.
///
/// # Errors
/// Returns an error when source authority, data closure, stage bindings, or output identity fails.
pub fn plan_benchmark_cases(
    request: &BenchmarkCasePlanningRequest,
) -> Result<BenchmarkCaseMatrixV2> {
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize generic benchmark source repository")?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&repository_root)
            .context("require a clean generic benchmark source repository")?;
    let campaign_path = repository_root.join("benchmarks/contracts/stage-campaigns.toml");
    let interface_path = repository_root.join("benchmarks/contracts/stage-interfaces.toml");
    let mut campaigns = load_stage_campaign_registry(&campaign_path)?;
    let mut interfaces = load_stage_interface_catalog_from_repo(&interface_path, &repository_root)?;
    select_run_authorities(&mut campaigns, &mut interfaces, &request.run_id)?;
    let bindings = bijux_dna_stage_contract::canonical_tool_bindings()?;
    let campaign =
        campaigns.campaigns.first().context("selected generic benchmark campaign is absent")?;
    let data_snapshots = crate::repo::derive_benchmark_data_snapshots(
        &request.study_root,
        &request.data_index_path,
        &campaign.corpus_catalog_ids,
        &campaign.reference_catalog_ids,
        &campaign.database_catalog_ids,
    )?;
    let binding_bytes = serde_json::to_vec(&bindings.bindings().cloned().collect::<Vec<_>>())?;
    let matrix = super::plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &repository_commit,
        campaign_registry_sha256: &bijux_dna_infra::hash_file_sha256(&campaign_path)?,
        interface_catalog_sha256: &bijux_dna_infra::hash_file_sha256(&interface_path)?,
        tool_binding_registry_sha256: &bijux_dna_infra::hash_bytes_sha256(&binding_bytes),
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &bindings,
        data_snapshots: &data_snapshots,
    })?;
    ensure_benchmark_case_matrix(&request.output_path, &matrix)?;
    Ok(matrix)
}

/// Admit normalized case observations against their exact matrix authority.
///
/// # Errors
/// Returns an error when either input drifts or immutable publication fails.
pub fn admit_benchmark_cases(
    request: &BenchmarkCaseAdmissionRequest,
) -> Result<BenchmarkAdmittedCaseObservationSetV1> {
    let matrix = load_benchmark_case_matrix(&request.matrix_path)?;
    let observations: Vec<BenchmarkAdmittedCaseObservationV2> =
        read_json(&request.observations_path, "admitted benchmark case observations")?;
    let set = build_benchmark_admitted_case_observation_set(&matrix, observations)?;
    ensure_benchmark_admitted_case_observation_set(&request.output_path, &matrix, &set)?;
    Ok(set)
}

/// Aggregate one admitted observation set and immutably publish its case summaries.
///
/// # Errors
/// Returns an error when matrix, observations, aggregation, or output identity fails.
pub fn aggregate_benchmark_cases(
    request: &BenchmarkCaseAggregationRequest,
) -> Result<BenchmarkCaseAggregationSetV1> {
    let matrix = load_benchmark_case_matrix(&request.matrix_path)?;
    let observations =
        load_benchmark_admitted_case_observation_set(&request.observations_path, &matrix)?;
    let aggregations = aggregate_benchmark_admitted_case_observation_set(&matrix, &observations)?;
    ensure_benchmark_case_aggregation_set(
        &request.output_path,
        &matrix,
        &observations,
        &aggregations,
    )?;
    Ok(aggregations)
}

fn select_run_authorities(
    campaigns: &mut bijux_dna_bench_model::BenchmarkStageCampaignRegistry,
    interfaces: &mut bijux_dna_bench_model::BenchmarkStageInterfaceCatalog,
    run_id: &str,
) -> Result<()> {
    let stage_id = campaigns
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == run_id)
        .with_context(|| format!("generic benchmark campaign is not defined: {run_id}"))?
        .stage_id
        .clone();
    campaigns.campaigns.retain(|campaign| campaign.run_id == run_id);
    interfaces.interfaces.retain(|interface| interface.stage_id == stage_id);
    if interfaces.interfaces.len() != 1 {
        bail!("generic benchmark run {run_id} does not have exactly one stage interface");
    }
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_CASE_INPUT_BYTES)
        .with_context(|| format!("read {label} {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {label} {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};

    use super::select_run_authorities;

    fn repository_root() -> Result<PathBuf> {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .context("resolve repository root from benchmark crate")
    }

    #[test]
    fn selects_one_run_without_requiring_unrelated_campaign_bindings() -> Result<()> {
        let root = repository_root()?;
        let mut campaigns = super::load_stage_campaign_registry(
            &root.join("benchmarks/contracts/stage-campaigns.toml"),
        )?;
        let mut interfaces = super::load_stage_interface_catalog_from_repo(
            &root.join("benchmarks/contracts/stage-interfaces.toml"),
            &root,
        )?;

        select_run_authorities(&mut campaigns, &mut interfaces, "run-2001")?;

        assert_eq!(campaigns.campaigns.len(), 1);
        assert_eq!(campaigns.campaigns[0].run_id, "run-2001");
        assert_eq!(interfaces.interfaces.len(), 1);
        assert_eq!(interfaces.interfaces[0].stage_id, "bam.genotyping");
        Ok(())
    }

    #[test]
    fn rejects_unknown_run_identity() -> Result<()> {
        let root = repository_root()?;
        let mut campaigns = super::load_stage_campaign_registry(
            &root.join("benchmarks/contracts/stage-campaigns.toml"),
        )?;
        let mut interfaces = super::load_stage_interface_catalog_from_repo(
            &root.join("benchmarks/contracts/stage-interfaces.toml"),
            &root,
        )?;

        assert!(select_run_authorities(&mut campaigns, &mut interfaces, "run-2999")
            .is_err_and(|error| error.to_string().contains("not defined")));
        Ok(())
    }

    #[test]
    fn selects_damage_run_with_every_supported_tool_including_external_artifact() -> Result<()> {
        let root = repository_root()?;
        let mut campaigns = super::load_stage_campaign_registry(
            &root.join("benchmarks/contracts/stage-campaigns.toml"),
        )?;
        let mut interfaces = super::load_stage_interface_catalog_from_repo(
            &root.join("benchmarks/contracts/stage-interfaces.toml"),
            &root,
        )?;

        select_run_authorities(&mut campaigns, &mut interfaces, "run-2010")?;

        let expected =
            vec!["addeam", "damageprofiler", "mapdamage2", "ngsbriggs", "pmdtools", "pydamage"];
        assert_eq!(campaigns.campaigns.len(), 1);
        assert_eq!(campaigns.campaigns[0].tool_ids, expected);
        assert_eq!(interfaces.interfaces.len(), 1);
        assert_eq!(interfaces.interfaces[0].stage_id, "bam.damage");
        assert_eq!(interfaces.interfaces[0].tool_ids, expected);

        let bindings = bijux_dna_stage_contract::canonical_tool_bindings()?;
        let binding_bytes = serde_json::to_vec(&bindings.bindings().cloned().collect::<Vec<_>>())?;
        let matrix = crate::workflow::plan_benchmark_case_matrix(
            &crate::workflow::BenchmarkCaseMatrixRequest {
                repository_commit: &"a".repeat(40),
                campaign_registry_sha256: &"b".repeat(64),
                interface_catalog_sha256: &"c".repeat(64),
                tool_binding_registry_sha256: &bijux_dna_infra::hash_bytes_sha256(&binding_bytes),
                campaigns: &campaigns,
                interfaces: &interfaces,
                bindings: &bindings,
                data_snapshots: &[
                    crate::workflow::BenchmarkDataSnapshotV1 {
                        catalog_id: "ena-prjeb22390-err2112767-ngsbriggs-input".to_string(),
                        role: crate::workflow::BenchmarkDataSnapshotRoleV1::Corpus,
                        catalog_sha256: "d".repeat(64),
                        sample_ids: vec!["ERR2112767".to_string()],
                    },
                    crate::workflow::BenchmarkDataSnapshotV1 {
                        catalog_id: "equcab3-gcf-002863925.1".to_string(),
                        role: crate::workflow::BenchmarkDataSnapshotRoleV1::Reference,
                        catalog_sha256: "e".repeat(64),
                        sample_ids: Vec::new(),
                    },
                ],
            },
        )?;
        assert_eq!(matrix.cases.len(), 6);
        assert!(matrix.cases.iter().any(|case| {
            case.tool_id == "ngsbriggs"
                && case.binding.artifact_source
                    == bijux_dna_stage_contract::ToolArtifactSourceV1::ExternalArtifact {
                        artifact_kind: "operator_provided_binary_or_apptainer_sif".to_string(),
                        version_probe: vec!["${executable}".to_string(), "--version".to_string()],
                        checksum_required: true,
                    }
        }));
        Ok(())
    }
}
