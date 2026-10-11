use anyhow::{bail, Result};
use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations,
    evidence::adapters::{
        render_vcf_adapters_ready, ShellSyntaxObservation, VcfAdapterReadinessPorts,
    },
};
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBindingKey;
use std::path::Path;
fn unavailable_bindings(_: &Path) -> Result<Vec<VcfStageReadinessBindingKey>> {
    bail!("bindings unavailable")
}
fn forbidden_syntax(_: &Path, _: &Path) -> Result<ShellSyntaxObservation> {
    panic!("syntax inspection must follow admitted command publication")
}
#[test]
fn failed_adapter_inputs_publish_all_checks_without_invoking_shell() -> Result<()> {
    let inputs = crate::support::inputs(&crate::support::repo_root());
    let root = tempfile::tempdir()?;
    let observations = VcfAdapterObservations::new(&inputs, root.path());
    let output = root.path().join("readiness.json");
    let ports = VcfAdapterReadinessPorts {
        active_bindings: unavailable_bindings,
        inspect_shell: forbidden_syntax,
    };
    assert!(render_vcf_adapters_ready("fixture-run", &observations, &ports, &output).is_err());
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(output)?)?;
    let checks = report["checks"].as_array().expect("all checks");
    assert_eq!(
        checks.iter().map(|check| check["goal_id"].as_u64().expect("identity")).collect::<Vec<_>>(),
        (231..=245).collect::<Vec<_>>()
    );
    assert_eq!(report["ok"], false);
    assert_eq!(report["checked_goal_count"], 15);
    assert_eq!(
        checks[0]["ok"], true,
        "catalog-derived serving remains independent of absent files"
    );
    Ok(())
}
