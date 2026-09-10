//! Owner: bijux-dna-bench-vcf
//! descent adapter service contracts.
use super::{
    anyhow, build_descent_family_row, ensure_vcf_descent_family_adapter_contract,
    load_registry_tool_contract, path_relative_to_repo, repo_relative_path, BTreeMap, BTreeSet,
    Context, Path, Result, VcfAdapterInputs, VcfDescentFamilyAdapterReport,
    VcfDescentFamilyAdapterRow, GOVERNED_DESCENT_ROWS, VCF_DESCENT_FAMILY_ADAPTER_SCHEMA_VERSION,
};

/// Publish a typed adapter report for admitted VCF bindings.
///
/// # Errors
/// Returns an error for invalid tool contracts, unavailable fixture inputs or failed artifact IO.
pub fn render_vcf_descent_family_adapter(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfDescentFamilyAdapterReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_vcf_descent_family_adapter_rows(inputs, repo_root)?;
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let parser_output_row_count =
        rows.iter().filter(|row| !row.parser_output_ids.is_empty()).count();
    let normalized_output_row_count =
        rows.iter().filter(|row| !row.normalized_output_path.is_empty()).count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfDescentFamilyAdapterReport {
        schema_version: VCF_DESCENT_FAMILY_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        tool_count,
        benchmark_ready_row_count,
        parser_output_row_count,
        normalized_output_row_count,
        missing_input_test_passed_row_count,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

/// Construct tool-family adapter rows from admitted VCF bindings.
///
/// # Errors
/// Returns an error for invalid tool contracts, unavailable fixture inputs or failed artifact IO.
pub fn collect_vcf_descent_family_adapter_rows(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
) -> Result<Vec<VcfDescentFamilyAdapterRow>> {
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

    let mut rows = Vec::new();
    for (tool_id, stage_id) in GOVERNED_DESCENT_ROWS {
        let registry_tool = load_registry_tool_contract(repo_root, tool_id)?;
        let catalog_row = catalog_by_stage.get(stage_id).ok_or_else(|| {
            anyhow!(
                "VCF descent-family adapter report is missing catalog coverage for `{stage_id}`"
            )
        })?;
        rows.push(build_descent_family_row(
            repo_root,
            &registry_tool,
            stage_id,
            catalog_row,
            matrix_by_pair.get(&(stage_id.to_string(), tool_id.to_string())),
        )?);
    }
    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then(left.tool_id.cmp(&right.tool_id))
    });
    ensure_vcf_descent_family_adapter_contract(&rows)?;
    Ok(rows)
}
