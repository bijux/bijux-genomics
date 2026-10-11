use anyhow::Result;
use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations, inputs::VcfAdapterInputs,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static REFERENCE_ACQUISITIONS: AtomicUsize = AtomicUsize::new(0);

fn count_reference(
    source: &std::path::Path,
    output: &std::path::Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    REFERENCE_ACQUISITIONS.fetch_add(1, Ordering::SeqCst);
    crate::support::reference_fixture(source, output)
}

#[test]
fn repeated_report_publication_reuses_acquired_fixture_observations() -> Result<()> {
    let root = crate::support::repo_root();
    let base = crate::support::inputs(&root);
    let inputs =
        VcfAdapterInputs::new(base.catalog().to_vec(), base.bindings().to_vec(), count_reference)?;
    let observations = VcfAdapterObservations::new(&inputs, &root);
    let report = observations.bcftools()?;
    let count = REFERENCE_ACQUISITIONS.load(Ordering::SeqCst);
    assert!(count > 0, "the reference effect must be exercised before testing reuse");
    assert!(std::ptr::eq(report, observations.bcftools()?));
    let output = tempfile::tempdir()?;
    let first = output.path().join("first.json");
    let second = output.path().join("second.json");
    let left = observations.publish_bcftools(&first)?;
    let right = observations.publish_bcftools(&second)?;
    assert_eq!(REFERENCE_ACQUISITIONS.load(Ordering::SeqCst), count);
    assert_eq!(serde_json::to_value(&left.rows)?, serde_json::to_value(&right.rows)?);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(&first)?)?,
        serde_json::to_value(&left)?
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(&second)?)?,
        serde_json::to_value(&right)?
    );
    assert_eq!(left.output_path, first.to_string_lossy());
    assert_eq!(right.output_path, second.to_string_lossy());
    Ok(())
}

#[test]
fn unknown_observation_tool_refuses_without_loading_a_repository() -> Result<()> {
    let root = crate::support::repo_root();
    let inputs = crate::support::inputs(&root);
    let directory = tempfile::tempdir()?;
    let observations = VcfAdapterObservations::new(&inputs, directory.path());
    let destination = directory.path().join("unpublished/report.json");
    let error =
        observations.publish_phasing("unknown", &destination).err().map(|error| error.to_string());
    assert!(
        error.is_some_and(|message| message.contains("unsupported VCF phasing observation tool"))
    );
    assert!(!destination.exists());
    assert!(!directory.path().join("unpublished").exists());
    Ok(())
}

#[test]
fn command_consumers_reuse_family_observations_and_preserve_pipeline_authority() -> Result<()> {
    use bijux_dna_bench_vcf::evidence::commands::collect_vcf_adapter_command_row_map;
    let root = crate::support::repo_root();
    let base = crate::support::inputs(&root);
    let inputs =
        VcfAdapterInputs::new(base.catalog().to_vec(), base.bindings().to_vec(), count_reference)?;
    let observations = VcfAdapterObservations::new(&inputs, &root);
    let first = collect_vcf_adapter_command_row_map(&observations)?;
    let effects = REFERENCE_ACQUISITIONS.load(Ordering::SeqCst);
    assert!(effects > 0);
    let second = collect_vcf_adapter_command_row_map(&observations)?;
    assert_eq!(REFERENCE_ACQUISITIONS.load(Ordering::SeqCst), effects);
    assert_eq!(
        serde_json::to_value(first.values().collect::<Vec<_>>())?,
        serde_json::to_value(second.values().collect::<Vec<_>>())?
    );
    let roh =
        first.get(&("vcf.roh".to_owned(), "plink2".to_owned())).expect("ROH command authority");
    assert_eq!(roh.command_source, "vcf_descent_family_adapter");
    let call = first.get(&("vcf.call".to_owned(), "bcftools".to_owned())).expect("call pipeline");
    assert!(call.command_steps.iter().any(|step| step.consumes_previous_stdout));
    assert!(call.script_commands.iter().any(|command| command.contains(" | ")));
    Ok(())
}
