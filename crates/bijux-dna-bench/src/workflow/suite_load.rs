//! Owner: bijux-dna-bench
//! Repository-backed loading for benchmark observations.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::repo::RunRepository;
use crate::repo::{
    bench_bundles_dir, bench_corpora_dir, bench_portfolio_path, bench_stage_campaigns_path,
    bench_stage_interfaces_path,
};
use bijux_dna_bench_model::contract::{
    validate_benchmark_portfolio, validate_bundle_manifest, validate_corpus_manifest,
    validate_defined_campaign_portfolio_alignment, validate_defined_interface_campaign_alignment,
    validate_interface_parameter_alignment, validate_pipeline_parameter_config,
    validate_stage_campaign_registry, validate_stage_interface_catalog,
};
use bijux_dna_bench_model::{
    BenchmarkBundleManifest, BenchmarkCorpusManifest, BenchmarkObservation,
    BenchmarkPipelineParameterConfig, BenchmarkPortfolio, BenchmarkStageCampaignRegistry,
    BenchmarkStageInterfaceCatalog,
};

use super::parameter_authority::load_stage_parameter_authority;

const STAGE_INTERFACE_REGISTRY_PATH: &str = "benchmarks/contracts/stage-interfaces.toml";

/// Exact render-only authority used while publishing a benchmark report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkReportPresentationAuthority {
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub overlaid_stage_ids: Vec<String>,
}

/// Validated stage interfaces and the render-only authority that completed them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkPublicationInterfaceCatalog {
    pub catalog: BenchmarkStageInterfaceCatalog,
    pub presentation: BenchmarkReportPresentationAuthority,
}

/// Load observations for a suite from a repository.
///
/// # Errors
/// Returns an error if the repository cannot load observations.
pub fn load_suite(
    repo: &dyn RunRepository,
    run_ids: Option<&[String]>,
) -> Result<Vec<BenchmarkObservation>> {
    let ids = match run_ids {
        Some(ids) => ids.to_vec(),
        None => repo.list_runs()?,
    };
    let mut observations = Vec::new();
    for run_id in ids {
        observations.extend(repo.load_observations(&run_id)?);
    }
    Ok(observations)
}

/// Load and validate a single benchmark corpus manifest.
///
/// # Errors
/// Returns an error if the manifest cannot be read, parsed, or validated.
pub fn load_corpus_manifest(path: &Path) -> Result<BenchmarkCorpusManifest> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read corpus manifest {}", path.display()))?;
    let manifest: BenchmarkCorpusManifest = toml::from_str(&raw)
        .with_context(|| format!("parse corpus manifest {}", path.display()))?;
    validate_corpus_manifest(&manifest)
        .with_context(|| format!("validate corpus manifest {}", path.display()))?;
    Ok(manifest)
}

/// Load and validate all checked-in benchmark corpus manifests.
///
/// # Errors
/// Returns an error if manifests cannot be loaded or validated.
pub fn load_corpus_catalog() -> Result<Vec<BenchmarkCorpusManifest>> {
    let mut manifests = Vec::new();
    for entry in fs::read_dir(bench_corpora_dir()).context("read benchmark corpus directory")? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        manifests.push(load_corpus_manifest(&path)?);
    }
    manifests.sort_by(|a, b| a.corpus_id.cmp(&b.corpus_id));
    Ok(manifests)
}

/// Load and validate a single benchmark bundle manifest.
///
/// # Errors
/// Returns an error if the manifest cannot be read, parsed, or validated.
pub fn load_bundle_manifest(path: &Path) -> Result<BenchmarkBundleManifest> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read bundle manifest {}", path.display()))?;
    let manifest: BenchmarkBundleManifest = toml::from_str(&raw)
        .with_context(|| format!("parse bundle manifest {}", path.display()))?;
    validate_bundle_manifest(&manifest)
        .with_context(|| format!("validate bundle manifest {}", path.display()))?;
    Ok(manifest)
}

/// Load and validate all checked-in benchmark bundle manifests.
///
/// # Errors
/// Returns an error if manifests cannot be loaded or validated.
pub fn load_bundle_catalog() -> Result<Vec<BenchmarkBundleManifest>> {
    let mut manifests = Vec::new();
    for entry in fs::read_dir(bench_bundles_dir()).context("read benchmark bundle directory")? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        manifests.push(load_bundle_manifest(&path)?);
    }
    manifests.sort_by(|a, b| a.bundle_id.cmp(&b.bundle_id));
    Ok(manifests)
}

/// Load and validate the stage campaign registry.
///
/// # Errors
/// Returns an error if the registry cannot be read, parsed, or validated.
pub fn load_stage_campaign_registry(path: &Path) -> Result<BenchmarkStageCampaignRegistry> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read stage campaign registry {}", path.display()))?;
    let registry: BenchmarkStageCampaignRegistry = toml::from_str(&raw)
        .with_context(|| format!("parse stage campaign registry {}", path.display()))?;
    validate_stage_campaign_registry(&registry)
        .with_context(|| format!("validate stage campaign registry {}", path.display()))?;
    Ok(registry)
}

/// Load and validate the benchmark portfolio declaration.
///
/// # Errors
/// Returns an error if the portfolio cannot be read, parsed, or validated.
pub fn load_benchmark_portfolio(path: &Path) -> Result<BenchmarkPortfolio> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read benchmark portfolio {}", path.display()))?;
    let portfolio: BenchmarkPortfolio = toml::from_str(&raw)
        .with_context(|| format!("parse benchmark portfolio {}", path.display()))?;
    validate_benchmark_portfolio(&portfolio)
        .with_context(|| format!("validate benchmark portfolio {}", path.display()))?;
    validate_portfolio_repository_paths(&portfolio)
        .with_context(|| format!("validate benchmark portfolio paths {}", path.display()))?;
    Ok(portfolio)
}

/// Load and validate the benchmark stage-interface catalog.
///
/// # Errors
/// Returns an error if the catalog cannot be read, parsed, or validated.
pub fn load_stage_interface_catalog(path: &Path) -> Result<BenchmarkStageInterfaceCatalog> {
    let repo_root = crate::repo::resolve_repo_root().context("resolve repository root")?;
    load_stage_interface_catalog_from_repo(path, &repo_root)
}

/// Load and validate the benchmark stage-interface catalog against an explicit repository root.
///
/// # Errors
/// Returns an error if the catalog or any repository-owned contract cannot be validated.
pub fn load_stage_interface_catalog_from_repo(
    path: &Path,
    repo_root: &Path,
) -> Result<BenchmarkStageInterfaceCatalog> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read stage interface catalog {}", path.display()))?;
    let catalog: BenchmarkStageInterfaceCatalog = toml::from_str(&raw)
        .with_context(|| format!("parse stage interface catalog {}", path.display()))?;
    validate_stage_interface_catalog(&catalog)
        .with_context(|| format!("validate stage interface catalog {}", path.display()))?;
    validate_stage_interface_repository_contracts(&catalog, repo_root)
        .with_context(|| format!("validate stage interface repository paths {}", path.display()))?;
    Ok(catalog)
}

/// Load source-locked interfaces for report publication with an explicit render-only authority.
///
/// Current catalogs remain entirely source-owned. Historical v1 catalogs that predate required
/// `report_presentation` tables receive only those absent tables from the presentation repository;
/// every scientific, execution, parameter, adapter, and downstream-probe field remains sourced
/// from and validated against the historical repository.
///
/// # Errors
/// Returns an error when either authority is invalid, a historical interface is malformed, or the
/// presentation authority does not define exactly one matching stage.
pub fn load_stage_interface_catalog_for_publication(
    source_path: &Path,
    source_repo_root: &Path,
    presentation_repo_root: &Path,
) -> Result<BenchmarkPublicationInterfaceCatalog> {
    let source_raw = fs::read_to_string(source_path)
        .with_context(|| format!("read stage interface catalog {}", source_path.display()))?;
    let source_sha256 = bijux_dna_infra::hash_file_sha256(source_path)?;
    if let Ok(catalog) = toml::from_str::<BenchmarkStageInterfaceCatalog>(&source_raw) {
        validate_stage_interface_catalog(&catalog).with_context(|| {
            format!("validate stage interface catalog {}", source_path.display())
        })?;
        validate_stage_interface_repository_contracts(&catalog, source_repo_root).with_context(
            || format!("validate stage interface repository paths {}", source_path.display()),
        )?;
        return Ok(BenchmarkPublicationInterfaceCatalog {
            catalog,
            presentation: BenchmarkReportPresentationAuthority {
                repository_commit: bijux_dna_containers::api::repository_head_commit(
                    source_repo_root,
                )?,
                catalog_sha256: source_sha256,
                overlaid_stage_ids: Vec::new(),
            },
        });
    }

    let presentation_repo_root = presentation_repo_root
        .canonicalize()
        .context("canonicalize benchmark report presentation repository")?;
    let presentation_path = presentation_repo_root.join(STAGE_INTERFACE_REGISTRY_PATH);
    let presentation_catalog =
        load_stage_interface_catalog_from_repo(&presentation_path, &presentation_repo_root)
            .context("load benchmark report presentation authority")?;
    let presentations = presentation_catalog
        .interfaces
        .iter()
        .map(|interface| (interface.stage_id.as_str(), &interface.report_presentation))
        .collect::<BTreeMap<_, _>>();

    let mut source_value: toml::Value = toml::from_str(&source_raw).with_context(|| {
        format!("parse historical stage interface catalog {}", source_path.display())
    })?;
    let interfaces = source_value
        .get_mut("interfaces")
        .and_then(toml::Value::as_array_mut)
        .context("historical stage interface catalog omits its interface array")?;
    let mut overlaid_stage_ids = Vec::new();
    for interface in interfaces {
        let table =
            interface.as_table_mut().context("historical stage interface entry is not a table")?;
        if table.contains_key("report_presentation") {
            continue;
        }
        let stage_id = table
            .get("stage_id")
            .and_then(toml::Value::as_str)
            .context("historical stage interface omits its stage ID")?
            .to_string();
        let presentation = presentations.get(stage_id.as_str()).with_context(|| {
            format!("presentation authority omits historical benchmark stage {stage_id}")
        })?;
        let value = toml::Value::try_from(*presentation)
            .with_context(|| format!("encode report presentation for stage {stage_id}"))?;
        table.insert("report_presentation".to_string(), value);
        overlaid_stage_ids.push(stage_id);
    }
    if overlaid_stage_ids.is_empty() {
        bail!("stage interface catalog is invalid for reasons other than absent presentation metadata");
    }

    let catalog: BenchmarkStageInterfaceCatalog = source_value.try_into().with_context(|| {
        format!("parse completed stage interface catalog {}", source_path.display())
    })?;
    validate_stage_interface_catalog(&catalog).with_context(|| {
        format!("validate completed stage interface catalog {}", source_path.display())
    })?;
    validate_stage_interface_repository_contracts(&catalog, source_repo_root).with_context(
        || {
            format!(
                "validate historical stage interface repository paths {}",
                source_path.display()
            )
        },
    )?;
    overlaid_stage_ids.sort();
    Ok(BenchmarkPublicationInterfaceCatalog {
        catalog,
        presentation: BenchmarkReportPresentationAuthority {
            repository_commit: bijux_dna_containers::api::repository_head_commit(
                &presentation_repo_root,
            )?,
            catalog_sha256: bijux_dna_infra::hash_file_sha256(&presentation_path)?,
            overlaid_stage_ids,
        },
    })
}

/// Load the checked-in campaign and portfolio contracts and validate all planning inputs.
///
/// # Errors
/// Returns an error if the checked-in contracts cannot be loaded or drift from each other.
pub fn load_checked_in_benchmark_contracts(
) -> Result<(BenchmarkStageCampaignRegistry, BenchmarkPortfolio)> {
    let (registry, portfolio, _) = load_checked_in_benchmark_planning_contracts()?;
    Ok((registry, portfolio))
}

/// Load the complete checked-in benchmark planning contract set and validate its alignment.
///
/// # Errors
/// Returns an error if the checked-in contracts cannot be loaded or drift from each other.
pub fn load_checked_in_benchmark_planning_contracts(
) -> Result<(BenchmarkStageCampaignRegistry, BenchmarkPortfolio, BenchmarkStageInterfaceCatalog)> {
    let registry = load_stage_campaign_registry(&bench_stage_campaigns_path())?;
    let portfolio = load_benchmark_portfolio(&bench_portfolio_path())?;
    let interfaces = load_stage_interface_catalog(&bench_stage_interfaces_path())?;
    let repo_root = crate::repo::resolve_repo_root().context("resolve repository root")?;
    for interface in &interfaces.interfaces {
        load_stage_parameter_authority(&repo_root, &interface.stage_id)
            .with_context(|| format!("validate parameter registry for {}", interface.stage_id))?;
    }
    validate_defined_campaign_portfolio_alignment(&registry, &portfolio)
        .context("validate defined benchmark campaigns against portfolio reservations")?;
    validate_defined_interface_campaign_alignment(&interfaces, &registry)
        .context("validate benchmark interfaces against defined campaigns")?;
    Ok((registry, portfolio, interfaces))
}

fn validate_stage_interface_repository_contracts(
    catalog: &BenchmarkStageInterfaceCatalog,
    repo_root: &Path,
) -> Result<()> {
    let mut parameter_configs = std::collections::BTreeMap::new();
    for interface in &catalog.interfaces {
        for relative in std::iter::once(interface.normalized_result_contract.as_str())
            .chain(interface.planner_contract.as_deref())
            .chain(std::iter::once(interface.downstream_probe.source.as_str()))
            .chain(interface.provenance_sources.iter().map(String::as_str))
        {
            let path = repo_root.join(relative);
            if !path.is_file() {
                anyhow::bail!(
                    "benchmark stage interface source is absent for {}: {}",
                    interface.stage_id,
                    path.display()
                );
            }
        }

        if !parameter_configs.contains_key(&interface.parameter_config) {
            let path = repo_root.join(&interface.parameter_config);
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("read pipeline parameter config {}", path.display()))?;
            let config: BenchmarkPipelineParameterConfig = serde_yaml::from_str(&raw)
                .with_context(|| format!("parse pipeline parameter config {}", path.display()))?;
            validate_pipeline_parameter_config(&config).with_context(|| {
                format!("validate pipeline parameter config {}", path.display())
            })?;
            parameter_configs.insert(interface.parameter_config.clone(), config);
        }
        let config = parameter_configs
            .get(&interface.parameter_config)
            .context("loaded pipeline parameter config is absent from cache")?;
        validate_interface_parameter_alignment(interface, config)
            .with_context(|| format!("validate parameter ownership for {}", interface.stage_id))?;
    }
    Ok(())
}

fn validate_portfolio_repository_paths(portfolio: &BenchmarkPortfolio) -> Result<()> {
    let repo_root = crate::repo::resolve_repo_root().context("resolve repository root")?;
    for corpus in &portfolio.corpora {
        for source_spec in &corpus.source_specs {
            let source_path = Path::new(source_spec);
            if source_path.is_absolute() {
                anyhow::bail!("benchmark source spec must be repository-relative: {source_spec}");
            }
            let path = repo_root.join(source_path);
            if !path.is_file() {
                anyhow::bail!("benchmark source spec is absent: {}", path.display());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use bijux_dna_bench_model::{BenchmarkEvidenceTier, CorpusDomain, CorpusScale};

    use super::{
        load_bundle_catalog, load_checked_in_benchmark_planning_contracts, load_corpus_catalog,
    };

    #[test]
    fn corpus_catalog_contains_fastq_ci_small_fixture_matrix() -> anyhow::Result<()> {
        let catalog = load_corpus_catalog()?;
        let has_fastq_ci = catalog.iter().any(|manifest| {
            manifest.domain == CorpusDomain::Fastq
                && manifest.scale == CorpusScale::CiSmall
                && manifest.datasets.len() >= 8
        });
        assert!(
            has_fastq_ci,
            "checked-in corpus catalog must include fastq ci-small coverage with at least 8 datasets"
        );
        Ok(())
    }

    #[test]
    fn bundle_catalog_contains_scientific_caveat_bundle() -> anyhow::Result<()> {
        let catalog = load_bundle_catalog()?;
        assert!(
            catalog.iter().any(|bundle| !bundle.scientific_caveats.is_empty()),
            "checked-in bundle catalog must include at least one caveated bundle"
        );
        Ok(())
    }

    #[test]
    fn checked_in_benchmark_contracts_include_run_2042_and_71_stage_reservations(
    ) -> anyhow::Result<()> {
        let (registry, portfolio, interfaces) = load_checked_in_benchmark_planning_contracts()?;
        assert_eq!(
            portfolio.stage_runs.len(),
            71,
            "checked-in benchmark portfolio must reserve all 71 stage run ids"
        );
        assert!(
            registry.campaigns.len() >= 7,
            "checked-in benchmark registry must include the current seed campaign set"
        );
        let run_2042 = registry
            .campaigns
            .iter()
            .find(|campaign| campaign.run_id == "run-2042")
            .context("find run-2042 stage campaign")?;
        assert_eq!(run_2042.stage_id, "fastq.profile_read_lengths");
        assert!(matches!(run_2042.evidence_tier, BenchmarkEvidenceTier::Comparative));
        assert_eq!(run_2042.tool_ids, vec!["seqfu".to_string(), "seqkit".to_string()]);
        assert_eq!(interfaces.interfaces.len(), 9);
        let validation = interfaces
            .interfaces
            .iter()
            .find(|interface| interface.stage_id == "fastq.validate_reads")
            .context("find FASTQ validation stage interface")?;
        assert_eq!(
            validation.tool_ids,
            ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"]
        );
        Ok(())
    }
}
