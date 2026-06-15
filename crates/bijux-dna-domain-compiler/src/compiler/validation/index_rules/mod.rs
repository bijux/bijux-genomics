use super::{
    anyhow, bail, is_umbrella_stage, is_unspecified, read_yaml, BTreeMap, BTreeSet, Context,
    DomainIndex, DomainStage, DomainToolLoose, Path, Result, ValidateOptions,
};

mod compatibility_matrix;
mod domain_inventory;
mod domain_versions;

use self::compatibility_matrix::{validate_index_matrix_and_pipelines, ToolCatalogs};
use self::domain_inventory::validate_domain_index_inventory;
use self::domain_versions::validate_domain_versions;

pub(super) fn validate_domain_indexes_and_pipelines(
    options: &ValidateOptions,
    stage_ids: &BTreeMap<String, String>,
    tool_ids: &BTreeMap<String, String>,
    tool_capabilities_by_domain: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    tool_statuses_by_domain: &BTreeMap<String, BTreeMap<String, String>>,
    tool_metrics_schemas_by_domain: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<()> {
    validate_domain_versions(options)?;

    for dom in ["fastq", "bam", "vcf"] {
        let index_path = options.domain_dir.join(dom).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;
        let stage_status_by_id =
            validate_domain_index_inventory(options, dom, &index, stage_ids, tool_ids)?;
        let domain_capabilities = tool_capabilities_by_domain.get(dom).ok_or_else(|| {
            anyhow!("{} missing tool capability catalog for domain {}", index_path.display(), dom)
        })?;
        let domain_statuses = tool_statuses_by_domain.get(dom).ok_or_else(|| {
            anyhow!("{} missing tool status catalog for domain {}", index_path.display(), dom)
        })?;
        let domain_metrics_schemas = tool_metrics_schemas_by_domain.get(dom).ok_or_else(|| {
            anyhow!(
                "{} missing tool metrics schema catalog for domain {}",
                index_path.display(),
                dom
            )
        })?;
        let tool_catalogs = ToolCatalogs {
            capabilities: domain_capabilities,
            statuses: domain_statuses,
            metrics_schemas: domain_metrics_schemas,
        };
        if dom == "vcf" {
            validate_vcf_index_contracts(&index, &index_path, &stage_status_by_id)?;
        } else {
            validate_index_matrix_and_pipelines(
                options,
                dom,
                &index,
                &index_path,
                &stage_status_by_id,
                &tool_catalogs,
            )?;
        }
    }
    Ok(())
}

fn validate_vcf_index_contracts(
    index: &DomainIndex,
    index_path: &Path,
    stage_status_by_id: &BTreeMap<String, String>,
) -> Result<()> {
    for (stage_id, status) in stage_status_by_id {
        if status != "supported" {
            continue;
        }
        let compatible_tools = index.stage_tool_compatibility.get(stage_id).ok_or_else(|| {
            anyhow!(
                "{} supported VCF stage {} missing stage_tool_compatibility entry",
                index_path.display(),
                stage_id
            )
        })?;
        if compatible_tools.is_empty() {
            bail!(
                "{} supported VCF stage {} must declare at least one compatible tool",
                index_path.display(),
                stage_id
            );
        }
        let default_tool = index.active_defaults.get(stage_id).ok_or_else(|| {
            anyhow!(
                "{} supported VCF stage {} missing active_defaults entry",
                index_path.display(),
                stage_id
            )
        })?;
        if !compatible_tools.iter().any(|tool_id| tool_id == default_tool) {
            bail!(
                "{} VCF default {} is not in stage_tool_compatibility for {}",
                index_path.display(),
                default_tool,
                stage_id
            );
        }
        let rationale =
            index.active_default_rationale.get(stage_id).map_or("", std::string::String::as_str);
        if is_unspecified(rationale) {
            bail!(
                "{} supported VCF stage {} missing active_default_rationale",
                index_path.display(),
                stage_id
            );
        }
    }
    Ok(())
}
