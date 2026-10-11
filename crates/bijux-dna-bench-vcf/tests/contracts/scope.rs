use anyhow::Result;
use bijux_dna_bench_vcf::evidence::scope::{
    collect_vcf_active_stage_tool_matrix_rows_from_expected, render_vcf_active_stage_tool_matrix,
};
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBindingKey;

#[test]
fn foreign_or_repeated_active_binding_refuses_before_registry_acquisition() -> Result<()> {
    let inputs = crate::support::inputs(&crate::support::repo_root());
    let root = tempfile::tempdir()?;
    let row = &inputs.bindings()[0];
    let key = VcfStageReadinessBindingKey {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    };
    let mut foreign = key.clone();
    foreign.corpus_id = "foreign".to_owned();
    for active in [vec![foreign], vec![key.clone(), key]] {
        let error = collect_vcf_active_stage_tool_matrix_rows_from_expected(
            &inputs,
            root.path(),
            &active,
            &[],
        )
        .err()
        .map(|error| error.to_string());
        assert!(error.is_some_and(|message| message.contains("foreign or repeated active binding")));
        assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    }
    Ok(())
}

#[test]
fn ambiguous_pair_authority_refuses_before_registry_acquisition() -> Result<()> {
    let base = crate::support::inputs(&crate::support::repo_root());
    let mut bindings = base.bindings().to_vec();
    let mut duplicate = bindings[0].clone();
    duplicate.asset_profile_id = "another-asset".to_owned();
    bindings.push(duplicate);
    let inputs = bijux_dna_bench_vcf::inputs::VcfAdapterInputs::new(
        base.catalog().to_vec(),
        bindings,
        |_, _| anyhow::bail!("unexpected reference effect"),
    )?;
    let root = tempfile::tempdir()?;
    let error =
        collect_vcf_active_stage_tool_matrix_rows_from_expected(&inputs, root.path(), &[], &[])
            .err()
            .map(|error| error.to_string());
    assert!(error.is_some_and(|message| message.contains("ambiguous corpus/asset authority")));
    assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    Ok(())
}

#[test]
fn non_string_retained_stage_refuses_without_replacing_report() -> Result<()> {
    let inputs = crate::support::inputs(&crate::support::repo_root());
    let root = tempfile::tempdir()?;
    let registry = root.path().join("configs/ci/registry");
    std::fs::create_dir_all(&registry)?;
    std::fs::write(
        registry.join("tool_registry_vcf.toml"),
        "[[tools]]\nid = \"bcftools\"\nstatus = \"production\"\nstage_ids = [3]\n",
    )?;
    let output = root.path().join("scope.tsv");
    std::fs::write(&output, b"retained scope evidence")?;
    let error = render_vcf_active_stage_tool_matrix(&inputs, root.path(), &[], &[], &output)
        .err()
        .map(|error| format!("{error:#}"));
    assert!(error
        .is_some_and(|message| message.contains("invalid type") && message.contains("integer")));
    assert_eq!(std::fs::read(&output)?, b"retained scope evidence");
    Ok(())
}
