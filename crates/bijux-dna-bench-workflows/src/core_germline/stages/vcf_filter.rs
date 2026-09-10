//! Owner: bijux-dna-bench-workflows
//! Core germline stages vcf filter.
use super::super::{
    parse_output_sample_count, path_relative_to_repo, read_json_document, required_output,
    required_tool_id, run_filter_stage_real, vcf_validate_input, BTreeMap, Context,
    CoreGermlineMicroPipelineRow, Path, Result, Value, VcfFieldRequirement, VcfFilterParams,
    VcfFilterStageReport, PIPELINE_SAMPLE_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_vcf_filter_stage(
    repo_root: &Path,
    output_root: &Path,
    call_row: &CoreGermlineMicroPipelineRow,
    tool_ids: &BTreeMap<String, String>,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.filter");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let called_vcf = repo_root.join(required_output(call_row, "called_vcf")?);
    let stage_outputs = run_filter_stage_real(
        &called_vcf,
        &stage_root,
        &VcfFilterParams {
            sample_name: PIPELINE_SAMPLE_ID.to_string(),
            min_qual: 0.0,
            require_pass: false,
            normalize: true,
            require_bgzip_tabix: true,
            production_profile: false,
            ..VcfFilterParams::default()
        },
    )?;
    let validation = vcf_validate_input(
        &stage_outputs.filtered_vcf,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )?;
    let sample_count = parse_output_sample_count(&stage_outputs.filtered_vcf)?;
    let filter_breakdown = read_json_document(&stage_outputs.filter_breakdown_json)?;
    let pass_variant_count =
        filter_breakdown.pointer("/counts/PASS").and_then(Value::as_u64).unwrap_or(0);
    let total_tagged_variant_count =
        filter_breakdown.get("counts").and_then(Value::as_object).map_or(0, |counts| {
            counts
                .iter()
                .filter(|(filter_id, _)| filter_id.as_str() != "PASS")
                .filter_map(|(_, count)| count.as_u64())
                .sum::<u64>()
        });
    let report = VcfFilterStageReport {
        schema_version: "bijux.bench.local_core_germline_micro_pipeline.vcf_filter.v1".to_string(),
        stage_id: stage::VCF_FILTER.to_string(),
        tool_id: required_tool_id(tool_ids, stage::VCF_FILTER)?,
        consumed_call_metrics_path: required_output(call_row, "call_metrics_json")?,
        output_vcf_path: path_relative_to_repo(repo_root, &stage_outputs.filtered_vcf),
        output_tbi_path: path_relative_to_repo(repo_root, &stage_outputs.filtered_tbi),
        filter_breakdown_path: path_relative_to_repo(
            repo_root,
            &stage_outputs.filter_breakdown_json,
        ),
        filter_breakdown_tsv_path: path_relative_to_repo(
            repo_root,
            &stage_outputs.filter_breakdown_tsv,
        ),
        filter_explain_path: path_relative_to_repo(repo_root, &stage_outputs.filter_explain_json),
        pass_variant_count,
        tagged_variant_count: total_tagged_variant_count,
        sample_count,
        validation_checks: validation.checks.clone(),
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::VCF_FILTER.to_string(),
        domain: "vcf".to_string(),
        tool_id: report.tool_id.clone(),
        execution_mode: "stage_runtime".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("called_vcf".to_string(), required_output(call_row, "called_vcf")?),
            ("called_vcf_tbi".to_string(), required_output(call_row, "called_vcf_tbi")?),
            ("call_metrics_json".to_string(), required_output(call_row, "call_metrics_json")?),
        ]),
        outputs: BTreeMap::from([
            ("filtered_vcf".to_string(), report.output_vcf_path.clone()),
            ("filtered_vcf_tbi".to_string(), report.output_tbi_path.clone()),
            ("filter_report_json".to_string(), path_relative_to_repo(repo_root, &report_path)),
            ("filter_breakdown_json".to_string(), report.filter_breakdown_path.clone()),
        ]),
        metrics: BTreeMap::from([
            ("pass_variant_count".to_string(), Value::from(report.pass_variant_count)),
            ("tagged_variant_count".to_string(), Value::from(report.tagged_variant_count)),
            ("sample_count".to_string(), Value::from(report.sample_count)),
        ]),
    })
}
