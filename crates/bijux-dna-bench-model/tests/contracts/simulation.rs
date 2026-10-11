use super::{
    all_domain_simulation_output_path as output_relative_path,
    all_domain_simulation_output_role as output_role,
};

#[test]
fn output_relative_path_keeps_expected_all_domain_shapes() {
    assert_eq!(output_relative_path("called_vcf").to_string_lossy(), "called.vcf");
    assert_eq!(output_relative_path("called_vcf_tbi").to_string_lossy(), "called.vcf.tbi");
    assert_eq!(
        output_relative_path("classification_report_json").to_string_lossy(),
        "classification_report.json"
    );
    assert_eq!(output_relative_path("qc_bundle").to_string_lossy(), "qc_bundle");
}

#[test]
fn output_role_classifies_all_domain_symbols() {
    assert_eq!(output_role("called_vcf"), "vcf");
    assert_eq!(output_role("called_vcf_tbi"), "index");
    assert_eq!(output_role("trim_metrics"), "metrics");
    assert_eq!(output_role("classification_table"), "table");
    assert_eq!(output_role("qc_bundle"), "bundle");
}
