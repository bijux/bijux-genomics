//! Admit scientific output evidence and construct the persisted benchmark metrics.
use crate::damage_filter::execution::artifacts::StageArtifacts;
use crate::damage_filter::model::GovernedVcfDamageFilterSmokeContract;
use crate::damage_filter::model::LocalVcfDamageFilterSmokeMetrics;
use crate::damage_filter::model::LOCAL_VCF_DAMAGE_FILTER_SMOKE_METRICS_SCHEMA_VERSION;
use crate::damage_filter::observations::reports::count_value;
use crate::damage_filter::observations::reports::read_json;
use crate::damage_filter::observations::variants::format_damage_context_rule;
use crate::damage_filter::observations::variants::parse_vcf_record_count;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::vcf_sample_count as parse_output_sample_count;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;

pub(crate) struct StageEvidence {
    pub(crate) metrics: LocalVcfDamageFilterSmokeMetrics,
    pub(crate) validation: VcfValidationSummary,
}

pub(crate) fn observe_stage(
    contract: &GovernedVcfDamageFilterSmokeContract,
    artifacts: &StageArtifacts,
    input_variants: u64,
) -> Result<StageEvidence> {
    let output_vcf = &artifacts.output_vcf;
    let summary_path = &artifacts.summary_path;
    let counts_path = &artifacts.counts_path;
    let validation =
        vcf_validate_input(output_vcf, VcfFieldRequirement { require_gt: true, require_gl: true })
            .with_context(|| format!("validate {}", output_vcf.display()))?;
    let retained_variants = parse_vcf_record_count(output_vcf)?;
    if retained_variants >= input_variants {
        bail!(
            "governed damage_filter smoke must remove at least one variant, found input={input_variants} output={retained_variants}"
        );
    }
    let removed_variants = input_variants - retained_variants;
    let sample_count = parse_output_sample_count(output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;
    if sample_count != 1 {
        bail!("governed damage_filter smoke expects exactly one sample, found {sample_count}");
    }

    let summary_json = read_json(summary_path)?;
    let counts_json = read_json(counts_path)?;
    let terminal_context_count = summary_json
        .pointer("/prefilter/read_position_signal/ct_five_prime_high")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        + summary_json
            .pointer("/prefilter/read_position_signal/ga_three_prime_high")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
    if terminal_context_count == 0 {
        bail!("governed damage_filter smoke requires at least one terminal damage context row");
    }
    let low_quality_filtered_variants = count_value(&counts_json, "/counts/low_qual");
    let damage_ratio_filtered_variants = count_value(&counts_json, "/counts/damage_ratio_exceeded");
    let terminal_damage_filtered_variants =
        count_value(&counts_json, "/counts/terminal_damage_filtered");
    let damage_context_rule = format_damage_context_rule(&summary_json)?;

    let metrics = LocalVcfDamageFilterSmokeMetrics {
        schema_version: LOCAL_VCF_DAMAGE_FILTER_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        input_variants,
        removed_variants,
        retained_variants,
        low_quality_filtered_variants,
        damage_ratio_filtered_variants,
        terminal_damage_filtered_variants,
        damage_context_rule: damage_context_rule.clone(),
        terminal_context_count,
        sample_count,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    Ok(StageEvidence { metrics, validation })
}
