use super::projection::comparable_metric_direction;
use bijux_dna_domain_vcf::VcfComparableMetricDirection;
#[test]
fn comparable_metric_direction_labels_are_stable() {
    assert_eq!(
        comparable_metric_direction(VcfComparableMetricDirection::ExactMatchPreferred).as_str(),
        "exact_match_preferred",
    );
    assert_eq!(
        comparable_metric_direction(VcfComparableMetricDirection::HigherIsBetter).as_str(),
        "higher_is_better",
    );
    assert_eq!(
        comparable_metric_direction(VcfComparableMetricDirection::LowerIsBetter).as_str(),
        "lower_is_better",
    );
}

#[test]
fn undeclared_multi_tool_stage_has_no_comparable_metric_contract() {
    let tools = std::collections::BTreeMap::from([(
        "unknown-stage".to_string(),
        std::collections::BTreeSet::from(["one".to_string(), "two".to_string()]),
    )]);
    let result = super::projection::collect_vcf_comparable_metric_rows(&tools);
    assert!(result.as_ref().err().is_some_and(|error| error
        .to_string()
        .contains("missing multi-tool stages: unknown-stage")));
}
