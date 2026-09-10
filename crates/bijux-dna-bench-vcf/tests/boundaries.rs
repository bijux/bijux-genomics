//! Explicit input admission and dependency direction for VCF adapters.
use anyhow::{bail, Result};
use bijux_dna_bench_model::{VcfStageCatalogRow, VcfStageMatrixRow};
use bijux_dna_bench_vcf::inputs::{project_supported_vcf_bindings, VcfAdapterInputs};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
static EFFECTS: AtomicUsize = AtomicUsize::new(0);
fn unavailable_reference(_: &Path, _: &Path) -> Result<(PathBuf, PathBuf)> {
    EFFECTS.fetch_add(1, Ordering::SeqCst);
    bail!("reference authority unavailable")
}
fn catalog() -> VcfStageCatalogRow {
    VcfStageCatalogRow {
        stage_id: "vcf.qc".into(),
        stage_name: "QC".into(),
        support_status: "supported".into(),
        default_tool_id: "bcftools".into(),
        metrics_schema_id: "metrics.v1".into(),
        input_types: vec!["vcf".into()],
        output_types: vec!["metrics".into()],
        required_assets: vec![],
        benchmark_category: "qc".into(),
        local_smoke_mode: "fixture".into(),
    }
}
fn binding() -> VcfStageMatrixRow {
    VcfStageMatrixRow {
        stage_id: "vcf.qc".into(),
        tool_id: "bcftools".into(),
        corpus_id: "corpus".into(),
        asset_profile_id: "fixture".into(),
        adapter_id: "adapter".into(),
        parser_id: "parser".into(),
        expected_outputs: vec!["metrics".into()],
    }
}
#[test]
fn input_admission_rejects_invalid_observations_before_reference_effects() -> Result<()> {
    EFFECTS.store(0, Ordering::SeqCst);
    for rows in [vec![], vec![binding(), binding()]] {
        assert!(VcfAdapterInputs::new(vec![catalog()], rows, unavailable_reference).is_err());
    }
    for invalid in 0..7 {
        let mut row = binding();
        match invalid {
            0 => row.stage_id = "bam.qc".into(),
            1 => row.tool_id = " ".into(),
            2 => row.expected_outputs.clear(),
            3 => row.expected_outputs.push("metrics".into()),
            4 => row.tool_id = "bcftools ".into(),
            5 => row.expected_outputs = vec!["metrics ".into()],
            _ => row.expected_outputs = vec![" ".into()],
        }
        assert!(VcfAdapterInputs::new(vec![catalog()], vec![row], unavailable_reference).is_err());
    }
    let admitted = VcfAdapterInputs::new(vec![catalog()], vec![binding()], unavailable_reference)?;
    assert_eq!(EFFECTS.load(Ordering::SeqCst), 0);
    let error = admitted.materialize_reference_fasta(Path::new("input.fa"), Path::new("out"));
    assert!(error.is_err_and(|e| e.to_string() == "reference authority unavailable"));
    assert_eq!(EFFECTS.load(Ordering::SeqCst), 1);
    Ok(())
}
#[test]
fn supported_binding_projection_preserves_distinct_tools_and_refuses_aliasing() -> Result<()> {
    let pairs = vec![("vcf.qc".into(), "tool_b".into()), ("vcf.qc".into(), "tool_a".into())];
    let rows = project_supported_vcf_bindings(&[binding()], pairs)?;
    assert_eq!(rows.iter().map(|r| r.tool_id.as_str()).collect::<Vec<_>>(), ["tool_a", "tool_b"]);
    assert!(project_supported_vcf_bindings(&[binding(), binding()], vec![]).is_err());
    assert!(project_supported_vcf_bindings(
        &[binding()],
        vec![("vcf.qc".into(), "a".into()), ("vcf.qc".into(), "a".into())]
    )
    .is_err());
    assert!(project_supported_vcf_bindings(&[binding()], vec![("vcf.unknown".into(), "a".into())])
        .is_err());
    Ok(())
}
#[test]
fn normal_dependencies_do_not_reverse_into_composition_or_activate_fastq() -> Result<()> {
    let manifest: toml::Value = toml::from_str(include_str!("../Cargo.toml"))?;
    let dependencies = manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| anyhow::anyhow!("missing dependencies"))?;
    for name in [
        "bijux-dna",
        "bijux-dna-api",
        "bijux-dna-bench",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-fastq",
    ] {
        assert!(!dependencies.contains_key(name), "unexpected {name}");
    }
    assert_eq!(dependencies["bijux-dna-bench-model"]["default-features"].as_bool(), Some(false));
    Ok(())
}
