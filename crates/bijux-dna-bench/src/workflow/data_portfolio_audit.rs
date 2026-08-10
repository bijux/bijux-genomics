//! Owner: bijux-dna-bench
//! Deterministic audit of portfolio declarations against governed STUDY data.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_data_index, validate_benchmark_data_portfolio_audit,
    validate_data_fitness_portfolio_alignment, BenchmarkCorpusClass, BenchmarkCorpusDataAudit,
    BenchmarkCorpusStatus, BenchmarkDataIndex, BenchmarkDataIndexRecord,
    BenchmarkDataPortfolioAudit, BenchmarkDataPortfolioAuditSummary,
    BenchmarkDataSourceSpecIdentity, BenchmarkEligibility, BenchmarkEvidenceScope,
    BenchmarkGovernedDataBinding, BenchmarkUnboundDataAsset, DataIntegrityStatus,
    BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2, BENCHMARK_DATA_SOURCE_SCHEMA_V1,
    FASTQ_CORPUS_SOURCE_SCHEMA_V1,
};
use serde::Deserialize;

use crate::{
    load_benchmark_data_fitness_rubric, load_benchmark_data_source_spec, load_benchmark_portfolio,
    load_fastq_corpus_source_spec,
};

const PORTFOLIO_PATH: &str = "benchmarks/contracts/portfolio.toml";
const DATA_FITNESS_PATH: &str = "benchmarks/contracts/data-fitness.toml";
const MAX_AUTHORITY_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Deserialize)]
struct DataCatalogAuditMetadata {
    sample_count: Option<u64>,
    samples: Option<Vec<serde::de::IgnoredAny>>,
    sample_accession: Option<String>,
    accession: Option<String>,
    access_terms: Option<String>,
}

#[derive(Deserialize)]
struct SourceSpecHeader {
    schema_version: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AuditableDataCatalog {
    Preserved { preserved_catalog: DataCatalogAuditMetadata },
    Native(DataCatalogAuditMetadata),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkDataPortfolioAuditRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub data_index_path: PathBuf,
    pub source_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkDataPortfolioAuditPublicationRequest {
    pub audit: BenchmarkDataPortfolioAuditRequest,
    pub json_path: PathBuf,
    pub markdown_path: PathBuf,
}

/// Immutably publish the typed audit and its human-readable projection.
///
/// # Errors
/// Returns an error when derivation fails, either destination exists, or atomic publication fails.
pub fn publish_benchmark_data_portfolio_audit(
    request: &BenchmarkDataPortfolioAuditPublicationRequest,
) -> Result<BenchmarkDataPortfolioAudit> {
    for path in [&request.json_path, &request.markdown_path] {
        if path.exists() || path.is_symlink() {
            bail!("refusing to replace data-portfolio audit: {}", path.display());
        }
    }
    let audit = derive_benchmark_data_portfolio_audit(&request.audit)?;
    bijux_dna_infra::atomic_create_json(&request.json_path, &audit)
        .with_context(|| format!("publish data-portfolio audit {}", request.json_path.display()))?;
    let markdown = render_benchmark_data_portfolio_audit_markdown(&audit);
    bijux_dna_infra::atomic_create_bytes(&request.markdown_path, markdown.as_bytes())
        .with_context(|| {
            format!("publish data-portfolio audit {}", request.markdown_path.display())
        })?;
    Ok(audit)
}

/// Derive a fail-closed data portfolio audit without changing admission state.
///
/// # Errors
/// Returns an error when repository, STUDY, portfolio, index, catalog, or source-spec authorities
/// are malformed, escape their roots, drift from recorded digests, or produce an invalid audit.
pub fn derive_benchmark_data_portfolio_audit(
    request: &BenchmarkDataPortfolioAuditRequest,
) -> Result<BenchmarkDataPortfolioAudit> {
    validate_commit(&request.source_commit)?;
    let repository_root = canonical_directory(&request.repository_root, "repository root")?;
    let study_root = canonical_directory(&request.study_root, "STUDY root")?;
    let portfolio_path = governed_file(
        &repository_root,
        &repository_root.join(PORTFOLIO_PATH),
        "benchmark portfolio",
    )?;
    let data_fitness_path = governed_file(
        &repository_root,
        &repository_root.join(DATA_FITNESS_PATH),
        "benchmark data-fitness rubric",
    )?;
    let data_index_path = governed_file(&study_root, &request.data_index_path, "data index")?;
    let portfolio = load_benchmark_portfolio(&portfolio_path)?;
    let data_fitness = load_benchmark_data_fitness_rubric(&data_fitness_path)?;
    validate_data_fitness_portfolio_alignment(&data_fitness, &portfolio)
        .context("validate benchmark data-fitness rubric against portfolio profiles")?;
    let index_bytes = read_bounded(&data_index_path, "data index")?;
    let index: BenchmarkDataIndex = serde_json::from_slice(&index_bytes)
        .with_context(|| format!("parse data index {}", data_index_path.display()))?;
    validate_benchmark_data_index(&index).context("validate governed data index")?;
    let records =
        index.records.iter().map(|record| (record.id.as_str(), record)).collect::<BTreeMap<_, _>>();
    let mut referenced = BTreeSet::new();
    let mut corpora = Vec::with_capacity(portfolio.corpora.len());
    let mut publishable_contamination = Vec::new();

    for corpus in &portfolio.corpora {
        let source_specs = source_spec_identities(
            &repository_root,
            &corpus.source_specs,
            &corpus.id,
            corpus.class,
            &corpus.roles,
            &corpus.truth_strategy,
            &corpus.data_types,
        )?;
        let mut bindings = Vec::with_capacity(corpus.catalog_ids.len());
        let mut blockers = Vec::new();
        for catalog_id in &corpus.catalog_ids {
            referenced.insert(catalog_id.as_str());
            let binding = match records.get(catalog_id.as_str()) {
                Some(record) => binding_from_record(&study_root, record)?,
                None => missing_binding(catalog_id),
            };
            if !binding.present {
                blockers.push(format!("governed data index lacks catalog {catalog_id}"));
            } else if !binding.integrity_verified || !binding.benchmark_eligible {
                blockers
                    .push(format!("catalog {catalog_id} is not verified and benchmark-eligible"));
            }
            if corpus.status == BenchmarkCorpusStatus::Admitted
                && records
                    .get(catalog_id.as_str())
                    .is_some_and(|record| !nonproduction_reasons(record).is_empty())
            {
                publishable_contamination.push(format!("{}:{catalog_id}", corpus.id));
                blockers.push(format!(
                    "catalog {catalog_id} is suspected toy, fixture, synthetic, or smoke data"
                ));
            }
            bindings.push(binding);
        }
        bindings.sort_by(|left, right| left.catalog_id.cmp(&right.catalog_id));
        match corpus.status {
            BenchmarkCorpusStatus::Missing => {
                blockers.push("corpus source selection is missing".to_string());
            }
            BenchmarkCorpusStatus::SourceDefined => {
                blockers.push("source-defined corpus has not been admitted".to_string());
            }
            BenchmarkCorpusStatus::Admitted => {
                let observed =
                    bindings.iter().filter_map(|binding| binding.observed_sample_count).max();
                if observed.is_none() {
                    blockers.push(
                        "admitted corpus lacks an authoritative observed sample count".to_string(),
                    );
                } else if observed.is_some_and(|count| count < u64::from(corpus.sample_count)) {
                    blockers.push(
                        "observed sample count is below the admitted declaration".to_string(),
                    );
                }
            }
        }
        if corpus.catalog_ids.is_empty() && corpus.status == BenchmarkCorpusStatus::Admitted {
            blockers.push("admitted corpus has no governed catalog binding".to_string());
        }
        blockers.sort();
        blockers.dedup();
        let verified_storage_bytes = bindings.iter().try_fold(0_u64, |total, binding| {
            total.checked_add(binding.verified_size_bytes).context("corpus byte total overflow")
        })?;
        let projected_storage_bytes =
            sum_projection(source_specs.iter().map(|source| source.projected_storage_bytes))?
                .or_else(|| {
                    (corpus.status == BenchmarkCorpusStatus::Admitted)
                        .then_some(verified_storage_bytes)
                });
        let projected_cpu_hours =
            sum_projection(source_specs.iter().map(|source| source.projected_cpu_hours))?;
        corpora.push(BenchmarkCorpusDataAudit {
            corpus_id: corpus.id.clone(),
            status: corpus.status,
            evidence_scope: corpus.evidence_scope,
            class: corpus.class,
            declared_sample_count: corpus.sample_count,
            minimum_sample_count: corpus.minimum_sample_count.unwrap_or(corpus.sample_count),
            data_types: corpus.data_types.clone(),
            truth_roles: corpus.roles.clone(),
            truth_strategy: corpus.truth_strategy.clone(),
            source_specs,
            catalog_bindings: bindings,
            verified_storage_bytes,
            projected_storage_bytes,
            projected_cpu_hours,
            publishable_data_ready: corpus.status == BenchmarkCorpusStatus::Admitted
                && blockers.is_empty(),
            blockers,
        });
    }
    corpora.sort_by(|left, right| left.corpus_id.cmp(&right.corpus_id));
    publishable_contamination.sort();
    let mut unbound_assets = index
        .records
        .iter()
        .filter(|record| !referenced.contains(record.id.as_str()))
        .map(unbound_asset)
        .collect::<Result<Vec<_>>>()?;
    unbound_assets.sort_by(|left, right| left.catalog_id.cmp(&right.catalog_id));
    let summary = summarize(&corpora, &unbound_assets, publishable_contamination.len())?;
    let audit = BenchmarkDataPortfolioAudit {
        schema_version: BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2.to_string(),
        source_commit: request.source_commit.clone(),
        portfolio_path: PORTFOLIO_PATH.to_string(),
        portfolio_sha256: bijux_dna_infra::hash_file_sha256(&portfolio_path)?,
        data_index_path: stable_study_path(&study_root, &data_index_path)?,
        data_index_sha256: bijux_dna_infra::hash_bytes_sha256(&index_bytes),
        summary,
        corpora,
        unbound_assets,
        publishable_contamination,
    };
    validate_benchmark_data_portfolio_audit(&audit).context("validate data-portfolio audit")?;
    Ok(audit)
}

/// Render the operator-readable data audit without weakening the typed JSON authority.
#[must_use]
pub fn render_benchmark_data_portfolio_audit_markdown(
    audit: &BenchmarkDataPortfolioAudit,
) -> String {
    let mut lines = vec![
        "# Benchmark data portfolio audit".to_string(),
        String::new(),
        "This report joins portfolio declarations to checksum-verified STUDY records. Data-index"
            .to_string(),
        "eligibility proves governed file identity; it does not by itself admit comparative science."
            .to_string(),
        String::new(),
        format!("- Source commit: `{}`", audit.source_commit),
        format!("- Portfolio corpora: **{}**", audit.summary.corpus_count),
        format!(
            "- Publishable-data ready: **{}**",
            audit.summary.publishable_data_ready_count
        ),
        format!("- Bound governed catalogs: **{}**", audit.summary.bound_catalog_count),
        format!("- Unbound governed catalogs: **{}**", audit.summary.unbound_catalog_count),
        format!("- Verified bound bytes: **{}**", audit.summary.verified_bound_size_bytes),
        format!(
            "- Corpora without storage projections: **{}**",
            audit.summary.unknown_storage_projection_count
        ),
        format!(
            "- Corpora without compute projections: **{}**",
            audit.summary.unknown_compute_projection_count
        ),
        format!(
            "- Publishable contamination findings: **{}**",
            audit.summary.publishable_contamination_count
        ),
        String::new(),
        "## Corpus inventory and projections".to_string(),
        String::new(),
        "| Corpus | Class | State | Samples | Catalogs | Verified bytes | Projected bytes | Projected CPU hours | Ready | Blockers |".to_string(),
        "|---|---|---|---:|---:|---:|---:|---:|---|---|".to_string(),
    ];
    for corpus in &audit.corpora {
        let blockers = if corpus.blockers.is_empty() {
            "-".to_string()
        } else {
            markdown_cell(&corpus.blockers.join("; "))
        };
        lines.push(format!(
            "| `{}` | {} | {} | {}/{} | {} | {} | {} | {} | {} | {} |",
            corpus.corpus_id,
            corpus.class.as_str(),
            corpus_status(corpus.status),
            corpus.declared_sample_count,
            corpus.minimum_sample_count,
            corpus.catalog_bindings.len(),
            corpus.verified_storage_bytes,
            optional_u64(corpus.projected_storage_bytes),
            optional_u64(corpus.projected_cpu_hours),
            if corpus.publishable_data_ready { "yes" } else { "no" },
            blockers,
        ));
    }
    lines.extend([
        String::new(),
        "## Truth-role matrix".to_string(),
        String::new(),
        "| Corpus | Evidence scope | Data types | Truth roles | Truth strategy |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ]);
    for corpus in &audit.corpora {
        lines.push(format!(
            "| `{}` | {} | {} | {} | {} |",
            corpus.corpus_id,
            evidence_scope(corpus.evidence_scope),
            markdown_cell(&corpus.data_types.join(", ")),
            markdown_cell(&corpus.truth_roles.join(", ")),
            markdown_cell(&corpus.truth_strategy),
        ));
    }
    lines.extend([
        String::new(),
        "## Bound catalog access matrix".to_string(),
        String::new(),
        "Access terms are copied from each exact governed catalog. Legacy null or absent terms are reported explicitly rather than inferred.".to_string(),
        String::new(),
        "| Corpus | Catalog | Present | Access | Redistribution | Access terms | Observed samples | Sample-count authority |".to_string(),
        "|---|---|---|---|---|---|---:|---|".to_string(),
    ]);
    for corpus in &audit.corpora {
        for binding in &corpus.catalog_bindings {
            lines.push(format!(
                "| `{}` | `{}` | {} | {} | {} | {} | {} | {} |",
                corpus.corpus_id,
                binding.catalog_id,
                if binding.present { "yes" } else { "no" },
                optional_text(binding.access_classification.as_deref()),
                optional_text(binding.redistribution_classification.as_deref()),
                optional_text(binding.access_terms.as_deref()),
                optional_u64(binding.observed_sample_count),
                optional_text(binding.sample_count_authority.as_deref()),
            ));
        }
    }
    lines.extend([
        String::new(),
        "## Unbound governed assets".to_string(),
        String::new(),
        "These records are checksum-verified holdings but are not portfolio admission evidence."
            .to_string(),
        String::new(),
        "| Catalog | Access | Redistribution | Files | Bytes | Non-production suspicion | Reasons |"
            .to_string(),
        "|---|---|---|---:|---:|---|---|".to_string(),
    ]);
    for asset in &audit.unbound_assets {
        lines.push(format!(
            "| `{}` | {} | {} | {} | {} | {} | {} |",
            asset.catalog_id,
            asset.access_classification,
            asset.redistribution_classification,
            asset.verified_file_count,
            asset.verified_size_bytes,
            if asset.suspected_nonproduction { "yes" } else { "no" },
            if asset.classification_reasons.is_empty() {
                "-".to_string()
            } else {
                markdown_cell(&asset.classification_reasons.join(", "))
            },
        ));
    }
    lines.extend([
        String::new(),
        "## Publishable contamination findings".to_string(),
        String::new(),
    ]);
    if audit.publishable_contamination.is_empty() {
        lines.push("None.".to_string());
    } else {
        lines.extend(
            audit
                .publishable_contamination
                .iter()
                .map(|finding| format!("- `{}`", markdown_cell(finding))),
        );
    }
    lines.push(String::new());
    lines.join("\n")
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "unknown".to_string(), |value| value.to_string())
}

fn optional_text(value: Option<&str>) -> String {
    value.map_or_else(|| "not available".to_string(), markdown_cell)
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace(['\r', '\n'], " ")
}

fn binding_from_record(
    study_root: &Path,
    record: &BenchmarkDataIndexRecord,
) -> Result<BenchmarkGovernedDataBinding> {
    let catalog_path = resolve_study_uri(study_root, &record.catalog_path)?;
    let bytes = read_bounded(&catalog_path, "data catalog")?;
    if bijux_dna_infra::hash_bytes_sha256(&bytes) != record.catalog_sha256 {
        bail!("data catalog digest differs from index: {}", record.id);
    }
    let catalog: AuditableDataCatalog = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse data catalog {}", catalog_path.display()))?;
    let catalog = match &catalog {
        AuditableDataCatalog::Preserved { preserved_catalog } => preserved_catalog,
        AuditableDataCatalog::Native(catalog) => catalog,
    };
    let (observed_sample_count, sample_count_authority) = observed_sample_count(catalog)?;
    let access_terms = catalog_access_terms(catalog)?;
    let mut file_roles = record.files.iter().map(|file| file.role.clone()).collect::<Vec<_>>();
    file_roles.sort();
    file_roles.dedup();
    Ok(BenchmarkGovernedDataBinding {
        catalog_id: record.id.clone(),
        present: true,
        catalog_path: Some(record.catalog_path.clone()),
        catalog_sha256: Some(record.catalog_sha256.clone()),
        access_classification: Some(record.access_classification.clone()),
        redistribution_classification: Some(record.redistribution_classification.clone()),
        access_terms: Some(access_terms),
        integrity_verified: record.integrity_status == DataIntegrityStatus::Verified,
        benchmark_eligible: record.benchmark_eligibility == BenchmarkEligibility::Eligible,
        verified_file_count: u64::try_from(record.files.len()).context("file count overflow")?,
        verified_size_bytes: record.files.iter().try_fold(0_u64, |total, file| {
            total.checked_add(file.size_bytes).context("file size overflow")
        })?,
        observed_sample_count,
        sample_count_authority,
        file_roles,
    })
}

fn missing_binding(catalog_id: &str) -> BenchmarkGovernedDataBinding {
    BenchmarkGovernedDataBinding {
        catalog_id: catalog_id.to_string(),
        present: false,
        catalog_path: None,
        catalog_sha256: None,
        access_classification: None,
        redistribution_classification: None,
        access_terms: None,
        integrity_verified: false,
        benchmark_eligible: false,
        verified_file_count: 0,
        verified_size_bytes: 0,
        observed_sample_count: None,
        sample_count_authority: None,
        file_roles: Vec::new(),
    }
}

fn source_spec_identities(
    repository_root: &Path,
    paths: &[String],
    corpus_id: &str,
    class: BenchmarkCorpusClass,
    truth_roles: &[String],
    truth_strategy: &str,
    data_types: &[String],
) -> Result<Vec<BenchmarkDataSourceSpecIdentity>> {
    let mut identities = Vec::with_capacity(paths.len());
    for relative in paths {
        let path = governed_file(repository_root, &repository_root.join(relative), "source spec")?;
        let bytes = read_bounded(&path, "source spec")?;
        let header: SourceSpecHeader = toml::from_str(
            std::str::from_utf8(&bytes).context("source specification is not UTF-8")?,
        )
        .with_context(|| format!("parse source specification header {}", path.display()))?;
        let policy = match header.schema_version.as_str() {
            FASTQ_CORPUS_SOURCE_SCHEMA_V1 => {
                let spec = load_fastq_corpus_source_spec(&path)?;
                ensure_source_identity(
                    corpus_id,
                    class,
                    truth_roles,
                    truth_strategy,
                    &spec.portfolio_id,
                    &spec.policy,
                )?;
                spec.policy
            }
            BENCHMARK_DATA_SOURCE_SCHEMA_V1 => {
                let spec = load_benchmark_data_source_spec(&path)?;
                ensure_source_identity(
                    corpus_id,
                    class,
                    truth_roles,
                    truth_strategy,
                    &spec.portfolio_id,
                    &spec.policy,
                )?;
                if spec.data_types != data_types {
                    bail!(
                        "benchmark data source specification disagrees with portfolio data types for {corpus_id}"
                    );
                }
                spec.policy
            }
            schema => bail!("unsupported benchmark source specification schema {schema}"),
        };
        identities.push(BenchmarkDataSourceSpecIdentity {
            path: relative.clone(),
            sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
            projected_storage_bytes: Some(policy.projected_storage_bytes),
            projected_cpu_hours: Some(policy.projected_cpu_hours),
        });
    }
    identities.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(identities)
}

fn ensure_source_identity(
    corpus_id: &str,
    class: BenchmarkCorpusClass,
    truth_roles: &[String],
    truth_strategy: &str,
    source_portfolio_id: &str,
    policy: &bijux_dna_bench_model::CorpusSourcePolicy,
) -> Result<()> {
    if source_portfolio_id != corpus_id
        || policy.class != class
        || policy.truth_roles != truth_roles
        || policy.truth_strategy != truth_strategy
    {
        bail!("source specification disagrees with portfolio corpus {corpus_id}");
    }
    Ok(())
}

fn observed_sample_count(
    catalog: &DataCatalogAuditMetadata,
) -> Result<(Option<u64>, Option<String>)> {
    if let Some(count) = catalog.sample_count {
        return Ok((Some(count), Some("catalog.sample_count".to_string())));
    }
    if let Some(samples) = &catalog.samples {
        return Ok((
            Some(u64::try_from(samples.len()).context("sample count overflow")?),
            Some("catalog.samples".to_string()),
        ));
    }
    if catalog.sample_accession.is_some() || catalog.accession.is_some() {
        return Ok((Some(1), Some("catalog.sample_identity".to_string())));
    }
    Ok((None, None))
}

fn catalog_access_terms(catalog: &DataCatalogAuditMetadata) -> Result<String> {
    match &catalog.access_terms {
        Some(value) if !value.trim().is_empty() => Ok(value.clone()),
        None => Ok("not declared by governed catalog".to_string()),
        Some(_) => bail!("data catalog access_terms must be a non-empty string or null"),
    }
}

fn unbound_asset(record: &BenchmarkDataIndexRecord) -> Result<BenchmarkUnboundDataAsset> {
    let reasons = nonproduction_reasons(record);
    Ok(BenchmarkUnboundDataAsset {
        catalog_id: record.id.clone(),
        purpose: record.purpose.clone(),
        access_classification: record.access_classification.clone(),
        redistribution_classification: record.redistribution_classification.clone(),
        verified_file_count: u64::try_from(record.files.len()).context("file count overflow")?,
        verified_size_bytes: record.files.iter().try_fold(0_u64, |total, file| {
            total.checked_add(file.size_bytes).context("file size overflow")
        })?,
        suspected_nonproduction: !reasons.is_empty(),
        classification_reasons: reasons,
    })
}

fn nonproduction_reasons(record: &BenchmarkDataIndexRecord) -> Vec<String> {
    let basis = format!(
        "{} {} {} {} {}",
        record.id,
        record.purpose,
        record.catalog_path,
        record.source_identities.join(" "),
        record.known_limitations.join(" "),
    )
    .to_ascii_lowercase();
    ["toy", "fixture", "synthetic", "smoke"]
        .into_iter()
        .filter(|token| {
            basis
                .split(|character: char| !character.is_ascii_alphanumeric())
                .any(|word| word == *token)
        })
        .map(|token| format!("identity_or_purpose_contains_{token}"))
        .collect()
}

fn summarize(
    corpora: &[BenchmarkCorpusDataAudit],
    unbound: &[BenchmarkUnboundDataAsset],
    contamination: usize,
) -> Result<BenchmarkDataPortfolioAuditSummary> {
    let bindings = corpora.iter().flat_map(|corpus| &corpus.catalog_bindings);
    let mut seen = BTreeSet::new();
    let unique_bindings =
        bindings.filter(|binding| seen.insert(binding.catalog_id.as_str())).collect::<Vec<_>>();
    let bound_catalog_count = unique_bindings.iter().filter(|binding| binding.present).count();
    let missing_catalog_binding_count =
        unique_bindings.iter().filter(|binding| !binding.present).count();
    let verified_bound_file_count = unique_bindings.iter().try_fold(0_u64, |total, binding| {
        total.checked_add(binding.verified_file_count).context("file count overflow")
    })?;
    let verified_bound_size_bytes = unique_bindings.iter().try_fold(0_u64, |total, binding| {
        total.checked_add(binding.verified_size_bytes).context("file size overflow")
    })?;
    Ok(BenchmarkDataPortfolioAuditSummary {
        corpus_count: corpora.len().try_into().context("corpus count overflow")?,
        admitted_corpus_count: count_corpora(corpora, BenchmarkCorpusStatus::Admitted)?,
        source_defined_corpus_count: count_corpora(corpora, BenchmarkCorpusStatus::SourceDefined)?,
        missing_corpus_count: count_corpora(corpora, BenchmarkCorpusStatus::Missing)?,
        publishable_data_ready_count: corpora
            .iter()
            .filter(|corpus| corpus.publishable_data_ready)
            .count()
            .try_into()
            .context("ready count overflow")?,
        bound_catalog_count: bound_catalog_count.try_into().context("binding count overflow")?,
        missing_catalog_binding_count: missing_catalog_binding_count
            .try_into()
            .context("binding count overflow")?,
        unbound_catalog_count: unbound.len().try_into().context("unbound count overflow")?,
        verified_bound_file_count,
        verified_bound_size_bytes,
        unknown_storage_projection_count: corpora
            .iter()
            .filter(|corpus| corpus.projected_storage_bytes.is_none())
            .count()
            .try_into()
            .context("projection count overflow")?,
        unknown_compute_projection_count: corpora
            .iter()
            .filter(|corpus| corpus.projected_cpu_hours.is_none())
            .count()
            .try_into()
            .context("projection count overflow")?,
        suspected_nonproduction_asset_count: unbound
            .iter()
            .filter(|asset| asset.suspected_nonproduction)
            .count()
            .try_into()
            .context("non-production count overflow")?,
        publishable_contamination_count: contamination
            .try_into()
            .context("contamination count overflow")?,
    })
}

fn count_corpora(
    corpora: &[BenchmarkCorpusDataAudit],
    status: BenchmarkCorpusStatus,
) -> Result<u64> {
    corpora
        .iter()
        .filter(|corpus| corpus.status == status)
        .count()
        .try_into()
        .context("corpus count overflow")
}

fn sum_projection(values: impl Iterator<Item = Option<u64>>) -> Result<Option<u64>> {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() || values.iter().any(Option::is_none) {
        return Ok(None);
    }
    values
        .into_iter()
        .flatten()
        .try_fold(0_u64, u64::checked_add)
        .map(Some)
        .context("source-spec projection total overflow")
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a non-symlink directory: {}", path.display());
    }
    path.canonicalize().with_context(|| format!("canonicalize {label}"))
}

fn governed_file(root: &Path, path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular non-symlink file: {}", path.display());
    }
    let path = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    path.strip_prefix(root).with_context(|| format!("{label} escapes authority root"))?;
    Ok(path)
}

fn read_bounded(path: &Path, label: &str) -> Result<Vec<u8>> {
    let size = fs::metadata(path)?.len();
    if size > MAX_AUTHORITY_BYTES {
        bail!("{label} exceeds bounded authority size: {}", path.display());
    }
    fs::read(path).with_context(|| format!("read {label} {}", path.display()))
}

fn resolve_study_uri(study_root: &Path, value: &str) -> Result<PathBuf> {
    let relative = value.strip_prefix("study://").context("catalog path is not a study URI")?;
    governed_file(study_root, &study_root.join(relative), "data catalog")
}

fn stable_study_path(study_root: &Path, path: &Path) -> Result<String> {
    let relative = path.strip_prefix(study_root).context("data index escapes STUDY root")?;
    Ok(format!("study://{}", relative.to_string_lossy()))
}

fn validate_commit(value: &str) -> Result<()> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("data-portfolio source commit is not a full Git identity");
    }
    Ok(())
}

fn evidence_scope(scope: BenchmarkEvidenceScope) -> &'static str {
    match scope {
        BenchmarkEvidenceScope::Qualification => "qualification",
        BenchmarkEvidenceScope::Comparative => "comparative",
    }
}

fn corpus_status(status: BenchmarkCorpusStatus) -> &'static str {
    match status {
        BenchmarkCorpusStatus::Missing => "missing",
        BenchmarkCorpusStatus::SourceDefined => "source_defined",
        BenchmarkCorpusStatus::Admitted => "admitted",
    }
}
