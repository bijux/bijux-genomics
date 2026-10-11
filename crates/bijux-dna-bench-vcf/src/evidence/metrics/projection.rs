use anyhow::{anyhow, Result};
use bijux_dna_bench_model::{ComparableMetricDirection, VcfComparableMetricsRow};
use bijux_dna_domain_vcf::{
    comparable_metric_stage_ids, stage_comparable_metric_specs, VcfComparableMetricDirection,
    VcfDomainStage,
};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn collect_vcf_comparable_metric_rows(
    stage_tools: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<VcfComparableMetricsRow>> {
    let governed_stage_ids = comparable_metric_stage_ids()
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let multi_tool_stage_ids = stage_tools
        .iter()
        .filter(|(_, tools)| tools.len() >= 2)
        .map(|(stage_id, _)| stage_id.clone())
        .collect::<BTreeSet<_>>();

    let mut missing_contract_stage_ids =
        multi_tool_stage_ids.difference(&governed_stage_ids).cloned().collect::<Vec<_>>();
    if !missing_contract_stage_ids.is_empty() {
        missing_contract_stage_ids.sort();
        return Err(anyhow!(
            "VCF comparable metric contract is missing multi-tool stages: {}",
            missing_contract_stage_ids.join(", ")
        ));
    }

    let mut rows = Vec::new();
    for stage_id in multi_tool_stage_ids {
        let stage = VcfDomainStage::try_from(stage_id.as_str())
            .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
        let tools_covered = stage_tools
            .get(stage_id.as_str())
            .ok_or_else(|| anyhow!("missing retained tool coverage for `{stage_id}`"))?
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let specs = stage_comparable_metric_specs(stage);
        if specs.is_empty() {
            return Err(anyhow!(
                "VCF comparable metric contract for `{stage_id}` must publish at least one governed metric"
            ));
        }
        for spec in specs {
            rows.push(VcfComparableMetricsRow {
                stage_id: stage_id.clone(),
                metric_id: spec.metric_id.to_string(),
                metric_name: spec.metric_name.to_string(),
                unit: spec.unit.to_string(),
                direction: comparable_metric_direction(spec.direction),
                required: spec.required,
                tools_covered: tools_covered.clone(),
            });
        }
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.metric_id.cmp(&right.metric_id))
    });
    Ok(rows)
}
pub(super) fn comparable_metric_direction(
    direction: VcfComparableMetricDirection,
) -> ComparableMetricDirection {
    match direction {
        VcfComparableMetricDirection::ExactMatchPreferred => {
            ComparableMetricDirection::ExactMatchPreferred
        }
        VcfComparableMetricDirection::HigherIsBetter => ComparableMetricDirection::HigherIsBetter,
        VcfComparableMetricDirection::LowerIsBetter => ComparableMetricDirection::LowerIsBetter,
    }
}
