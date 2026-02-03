use std::path::Path;

#[test]
fn cli_forbids_internal_deps() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest).expect("read Cargo.toml");
    let mut in_deps = false;
    let mut deps = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = matches!(
                line,
                "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
            );
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            deps.push(name.trim().trim_matches('"').to_string());
        }
    }
    let forbidden = [
        "bijux-domain-fastq",
        "bijux-domain-bam",
        "bijux-domain-vcf",
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-engine",
        "bijux-pipelines",
    ];
    for dep in forbidden {
        assert!(
            !deps.contains(&dep.to_string()),
            "cli must not depend on {dep}"
        );
    }
}
