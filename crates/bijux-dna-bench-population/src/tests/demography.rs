#![cfg(test)]
//! Retained demography parser and service assertions.
use crate::demography::admission::binding::resolve_governed_vcf_demography_smoke_contract;
use crate::demography::observations::evidence::load_demography_evidence;
use crate::demography::LocalVcfDemographyEstimate;

#[test]
fn governed_vcf_demography_smoke_contract_matches_stage_matrix() {
    let contract =
        resolve_governed_vcf_demography_smoke_contract(&crate::test_support::inputs(), "ibdne")
            .expect("resolve contract");
    assert_eq!(contract.stage, "vcf.demography");
    assert_eq!(contract.tool, "ibdne");
    assert_eq!(contract.corpus, "vcf_production_regression");
    assert_eq!(contract.input_fixture, "vcf_mini_multisample_cohort");
}

#[test]
fn demography_evidence_requires_aligned_contract_and_metrics() {
    let dir = tempfile::tempdir().expect("tempdir");
    let contract_path = dir.path().join("contract.json");
    let metrics_path = dir.path().join("metrics.json");
    let rows = vec![serde_json::json!({
        "generation": 5,
        "ne": 1000.0,
        "ci_low": 850.0,
        "ci_high": 1150.0
    })];
    let payload = serde_json::json!({
        "method": "ibdne",
        "inference_status": "fallback_estimate",
        "status": "complete",
        "insufficient_data_reason": serde_json::Value::Null,
        "time_bins": [5],
        "ne_estimates": rows,
        "tool_attempts": { "ibdne": false }
    });
    bijux_dna_infra::atomic_write_json(&contract_path, &payload).expect("write contract");
    bijux_dna_infra::atomic_write_json(&metrics_path, &payload).expect("write metrics");
    let evidence = load_demography_evidence(&contract_path, &metrics_path).expect("load evidence");
    assert_eq!(evidence.method, "ibdne");
    assert_eq!(evidence.status, "complete");
    assert_eq!(evidence.time_bins, vec![5]);
    assert_eq!(
        evidence.ne_estimates,
        vec![LocalVcfDemographyEstimate {
            time_bin: 5,
            ne: 1000.0,
            ci_low: 850.0,
            ci_high: 1150.0
        }]
    );
}
