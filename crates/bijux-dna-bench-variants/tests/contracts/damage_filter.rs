use crate::damage_filter::admission::bindings::resolve_governed_vcf_damage_filter_smoke_contract;
use crate::damage_filter::fixtures::inputs::write_governed_damage_filter_input_vcf;
use crate::damage_filter::observations::variants::format_damage_context_rule;
use crate::damage_filter::observations::variants::parse_vcf_record_count;

use std::path::PathBuf;

#[test]
fn governed_damage_filter_contract_uses_single_sample_matrix_row() {
    let contract = resolve_governed_vcf_damage_filter_smoke_contract(
        &crate::test_support::inputs(),
        "bcftools",
    )
    .expect("resolve governed damage filter contract");
    assert_eq!(contract.stage_id, "vcf.damage_filter");
    assert_eq!(contract.tool_id, "bcftools");
    assert_eq!(contract.corpus_id, "vcf_production_regression");
    assert_eq!(contract.input_fixture_id, "terminal_damage_single_sample");
    assert_eq!(contract.sample_name, "sample_a");
}

#[test]
fn governed_damage_filter_fixture_contains_four_records() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_vcf = dir.path().join("damage_input.vcf");
    write_governed_damage_filter_input_vcf(&input_vcf).expect("write governed input");
    let record_count = parse_vcf_record_count(&input_vcf).expect("count records");
    assert_eq!(record_count, 4);
}

#[test]
fn damage_context_rule_uses_stage_thresholds() {
    let summary = serde_json::json!({
        "masking_strategy": {"mode": "remove"},
        "thresholds": {
            "max_damage_ratio": 0.35,
            "terminal_damage_threshold": 0.50,
            "pmd_min": 3.0
        }
    });
    let rule = format_damage_context_rule(&summary).expect("format damage rule");
    assert_eq!(rule, "remove_ct_ga_with_ratio_gt_0.35_or_terminal_signal_ge_0.50_or_pmd_lt_3.0");
}

fn repo_root() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .parent()
        .expect("workspace")
        .to_path_buf()
}

#[test]
fn parse_vcf_record_count_reads_governed_single_sample_fixture() {
    let repo_root = repo_root();
    let fixture_vcf = repo_root
        .join("benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf");
    let record_count = parse_vcf_record_count(&fixture_vcf).expect("count fixture records");
    assert_eq!(record_count, 2);
}
