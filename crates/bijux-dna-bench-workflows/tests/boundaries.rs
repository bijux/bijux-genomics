use anyhow::{Context, Result};
#[test]
fn default_workflow_compilation_does_not_reverse_into_cli_or_benchmark_aggregation() -> Result<()> {
    let manifest: toml::Value = toml::from_str(include_str!("../Cargo.toml"))?;
    let deps = manifest["dependencies"].as_table().context("dependencies")?;
    for name in ["bijux-dna", "bijux-dna-api", "bijux-dna-bench", "bijux-dna-bench-model"] {
        assert!(!deps.contains_key(name), "unexpected workflow dependency {name}");
    }
    for name in [
        "bijux-dna-domain-bam",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-vcf",
        "bijux-dna-stages-vcf",
        "bijux-dna-db-ref",
    ] {
        assert_eq!(deps[name]["optional"].as_bool(), Some(true));
    }
    assert!(manifest["features"]["default"].as_array().context("default features")?.is_empty());
    Ok(())
}
#[cfg(feature = "genomics")]
#[test]
fn germline_binding_admission_is_complete_and_has_no_fixture_effects() -> Result<()> {
    use bijux_dna_bench_workflows::inputs::{
        AlignmentFixtureOperations, CoreGermlineWorkflowInputs,
    };
    fn encode(_: &std::path::Path, _: &std::path::Path, _: &std::path::Path) -> Result<()> {
        panic!("unexpected fixture effect")
    }
    fn intervals(
        _: &std::path::Path,
        _: bijux_dna_bench_workflows::inputs::AlignmentFixtureFormat,
        _: &std::path::Path,
    ) -> Result<()> {
        panic!("unexpected fixture effect")
    }
    let ports = AlignmentFixtureOperations::new(encode, intervals);
    let pairs: Vec<_> = ["vcf.call", "vcf.filter", "vcf.stats", "vcf.qc"]
        .into_iter()
        .map(|stage| (stage.to_string(), "bcftools".to_string()))
        .collect();
    assert_eq!(CoreGermlineWorkflowInputs::new(ports, pairs.clone())?.vcf_tools().len(), 4);
    for invalid in 0..5 {
        let mut input = pairs.clone();
        match invalid {
            0 => {
                input.pop();
            }
            1 => input.push(input[0].clone()),
            2 => input[0].0 = "bam.qc".to_string(),
            3 => input[0].1 = " ".to_string(),
            _ => input[0].1 = "bcftools ".to_string(),
        }
        assert!(CoreGermlineWorkflowInputs::new(ports, input).is_err());
    }
    Ok(())
}
