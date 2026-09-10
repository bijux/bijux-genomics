//! Owner: bijux-dna-bench-vcf
//! eigensoft adapter service contracts.
use super::{
    anyhow, build_eigensoft_row, ensure_vcf_eigensoft_adapter_contract,
    load_registry_tool_contract, path_relative_to_repo, repo_relative_path, BTreeMap, Context,
    Path, RegistryToolContract, Result, VcfAdapterInputs, VcfEigensoftAdapterReport,
    VcfEigensoftAdapterRow, VCF_EIGENSOFT_ADAPTER_SCHEMA_VERSION,
};

/// Publish a typed adapter report for admitted VCF bindings.
///
/// # Errors
/// Returns an error for invalid tool contracts, unavailable fixture inputs or failed artifact IO.
pub fn render_vcf_eigensoft_adapter(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfEigensoftAdapterReport> {
    let registry_tool = load_registry_tool_contract(repo_root, "eigensoft")?;
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_vcf_eigensoft_adapter_rows(inputs, repo_root, &registry_tool)?;
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let parser_output_row_count =
        rows.iter().filter(|row| !row.parser_output_ids.is_empty()).count();
    let normalized_metrics_row_count =
        rows.iter().filter(|row| !row.normalized_metrics_path.is_empty()).count();
    let conversion_output_row_count = rows
        .iter()
        .filter(|row| {
            !row.geno_path.is_empty() && !row.snp_path.is_empty() && !row.ind_path.is_empty()
        })
        .count();
    let pca_output_row_count = rows
        .iter()
        .filter(|row| !row.eigenvec_path.is_empty() && !row.eigenval_path.is_empty())
        .count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfEigensoftAdapterReport {
        schema_version: VCF_EIGENSOFT_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        tool_id: registry_tool.tool_id.clone(),
        tool_status: registry_tool.tool_status.clone(),
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        benchmark_ready_row_count,
        parser_output_row_count,
        normalized_metrics_row_count,
        conversion_output_row_count,
        pca_output_row_count,
        missing_input_test_passed_row_count,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

pub(super) fn collect_vcf_eigensoft_adapter_rows(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
) -> Result<Vec<VcfEigensoftAdapterRow>> {
    let catalog_by_stage = inputs
        .catalog()
        .iter()
        .cloned()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let matrix_by_stage = inputs
        .bindings()
        .iter()
        .filter(|row| row.tool_id == registry_tool.tool_id)
        .cloned()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();
    for stage_id in &registry_tool.stage_ids {
        let catalog_row = catalog_by_stage.get(stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF {} adapter report is missing catalog coverage for `{stage_id}`",
                registry_tool.tool_id
            )
        })?;
        rows.push(build_eigensoft_row(
            repo_root,
            registry_tool,
            stage_id,
            catalog_row,
            matrix_by_stage.get(stage_id.as_str()),
        )?);
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    ensure_vcf_eigensoft_adapter_contract(registry_tool, &rows)?;
    Ok(rows)
}

/// Construct tool-family adapter rows from admitted VCF bindings.
///
/// # Errors
/// Returns an error for invalid tool contracts, unavailable fixture inputs or failed artifact IO.
pub fn collect_vcf_eigensoft_adapter_rows_for_tool(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
) -> Result<Vec<VcfEigensoftAdapterRow>> {
    let registry_tool = load_registry_tool_contract(repo_root, "eigensoft")?;
    collect_vcf_eigensoft_adapter_rows(inputs, repo_root, &registry_tool)
}
