//! Admit scientific output evidence and construct the persisted benchmark metrics.
use crate::filter::execution::artifacts::StageArtifacts;
use crate::filter::model::GovernedVcfFilterSmokeContract;
use crate::filter::model::LocalVcfFilterSmokeMetrics;
use crate::filter::model::EXPECTED_FILTER_IDS;
use crate::filter::model::LOCAL_VCF_FILTER_SMOKE_METRICS_SCHEMA_VERSION;
use crate::filter::observations::reports::read_json;
use crate::filter::observations::variants::extract_threshold;
use crate::filter::observations::variants::summarize_filtered_output;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::vcf_sample_count as parse_output_sample_count;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;

pub(crate) struct StageEvidence {
    pub(crate) metrics: LocalVcfFilterSmokeMetrics,
    pub(crate) validation: VcfValidationSummary,
}

pub(crate) fn observe_stage(
    contract: &GovernedVcfFilterSmokeContract,
    artifacts: &StageArtifacts,
    input_variants: u64,
) -> Result<StageEvidence> {
    let output_vcf = &artifacts.output_vcf;
    let filter_breakdown_path = &artifacts.filter_breakdown_path;
    let filter_explain_path = &artifacts.filter_explain_path;
    let validation =
        vcf_validate_input(output_vcf, VcfFieldRequirement { require_gt: true, require_gl: false })
            .with_context(|| format!("validate {}", output_vcf.display()))?;
    let sample_count = parse_output_sample_count(output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;
    if sample_count != 1 {
        bail!("governed VCF filter smoke expects exactly one sample, found {sample_count}");
    }
    let output_summary = summarize_filtered_output(output_vcf)?;
    if output_summary.pass_variants + output_summary.failed_variants != input_variants {
        bail!(
            "governed VCF filter smoke output count drifted: input={} pass={} failed={}",
            input_variants,
            output_summary.pass_variants,
            output_summary.failed_variants
        );
    }
    let expected_filter_ids =
        EXPECTED_FILTER_IDS.iter().map(|id| (*id).to_string()).collect::<Vec<_>>();
    if output_summary.filter_ids != expected_filter_ids {
        bail!(
            "governed VCF filter smoke expected filter ids {:?}, found {:?}",
            expected_filter_ids,
            output_summary.filter_ids
        );
    }

    let filter_breakdown = read_json(filter_breakdown_path)?;
    let pass_count = filter_breakdown
        .pointer("/counts/PASS")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("filter breakdown is missing PASS count"))?;
    if pass_count != output_summary.pass_variants {
        bail!(
            "VCF filter smoke PASS count drifted between VCF and breakdown: {} vs {}",
            output_summary.pass_variants,
            pass_count
        );
    }
    for expected in EXPECTED_FILTER_IDS {
        let count = filter_breakdown
            .pointer(&format!("/counts/{expected}"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if count != 1 {
            bail!("governed VCF filter smoke expected one `{expected}` row, found {count}");
        }
    }

    let filter_explain = read_json(filter_explain_path)?;
    let output_subset = filter_explain
        .pointer("/filter_scope/output_subset")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("filter explain is missing output subset"))?;
    if output_subset != "retain_tagged_records" {
        bail!(
            "governed VCF filter smoke requires tagged-record retention, found `{output_subset}`"
        );
    }
    let depth_threshold = extract_threshold(&filter_explain, "/thresholds/min_depth", "min_depth")?;
    let quality_threshold = extract_threshold(&filter_explain, "/thresholds/min_qual", "min_qual")?;
    let missingness_threshold = extract_threshold(
        &filter_explain,
        "/thresholds/sample_missingness_max",
        "sample_missingness_max",
    )?;

    let metrics = LocalVcfFilterSmokeMetrics {
        schema_version: LOCAL_VCF_FILTER_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        input_variants,
        pass_variants: output_summary.pass_variants,
        failed_variants: output_summary.failed_variants,
        filter_ids: output_summary.filter_ids.clone(),
        depth_threshold,
        quality_threshold,
        missingness_threshold,
        sample_count,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    Ok(StageEvidence { metrics, validation })
}
