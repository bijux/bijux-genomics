//! Owner: bijux-dna-bench-workflows
//! Core germline stages vcf call.
use super::super::{
    bail, materialize_reference_fasta, parse_output_sample_count, parse_vcf_call_summary,
    path_relative_to_repo, read_json_document, required_output, required_tool_id,
    run_call_diploid_stage, vcf_validate_input, BTreeMap, Context, CoreGermlineMicroPipelineRow,
    Path, PathBuf, Result, Value, VcfCallParams, VcfCallStageReport, VcfFieldRequirement,
    PIPELINE_SAMPLE_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_vcf_call_stage(
    repo_root: &Path,
    output_root: &Path,
    align_row: &CoreGermlineMicroPipelineRow,
    coverage_row: &CoreGermlineMicroPipelineRow,
    reference_fasta: &Path,
    tool_ids: &BTreeMap<String, String>,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.call");
    let runtime_root = stage_root.join("runtime");
    bijux_dna_infra::ensure_dir(&runtime_root)
        .with_context(|| format!("create {}", runtime_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let materialized_reference = materialize_reference_fasta(reference_fasta, &runtime_root)?;
    let coverage_report_path =
        repo_root.join(required_output(coverage_row, "coverage_report_json")?);
    let coverage_regions_path =
        repo_root.join(required_output(coverage_row, "coverage_regions_json")?);
    let coverage_regions = read_json_document(&coverage_regions_path)?;
    let coverage_gate_passed = admits_variant_calling(&coverage_regions);
    if !coverage_gate_passed {
        bail!("vcf.call coverage gate failed for {}", coverage_report_path.display());
    }

    let stage_outputs = run_call_diploid_stage(
        &aligned_bam,
        &runtime_root,
        &VcfCallParams {
            caller: required_tool_id(tool_ids, stage::VCF_CALL)?,
            sample_name: PIPELINE_SAMPLE_ID.to_string(),
            reference_fasta: Some(materialized_reference.display().to_string()),
            ..VcfCallParams::default()
        },
    )?;
    let validation = vcf_validate_input(
        &stage_outputs.called_vcf,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )?;
    let call_summary = parse_vcf_call_summary(&stage_outputs.called_vcf, PIPELINE_SAMPLE_ID)?;
    let sample_count = parse_output_sample_count(&stage_outputs.called_vcf)?;
    let reference_fai = PathBuf::from(format!("{}.fai", materialized_reference.display()));
    if !reference_fai.is_file() {
        bail!("vcf.call did not materialize {}", reference_fai.display());
    }

    let report = VcfCallStageReport {
        schema_version: "bijux.bench.local_core_germline_micro_pipeline.vcf_call.v1".to_string(),
        stage_id: stage::VCF_CALL.to_string(),
        tool_id: required_tool_id(tool_ids, stage::VCF_CALL)?,
        sample_id: PIPELINE_SAMPLE_ID.to_string(),
        consumed_coverage_report: path_relative_to_repo(repo_root, &coverage_report_path),
        output_vcf_path: path_relative_to_repo(repo_root, &stage_outputs.called_vcf),
        output_tbi_path: path_relative_to_repo(repo_root, &stage_outputs.called_tbi),
        call_metrics_path: path_relative_to_repo(repo_root, &stage_outputs.call_metrics_json),
        call_manifest_path: path_relative_to_repo(repo_root, &stage_outputs.call_manifest_json),
        reference_fasta_path: path_relative_to_repo(repo_root, &materialized_reference),
        reference_fai_path: path_relative_to_repo(repo_root, &reference_fai),
        variant_count: call_summary.variants_called,
        snp_count: call_summary.snps,
        indel_count: call_summary.indels,
        sample_count,
        coverage_gate_passed,
        validation_checks: validation.checks.clone(),
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::VCF_CALL.to_string(),
        domain: "vcf".to_string(),
        tool_id: report.tool_id.clone(),
        execution_mode: "tool_backed".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("aligned_bai".to_string(), required_output(align_row, "aligned_bai")?),
            (
                "coverage_report_json".to_string(),
                required_output(coverage_row, "coverage_report_json")?,
            ),
            (
                "coverage_regions_json".to_string(),
                required_output(coverage_row, "coverage_regions_json")?,
            ),
            (
                "reference_fasta_contract".to_string(),
                path_relative_to_repo(repo_root, &materialized_reference),
            ),
            (
                "reference_fai_contract".to_string(),
                path_relative_to_repo(repo_root, &reference_fai),
            ),
        ]),
        outputs: BTreeMap::from([
            ("called_vcf".to_string(), report.output_vcf_path.clone()),
            ("called_vcf_tbi".to_string(), report.output_tbi_path.clone()),
            ("call_metrics_json".to_string(), report.call_metrics_path.clone()),
            ("call_manifest_json".to_string(), report.call_manifest_path.clone()),
        ]),
        metrics: BTreeMap::from([
            ("variant_count".to_string(), Value::from(report.variant_count)),
            ("snp_count".to_string(), Value::from(report.snp_count)),
            ("indel_count".to_string(), Value::from(report.indel_count)),
            ("sample_count".to_string(), Value::from(report.sample_count)),
            ("coverage_gate_passed".to_string(), Value::from(report.coverage_gate_passed)),
        ]),
    })
}

fn admits_variant_calling(coverage_regions: &Value) -> bool {
    coverage_regions.as_array().is_some_and(|rows| {
        rows.iter().any(|row| {
            row.get("covered_bases").and_then(Value::as_u64).is_some_and(|count| count > 0)
                || row.get("mean_depth").and_then(Value::as_f64).is_some_and(|depth| depth > 0.0)
        })
    })
}
