use anyhow::Result;

#[test]
fn cli_forbids_internal_deps() -> Result<()> {
    let manifest = super::support::crate_root("bijux-dna")?.join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest)?;
    let mut in_deps = false;
    let mut deps = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps =
                matches!(line, "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]");
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            let name = name.trim().trim_matches('"');
            deps.push(name.strip_suffix(".workspace").unwrap_or(name).to_string());
        }
    }
    let forbidden = [
        "bijux-dna-domain-bam",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-vcf",
        "bijux-dna-engine",
        "bijux-dna-pipelines",
        "bijux-dna-runner",
        "bijux-dna-runtime",
        "bijux-dna-stages-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-vcf",
    ];
    for dep in forbidden {
        assert!(!deps.contains(&dep.to_string()), "cli must not depend on {dep}");
    }
    assert!(
        deps.contains(&"bijux-dna-api".to_string()),
        "cli must enter domain, planning, runtime, and report behavior through bijux-dna-api"
    );
    Ok(())
}
