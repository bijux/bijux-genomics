//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages vcf call pseudohaploid stage.
use super::super::{
    materialize_reference_fasta, parse_output_sample_count, path_relative_to_repo, required_output,
    run_call_pseudohaploid_stage, succeeded_row, summarize_haploid_vcf, vcf_validate_input,
    AdnaMicroPipelineRow, AdnaPseudohaploidCallReport, BTreeMap, Context, Path, Result, Value,
    VcfFieldRequirement, ADNA_PIPELINE_SAMPLE_ID, VCF_CALL_PSEUDOHAPLOID_TOOL_ID,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_vcf_call_pseudohaploid_stage_row(
    repo_root: &Path,
    output_root: &Path,
    align_row: &AdnaMicroPipelineRow,
    coverage_row: &AdnaMicroPipelineRow,
    damage_row: &AdnaMicroPipelineRow,
    authenticity_row: &AdnaMicroPipelineRow,
    reference_fasta: &Path,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/vcf.call_pseudohaploid");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let materialized_reference = materialize_reference_fasta(reference_fasta, &stage_root)?;
    let outputs = run_call_pseudohaploid_stage(
        &aligned_bam,
        &stage_root,
        &bijux_dna_domain_vcf::params::VcfCallParams {
            caller: VCF_CALL_PSEUDOHAPLOID_TOOL_ID.to_string(),
            sample_name: ADNA_PIPELINE_SAMPLE_ID.to_string(),
            reference_fasta: Some(materialized_reference.display().to_string()),
            ..bijux_dna_domain_vcf::params::VcfCallParams::default()
        },
    )?;
    let validation = vcf_validate_input(
        &outputs.called_vcf,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )?;
    let sample_count = parse_output_sample_count(&outputs.called_vcf)?;
    let (variant_count, haploid_compatible) = summarize_haploid_vcf(&outputs.called_vcf)?;
    let report = AdnaPseudohaploidCallReport {
        schema_version: "bijux.bench.local_adna_micro_pipeline.pseudohaploid_call.v1".to_string(),
        stage_id: stage::VCF_CALL_PSEUDOHAPLOID.to_string(),
        tool_id: VCF_CALL_PSEUDOHAPLOID_TOOL_ID.to_string(),
        sample_id: ADNA_PIPELINE_SAMPLE_ID.to_string(),
        output_vcf_path: path_relative_to_repo(repo_root, &outputs.called_vcf),
        output_tbi_path: path_relative_to_repo(repo_root, &outputs.called_tbi),
        variant_count,
        sample_count,
        haploid_compatible,
        gt_present: validation.gt_present,
    };
    let report_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::VCF_CALL_PSEUDOHAPLOID,
            domain: "vcf",
            tool_id: VCF_CALL_PSEUDOHAPLOID_TOOL_ID,
            execution_mode: "governed_runtime",
        },
        path_relative_to_repo(repo_root, &report_path),
        &report.schema_version,
        BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            (
                "coverage_report_json".to_string(),
                required_output(coverage_row, "coverage_report_json")?,
            ),
            ("damage_report_json".to_string(), required_output(damage_row, "damage_report_json")?),
            (
                "authenticity_report_json".to_string(),
                required_output(authenticity_row, "authenticity_report_json")?,
            ),
        ]),
        BTreeMap::from([
            (
                "pseudohaploid_vcf".to_string(),
                path_relative_to_repo(repo_root, &outputs.called_vcf),
            ),
            (
                "pseudohaploid_vcf_tbi".to_string(),
                path_relative_to_repo(repo_root, &outputs.called_tbi),
            ),
            ("call_stage_metrics".to_string(), path_relative_to_repo(repo_root, &report_path)),
        ]),
        BTreeMap::from([
            ("variant_count".to_string(), Value::from(variant_count)),
            ("sample_count".to_string(), Value::from(sample_count)),
            ("haploid_compatible".to_string(), Value::from(haploid_compatible)),
        ]),
        "called a governed pseudohaploid VCF from the aligned aDNA-like BAM".to_string(),
    ))
}
