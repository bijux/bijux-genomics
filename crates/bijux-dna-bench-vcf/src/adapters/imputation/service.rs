//! Owner: bijux-dna-bench-vcf
//! imputation adapter service contracts.
use super::{
    anyhow, build_imputation_family_row, ensure_vcf_imputation_family_adapter_contract,
    load_registry_tool_contract, path_relative_to_repo, repo_relative_path, BTreeMap, BTreeSet,
    Path, RegistryToolContract, Result, VcfAdapterInputs, VcfImputationFamilyAdapterReport,
    VcfImputationFamilyAdapterRow, GOVERNED_IMPUTATION_STAGE_IDS, GOVERNED_IMPUTATION_TOOL_IDS,
    VCF_IMPUTATION_FAMILY_ADAPTER_SCHEMA_VERSION,
};

/// Publish a typed adapter report for admitted VCF bindings.
///
/// # Errors
/// Returns an error for invalid tool contracts, unavailable fixture inputs or failed artifact IO.
pub fn render_vcf_imputation_family_adapter(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfImputationFamilyAdapterReport> {
    let report = prepare_vcf_imputation_family_adapter(inputs, repo_root, output_path)?;
    crate::adapters::report::publish(repo_root, output_path, report)
}

/// Acquire validated family observations and project a report without publishing it.
/// Fixture effects are performed only here; the output path is the planned report destination.
pub(crate) fn prepare_vcf_imputation_family_adapter(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfImputationFamilyAdapterReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_vcf_imputation_family_adapter_rows(inputs, repo_root)?;
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let parser_output_row_count =
        rows.iter().filter(|row| !row.parser_output_ids.is_empty()).count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();

    let report = VcfImputationFamilyAdapterReport {
        schema_version: VCF_IMPUTATION_FAMILY_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        tool_count,
        benchmark_ready_row_count,
        parser_output_row_count,
        missing_input_test_passed_row_count,
        rows,
    };
    Ok(report)
}

/// Construct tool-family adapter rows from admitted VCF bindings.
///
/// # Errors
/// Returns an error for invalid tool contracts, unavailable fixture inputs or failed artifact IO.
pub fn collect_vcf_imputation_family_adapter_rows(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
) -> Result<Vec<VcfImputationFamilyAdapterRow>> {
    let catalog_by_stage = inputs
        .catalog()
        .iter()
        .cloned()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let matrix_by_pair = inputs
        .bindings()
        .iter()
        .cloned()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row))
        .collect::<BTreeMap<_, _>>();

    let mut registry_by_tool = BTreeMap::<String, RegistryToolContract>::new();
    for tool_id in GOVERNED_IMPUTATION_TOOL_IDS {
        registry_by_tool
            .insert(tool_id.to_string(), load_registry_tool_contract(repo_root, tool_id)?);
    }

    let mut rows = Vec::new();
    for tool_id in GOVERNED_IMPUTATION_TOOL_IDS {
        let registry_tool = registry_by_tool
            .get(tool_id)
            .ok_or_else(|| anyhow!("missing governed registry tool `{tool_id}`"))?;
        for stage_id in GOVERNED_IMPUTATION_STAGE_IDS {
            let catalog_row = catalog_by_stage.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "VCF imputation-family adapter report is missing catalog coverage for `{stage_id}`"
                )
            })?;
            rows.push(build_imputation_family_row(
                repo_root,
                registry_tool,
                stage_id,
                catalog_row,
                matrix_by_pair.get(&(stage_id.to_string(), tool_id.to_string())),
            )?);
        }
    }
    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then(left.stage_id.cmp(&right.stage_id))
    });
    ensure_vcf_imputation_family_adapter_contract(&registry_by_tool, &rows)?;
    Ok(rows)
}
