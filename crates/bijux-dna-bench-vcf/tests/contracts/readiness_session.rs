use anyhow::{bail, Result};
use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations,
    evidence::{
        expectations::collect_vcf_expected_benchmark_result_rows,
        readiness::VcfReadinessEvidenceSession,
        scope::collect_vcf_active_stage_tool_matrix_rows_from_expected,
    },
    inputs::VcfAdapterInputs,
};
use bijux_dna_bench_vcf_evidence::snapshot::{ActiveVcfBinding, VcfStageReadinessBindingKey};
use std::{
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};
static INPUT_CALLS: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_CALLS: AtomicUsize = AtomicUsize::new(0);
static FAILED_INPUT_CALLS: AtomicUsize = AtomicUsize::new(0);
#[allow(
    clippy::unnecessary_wraps,
    reason = "Test double implements the fallible input authority port."
)]
fn counted_inputs(root: &Path) -> Result<VcfAdapterInputs> {
    INPUT_CALLS.fetch_add(1, Ordering::Relaxed);
    Ok(crate::support::inputs(root))
}
fn counted_active(root: &Path) -> Result<Vec<ActiveVcfBinding>> {
    ACTIVE_CALLS.fetch_add(1, Ordering::Relaxed);
    let inputs = crate::support::inputs(root);
    let observations = VcfAdapterObservations::new(&inputs, root);
    let expected = collect_vcf_expected_benchmark_result_rows(&observations)?;
    let active = expected
        .iter()
        .map(|row| VcfStageReadinessBindingKey {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id.clone(),
            asset_profile_id: row.asset_profile_id.clone(),
        })
        .collect::<Vec<_>>();
    // Test composition projects full identities and contracts from admitted fixture declarations.
    Ok(collect_vcf_active_stage_tool_matrix_rows_from_expected(&inputs, root, &active, &expected)?
        .into_iter()
        .filter(|row| row.scope_state == "active")
        .map(|row| ActiveVcfBinding {
            key: VcfStageReadinessBindingKey {
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                corpus_id: row.corpus_id,
                asset_profile_id: row.asset_profile_id,
            },
            adapter_id: row.adapter_id,
            parser_id: row.parser_id,
            schema_id: row.schema_id,
        })
        .collect())
}
#[test]
fn repeated_stage_requests_borrow_one_admitted_snapshot_without_a_global_cache() -> Result<()> {
    let root = crate::support::repo_root();
    let session = VcfReadinessEvidenceSession::new(
        &root,
        "readiness-session-contract",
        counted_inputs,
        counted_active,
    );
    let first = session.evidence()?;
    let second = session.evidence()?;
    assert!(std::ptr::eq(first, second));
    assert_eq!(INPUT_CALLS.load(Ordering::Relaxed), 1);
    assert_eq!(ACTIVE_CALLS.load(Ordering::Relaxed), 1);
    assert!(!first.1.stage_bindings("vcf.call")?.is_empty());
    assert!(!second.1.stage_bindings("vcf.impute")?.is_empty());
    let independent = VcfReadinessEvidenceSession::new(
        &root,
        "readiness-session-contract",
        counted_inputs,
        counted_active,
    );
    let fresh = independent.evidence()?;
    assert!(!std::ptr::eq(first, fresh));
    assert_eq!(INPUT_CALLS.load(Ordering::Relaxed), 2);
    assert_eq!(ACTIVE_CALLS.load(Ordering::Relaxed), 2);
    Ok(())
}
fn failed_inputs(_: &Path) -> Result<VcfAdapterInputs> {
    FAILED_INPUT_CALLS.fetch_add(1, Ordering::Relaxed);
    bail!("input unavailable")
}
fn unreachable_active(_: &Path) -> Result<Vec<ActiveVcfBinding>> {
    bail!("active provider must not run before input admission")
}
#[test]
fn input_failure_is_reported_again_and_never_becomes_cached_success() -> Result<()> {
    let root = tempfile::tempdir()?;
    let session = VcfReadinessEvidenceSession::new(
        root.path(),
        "readiness-session-contract",
        failed_inputs,
        unreachable_active,
    );
    for _ in 0..2 {
        let result = session.evidence();
        assert!(result.err().is_some_and(|error| error.to_string() == "input unavailable"));
    }
    assert_eq!(FAILED_INPUT_CALLS.load(Ordering::Relaxed), 2);
    assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    Ok(())
}
