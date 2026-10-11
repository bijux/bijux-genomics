use anyhow::{bail, Result};
use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations,
    evidence::parsers::readiness::render_vcf_parsers_report_ready,
};
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBindingKey;
use std::{
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};
static PROVIDER_CALLS: AtomicUsize = AtomicUsize::new(0);
fn unavailable_binding_provider(_: &Path) -> Result<Vec<VcfStageReadinessBindingKey>> {
    PROVIDER_CALLS.fetch_add(1, Ordering::SeqCst);
    bail!("independent active bindings unavailable")
}
#[test]
fn unavailable_bindings_preserve_all_independent_check_results_before_returning_failure(
) -> Result<()> {
    let inputs = crate::support::inputs(&crate::support::repo_root());
    let root = tempfile::tempdir()?;
    let observations = VcfAdapterObservations::new(&inputs, root.path());
    let output = root.path().join("parser-readiness.json");
    let error =
        render_vcf_parsers_report_ready(&observations, unavailable_binding_provider, &output)
            .expect_err("unavailable bindings and fixtures");
    assert!(error.to_string().contains("VCF parser/report readiness gate failed"));
    assert_eq!(PROVIDER_CALLS.load(Ordering::SeqCst), 1);
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(output)?)?;
    let checks = report["checks"].as_array().expect("all independent checks");
    assert_eq!(
        checks
            .iter()
            .map(|check| check["goal_id"].as_u64().expect("check identity"))
            .collect::<Vec<_>>(),
        (246..=260).collect::<Vec<_>>()
    );
    assert_eq!(checks[0]["ok"], true, "schema generation does not depend on active bindings");
    assert_eq!(checks[10]["ok"], false);
    assert!(checks[10]["detail"]
        .as_str()
        .expect("coverage diagnostic")
        .contains("independent active bindings unavailable"));
    assert!(checks[14]["detail"]
        .as_str()
        .expect("completeness diagnostic")
        .contains("parser coverage report was not produced"));
    assert_eq!(report["checked_goal_count"], 15);
    assert_eq!(report["ok"], false);
    Ok(())
}
