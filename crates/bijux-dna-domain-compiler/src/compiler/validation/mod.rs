use super::{
    anyhow, bail, collect_yaml_files, ensure_status, has_supported_placeholder_forbidden_token,
    is_tool_meaningful_in_domain, is_umbrella_stage, is_unspecified, placeholders_allowed,
    read_yaml, validate_tool_output_subset, AdapterBank, BTreeMap, BTreeSet, ContaminationDbBank,
    Context, DomainArtifactVocabulary, DomainIndex, DomainMetricVocabulary, DomainStage,
    DomainToolLoose, Path, ReferenceBank, Result, ValidateOptions, DEFAULT_COMPILE_SCOPE,
};

mod catalog_coverage;
mod catalog_validation;
mod fixture_consistency;
mod index_rules;
mod stage_files;
mod strict_stage_schemas;
mod tool_files;

use self::catalog_coverage::validate_canonical_stage_coverage;
use self::catalog_validation::{
    validate_domain_vocabularies, validate_reference_catalogs, DomainVocabularies,
};
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
    let shared_tool_domains = load_shared_tool_domains(workspace_root)?;

    let mut tool_ids = BTreeMap::<String, String>::new();
    let mut stage_ids = BTreeMap::<String, String>::new();
    let mut tool_capabilities = BTreeMap::<String, BTreeSet<String>>::new();
    let mut tool_statuses = BTreeMap::<String, String>::new();
    let mut tool_metrics_schemas = BTreeMap::<String, String>::new();
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
        };
        validate_tool_files(options, dom, &artifact_vocab, &shared_tool_domains, &mut tool_state)?;
    }

    validate_canonical_stage_coverage(&stage_ids)?;

    validate_domain_indexes_and_pipelines(
        options,
        &stage_ids,
        &tool_ids,
        &tool_capabilities,
        &tool_statuses,
        &tool_metrics_schemas,
    )?;
    validate_fixture_consistency(options, &stage_ids, &tool_ids)?;

    println!("domain-validate: OK");
    Ok(())
}

fn load_shared_tool_domains(workspace_root: &Path) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let path = workspace_root.join("configs/domain/shared_tools.toml");
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let mut shared_tool_domains = BTreeMap::new();
    let Some(shared_tools) = value.get("shared_tools").and_then(toml::Value::as_table) else {
        return Ok(shared_tool_domains);
    };
    for (tool_id, entry) in shared_tools {
        let domains = entry
            .get("domains")
            .and_then(toml::Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(toml::Value::as_str)
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        shared_tool_domains.insert(tool_id.clone(), domains);
    }
    Ok(shared_tool_domains)
}
