use crate::inputs::GenotypeBenchmarkInputs;

#[test]
fn declarations_reject_foreign_duplicate_adapter_parser_and_output_drift() {
    let rows = crate::test_support::rows();
    let mut duplicate = rows.clone();
    duplicate.push(rows[0].clone());
    assert!(GenotypeBenchmarkInputs::new(duplicate).is_err());
    for field in ["stage", "adapter", "parser", "tool", "outputs"] {
        let mut changed = rows.clone();
        match field {
            "stage" => changed[0].stage_id = "vcf.phasing".to_owned(),
            "adapter" => changed[0].adapter_id = "foreign".to_owned(),
            "parser" => changed[0].parser_id = "foreign".to_owned(),
            "tool" => changed[0].tool_id = "../bcftools".to_owned(),
            _ => {
                let duplicate = changed[0].expected_outputs[0].clone();
                changed[0].expected_outputs.push(duplicate);
            }
        }
        assert!(GenotypeBenchmarkInputs::new(changed).is_err(), "{field}");
    }
}

#[test]
fn wrong_binding_and_output_role_refuse_before_fixture_or_output_effects() -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let inputs = crate::test_support::inputs();
    assert!(
        crate::call::run_local_vcf_call_smoke(&inputs, repository.path(), "unregistered").is_err()
    );
    let mut rows = crate::test_support::rows();
    for row in &mut rows {
        row.expected_outputs = vec!["foreign_role".to_owned()];
    }
    let inputs = GenotypeBenchmarkInputs::new(rows)?;
    assert!(crate::call::run_local_vcf_call_smoke(&inputs, repository.path(), "bcftools").is_err());
    assert!(crate::gl_propagation::run_local_vcf_gl_propagation_smoke(
        &inputs,
        repository.path(),
        "bcftools"
    )
    .is_err());
    assert_eq!(std::fs::read_dir(repository.path())?.count(), 0);
    Ok(())
}

#[test]
fn likelihood_publication_refuses_an_unowned_report_without_replacing_it() -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let output =
        repository.path().join("artifacts/runs/bench/local-smoke/vcf.gl_propagation/bcftools");
    bijux_dna_infra::ensure_dir(&output)?;
    std::fs::write(output.join("metrics.json"), b"user evidence")?;
    assert!(crate::gl_propagation::run_local_vcf_gl_propagation_smoke(
        &crate::test_support::inputs(),
        repository.path(),
        "bcftools"
    )
    .is_err());
    assert_eq!(std::fs::read(output.join("metrics.json"))?, b"user evidence");
    Ok(())
}
