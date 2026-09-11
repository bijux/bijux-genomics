use super::checks::completeness::ensure_pair_sets_match;
use std::collections::BTreeSet;
#[test]
fn pair_completeness_rejects_missing_and_extra_tool_bindings() {
    let expected = BTreeSet::from([("vcf.qc".to_string(), "plink2".to_string())]);
    let missing = BTreeSet::new();
    assert!(ensure_pair_sets_match("parser coverage", &expected, &missing).is_err());
    let extra = BTreeSet::from([
        ("vcf.qc".to_string(), "plink2".to_string()),
        ("vcf.qc".to_string(), "foreign".to_string()),
    ]);
    assert!(ensure_pair_sets_match("report map", &expected, &extra).is_err());
    assert!(ensure_pair_sets_match("report map", &expected, &expected).is_ok());
}
