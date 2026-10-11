//! Owner: bijux-dna-bench-vcf
//! bcftools adapter service contracts.
use super::{
    anyhow, build_bcftools_row, ensure_vcf_bcftools_adapter_contract, path_relative_to_repo,
    repo_relative_path, BTreeMap, Path, Result, VcfAdapterInputs, VcfBcftoolsAdapterReport,
    VcfBcftoolsAdapterRow, GOVERNED_BCFTOOLS_TOOL_ID, VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION,
};

/// Publish a typed adapter report for admitted VCF bindings.
///
/// # Errors
/// Returns an error for invalid tool contracts, unavailable fixture inputs or failed artifact IO.
pub fn render_vcf_bcftools_adapter(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfBcftoolsAdapterReport> {
    let report = prepare_vcf_bcftools_adapter(inputs, repo_root, output_path)?;
    crate::adapters::report::publish(repo_root, output_path, report)
}

/// Acquire validated family observations and project a report without publishing it.
/// Fixture effects are performed only here; the output path is the planned report destination.
pub(crate) fn prepare_vcf_bcftools_adapter(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfBcftoolsAdapterReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_vcf_bcftools_adapter_rows(inputs, repo_root)?;
    let supported_row_count = rows.iter().filter(|row| row.support_status == "supported").count();
    let planned_row_count = rows.len().saturating_sub(supported_row_count);
    let argv_valid_row_count = rows.iter().filter(|row| row.argv_validation_passed).count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();
    let indexed_row_count = rows
        .iter()
        .filter(|row| row.command_steps.iter().any(|step| step.step_kind == "index"))
        .count();

    Ok(VcfBcftoolsAdapterReport {
        schema_version: VCF_BCFTOOLS_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        tool_id: GOVERNED_BCFTOOLS_TOOL_ID,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        supported_row_count,
        planned_row_count,
        argv_valid_row_count,
        missing_input_test_passed_row_count,
        indexed_row_count,
        rows,
    })
}

/// Construct tool-family adapter rows from admitted VCF bindings.
///
/// # Errors
/// Returns an error for invalid tool contracts, unavailable fixture inputs or failed artifact IO.
pub fn collect_vcf_bcftools_adapter_rows(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
) -> Result<Vec<VcfBcftoolsAdapterRow>> {
    let catalog_by_stage = inputs
        .catalog()
        .iter()
        .cloned()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let mut rows = Vec::new();
    for matrix_row in inputs.bindings() {
        if matrix_row.tool_id != GOVERNED_BCFTOOLS_TOOL_ID {
            continue;
        }
        let catalog_row = catalog_by_stage.get(matrix_row.stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF bcftools adapter report is missing catalog coverage for `{}`",
                matrix_row.stage_id
            )
        })?;
        rows.push(build_bcftools_row(inputs, repo_root, matrix_row, catalog_row)?);
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    ensure_vcf_bcftools_adapter_contract(&rows)?;
    Ok(rows)
}
