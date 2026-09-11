//! Owner: bijux-dna-bench-vcf
//! angsd adapter service contracts.
use super::{
    anyhow, build_angsd_row, ensure_vcf_angsd_adapter_contract, load_admitted_angsd_stage_ids,
    path_relative_to_repo, repo_relative_path, BTreeMap, BTreeSet, Path, Result, VcfAdapterInputs,
    VcfAngsdAdapterReport, VcfAngsdAdapterRow, GOVERNED_ANGSD_TOOL_ID, GOVERNED_ANGSD_TOOL_STATUS,
    VCF_ANGSD_ADAPTER_SCHEMA_VERSION,
};

/// Publish a typed adapter report for admitted VCF bindings.
///
/// # Errors
/// Returns an error for invalid tool contracts, unavailable fixture inputs or failed artifact IO.
pub fn render_vcf_angsd_adapter(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfAngsdAdapterReport> {
    let report = prepare_vcf_angsd_adapter(inputs, repo_root, output_path)?;
    crate::adapters::report::publish(repo_root, output_path, report)
}

/// Acquire validated family observations and project a report without publishing it.
/// Fixture effects are performed only here; the output path is the planned report destination.
pub(crate) fn prepare_vcf_angsd_adapter(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfAngsdAdapterReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_vcf_angsd_adapter_rows(inputs, repo_root)?;
    let supported_stage_row_count =
        rows.iter().filter(|row| row.stage_status == "supported").count();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let argv_valid_row_count = rows.iter().filter(|row| row.argv_validation_passed).count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();
    let bam_list_row_count = rows.iter().filter(|row| row.bam_list_path.is_some()).count();
    let parser_output_row_count =
        rows.iter().filter(|row| !row.parser_output_ids.is_empty()).count();

    Ok(VcfAngsdAdapterReport {
        schema_version: VCF_ANGSD_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        tool_id: GOVERNED_ANGSD_TOOL_ID,
        tool_status: GOVERNED_ANGSD_TOOL_STATUS,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        supported_stage_row_count,
        benchmark_ready_row_count,
        argv_valid_row_count,
        missing_input_test_passed_row_count,
        bam_list_row_count,
        parser_output_row_count,
        rows,
    })
}

/// Construct tool-family adapter rows from admitted VCF bindings.
///
/// # Errors
/// Returns an error for invalid tool contracts, unavailable fixture inputs or failed artifact IO.
pub fn collect_vcf_angsd_adapter_rows(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
) -> Result<Vec<VcfAngsdAdapterRow>> {
    let catalog_by_stage = inputs
        .catalog()
        .iter()
        .cloned()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let benchmark_ready_stage_ids = inputs
        .bindings()
        .iter()
        .filter(|row| row.tool_id == GOVERNED_ANGSD_TOOL_ID)
        .cloned()
        .map(|row| row.stage_id)
        .collect::<BTreeSet<_>>();
    let admitted_stage_ids = load_admitted_angsd_stage_ids(repo_root)?;

    let mut rows = Vec::new();
    for stage_id in admitted_stage_ids {
        let catalog_row = catalog_by_stage.get(stage_id.as_str()).ok_or_else(|| {
            anyhow!("VCF angsd adapter report is missing catalog coverage for `{stage_id}`")
        })?;
        rows.push(build_angsd_row(
            repo_root,
            &stage_id,
            catalog_row,
            benchmark_ready_stage_ids.contains(stage_id.as_str()),
        )?);
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    ensure_vcf_angsd_adapter_contract(&rows)?;
    Ok(rows)
}
