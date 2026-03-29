use super::*;

mod compatibility_matrix;
mod domain_inventory;
mod domain_versions;

use self::compatibility_matrix::validate_index_matrix_and_pipelines;
use self::domain_inventory::validate_domain_index_inventory;
use self::domain_versions::validate_domain_versions;

pub(super) fn validate_domain_indexes_and_pipelines(
    options: &ValidateOptions,
    stage_ids: &BTreeMap<String, String>,
    tool_ids: &BTreeMap<String, String>,
    tool_capabilities: &BTreeMap<String, BTreeSet<String>>,
    tool_statuses: &BTreeMap<String, String>,
    tool_metrics_schemas: &BTreeMap<String, String>,
) -> Result<()> {
    validate_domain_versions(options)?;

    for dom in ["fastq", "bam"] {
        let index_path = options.domain_dir.join(dom).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;
        let stage_status_by_id =
            validate_domain_index_inventory(options, dom, &index, stage_ids, tool_ids)?;
        validate_index_matrix_and_pipelines(
            options,
            dom,
            &index,
            &index_path,
            &stage_status_by_id,
            tool_capabilities,
            tool_statuses,
            tool_metrics_schemas,
        )?;
    }
    Ok(())
}
