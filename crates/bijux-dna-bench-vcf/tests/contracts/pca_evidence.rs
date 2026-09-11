use anyhow::Result;
use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations,
    evidence::pca::{prepare_pca_adapter_evidence, publish_pca_commands},
    inputs::VcfAdapterInputs,
};
#[test]
fn ambiguous_or_absent_pca_binding_refuses_before_acquisition() -> Result<()> {
    let source = crate::support::repo_root();
    let base = crate::support::inputs(&source);
    for ambiguous in [false, true] {
        let mut rows = base.bindings().to_vec();
        if ambiguous {
            let mut repeated = rows
                .iter()
                .find(|row| row.stage_id == "vcf.pca" && row.tool_id == "plink2")
                .expect("PCA binding")
                .clone();
            repeated.corpus_id = "foreign-corpus".to_owned();
            rows.push(repeated);
        } else {
            rows.retain(|row| row.stage_id != "vcf.pca" || row.tool_id != "plink2");
        }
        let inputs = VcfAdapterInputs::new(
            base.catalog().to_vec(),
            rows,
            crate::support::reference_fixture,
        )?;
        let root = tempfile::tempdir()?;
        let observations = VcfAdapterObservations::new(&inputs, root.path());
        let error =
            prepare_pca_adapter_evidence(&observations).err().map(|error| error.to_string());
        assert!(error.is_some_and(|message| message.contains("exactly one full binding")));
        assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    }
    Ok(())
}
#[test]
fn pca_publication_keeps_structured_commands_and_bash_syntax() -> Result<()> {
    let source = crate::support::repo_root();
    let inputs = crate::support::inputs(&source);
    let observations = VcfAdapterObservations::new(&inputs, &source);
    let prepared = prepare_pca_adapter_evidence(&observations)?;
    assert_eq!(prepared.rows().len(), 2);
    let output = tempfile::tempdir()?;
    let report = publish_pca_commands(output.path(), &prepared)?;
    let script = output.path().join(&report.output_path);
    let argv = output.path().join(&report.argv_output_path);
    assert!(std::process::Command::new("bash").arg("-n").arg(&script).status()?.success());
    let script = std::fs::read_to_string(script)?;
    let records = std::fs::read_to_string(argv)?
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(records.len(), 2);
    for observation in prepared.rows() {
        let record = records
            .iter()
            .find(|record| record["tool_id"] == observation.binding.tool_id)
            .expect("PCA argv row");
        assert_eq!(record["stage_id"], "vcf.pca");
        assert_eq!(
            record["command_steps"].as_array().expect("steps").len(),
            observation.command.command_steps.len()
        );
        for (step, record) in observation
            .command
            .command_steps
            .iter()
            .zip(record["command_steps"].as_array().expect("steps"))
        {
            assert_eq!(record["argv"], serde_json::to_value(&step.argv)?);
            assert!(script.contains(&step.command));
        }
        assert!(!observation.output.manifest_output.is_empty());
        assert!(!observation.schema_id.is_empty());
    }
    Ok(())
}
