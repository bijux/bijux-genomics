use super::{
    anyhow, bail, collect_yaml_files, ensure_status, has_supported_placeholder_forbidden_token,
    is_tool_meaningful_in_domain, is_umbrella_stage, is_unspecified, placeholders_allowed,
    read_yaml, validate_tool_output_subset, AdapterBank, BTreeMap, BTreeSet, ContainerToolContract,
    ContaminationDbBank, Context, DomainArtifactVocabulary, DomainIndex, DomainMetricVocabulary,
    DomainStage, DomainToolLoose, Path, ReferenceBank, Result, ValidateOptions,
    DEFAULT_COMPILE_SCOPE,
};

mod catalog_coverage;
mod catalog_validation;
mod deprecations;
mod fixture_consistency;
mod index_rules;
mod stage_files;
mod strict_stage_schemas;
mod tool_files;

use self::catalog_coverage::validate_canonical_stage_coverage;
use self::catalog_validation::{
    validate_domain_vocabularies, validate_reference_catalogs, DomainVocabularies,
};
use self::deprecations::validate_deprecations_contract;
use self::fixture_consistency::validate_fixture_consistency;
use self::index_rules::validate_domain_indexes_and_pipelines;
use self::stage_files::validate_stage_files;
use self::strict_stage_schemas::validate_stage_schema_contracts;
use self::tool_files::{validate_tool_files, ToolValidationState};

/// Validate authored domain files and cross-domain invariants.
///
/// # Errors
///
/// Returns an error when required files are missing, schemas/invariants are
/// violated, or domain catalogs are inconsistent.
pub fn validate_domain(options: &ValidateOptions) -> Result<()> {
    for rel in [
        "fastq/stages/_schema.yaml",
        "bam/stages/_schema.yaml",
        "vcf/stages/_schema.yaml",
        "fastq/tools/_schema.yaml",
        "bam/tools/_schema.yaml",
        "vcf/tools/_schema.yaml",
        "fastq/artifacts.yaml",
        "bam/artifacts.yaml",
        "vcf/artifacts.yaml",
        "fastq/metrics.yaml",
        "bam/metrics.yaml",
        "vcf/metrics.yaml",
        "fastq/index.yaml",
        "bam/index.yaml",
        "vcf/index.yaml",
    ] {
        super::compile::require_exists(&options.domain_dir.join(rel))?;
    }
    let workspace_root = options.domain_dir.parent().unwrap_or(&options.domain_dir);
    validate_reference_catalogs(workspace_root)?;
    let container_catalog = super::loading::load_container_catalog(&options.domain_dir)?;
    let mut tool_ids = BTreeMap::<String, String>::new();
    let mut stage_ids = BTreeMap::<String, String>::new();
    let mut tool_capabilities = BTreeMap::<String, BTreeSet<String>>::new();
    let mut tool_statuses = BTreeMap::<String, String>::new();
    let mut tool_metrics_schemas = BTreeMap::<String, String>::new();
    let mut tool_capabilities_by_domain =
        BTreeMap::<String, BTreeMap<String, BTreeSet<String>>>::new();
    let mut tool_statuses_by_domain = BTreeMap::<String, BTreeMap<String, String>>::new();
    let mut tool_metrics_schemas_by_domain = BTreeMap::<String, BTreeMap<String, String>>::new();
    let DomainVocabularies { artifact_vocab, metric_vocab } =
        validate_domain_vocabularies(&options.domain_dir)?;

    for dom in ["fastq", "bam", "vcf"] {
        validate_stage_schema_contracts(options, dom)?;
        validate_stage_files(options, dom, &artifact_vocab, &metric_vocab, &mut stage_ids)?;
        let mut tool_state = ToolValidationState {
            ids: &mut tool_ids,
            capabilities: &mut tool_capabilities,
            statuses: &mut tool_statuses,
            metrics_schemas: &mut tool_metrics_schemas,
            domain_capabilities: &mut tool_capabilities_by_domain,
            domain_statuses: &mut tool_statuses_by_domain,
            domain_metrics_schemas: &mut tool_metrics_schemas_by_domain,
        };
        validate_tool_files(options, dom, &artifact_vocab, &container_catalog, &mut tool_state)?;
    }

    validate_canonical_stage_coverage(&stage_ids)?;

    validate_domain_indexes_and_pipelines(
        options,
        &stage_ids,
        &tool_ids,
        &tool_capabilities_by_domain,
        &tool_statuses_by_domain,
        &tool_metrics_schemas_by_domain,
    )?;
    validate_fixture_consistency(options, &stage_ids, &tool_ids)?;
    validate_deprecations_contract(workspace_root, &stage_ids, &tool_ids)?;

    println!("domain-validate: OK");
    Ok(())
}
