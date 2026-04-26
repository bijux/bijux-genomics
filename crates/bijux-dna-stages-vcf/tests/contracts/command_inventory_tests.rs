#[test]
fn command_inventory_lists_all_stages_vcf_operations() {
    let commands = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/COMMANDS.md"),
    )
    .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));

    let listed = command_names(&commands);
    let expected = expected_command_names();

    assert_eq!(
        listed, expected,
        "docs/COMMANDS.md must be the exact SSOT for stages-vcf operations"
    );
}

fn command_names(commands: &str) -> std::collections::BTreeSet<String> {
    commands
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let first_cell = line.strip_prefix("| `")?.split_once('`')?.0;
            Some(first_cell.to_string())
        })
        .collect()
}

fn expected_command_names() -> std::collections::BTreeSet<String> {
    [
        "check-vcf-reference-match",
        "check-vcf-stage-completeness",
        "compute-vcf-checksum-set",
        "compute-vcf-panel-overlap",
        "compute-vcf-stats-basic",
        "concat-vcf",
        "extract-vcf-region",
        "index-vcf-bgzip-tabix",
        "list-vcf-implemented-stages",
        "list-vcf-stage-catalog",
        "list-vcf-supported-stages",
        "normalize-vcf-headers",
        "parse-vcf-call-summary",
        "parse-vcf-filter-breakdown",
        "parse-vcf-stats",
        "read-vcf-text",
        "run-vcf-admixture-stage",
        "run-vcf-call-diploid-stage",
        "run-vcf-call-gl-stage",
        "run-vcf-call-pseudohaploid-stage",
        "run-vcf-chunked-regions",
        "run-vcf-damage-filter-stage",
        "run-vcf-demography-stage",
        "run-vcf-filter-stage",
        "run-vcf-gl-propagation-stage",
        "run-vcf-ibd-stage",
        "run-vcf-imputation-orchestration-stage",
        "run-vcf-impute-stage",
        "run-vcf-pca-stage",
        "run-vcf-phasing-stage",
        "run-vcf-pipeline",
        "run-vcf-population-structure-stage",
        "run-vcf-postprocess-stage",
        "run-vcf-preflight",
        "run-vcf-prepare-reference-panel-stage",
        "run-vcf-qc-stage",
        "run-vcf-roh-stage",
        "run-vcf-stats-stage",
        "split-vcf-by-chrom",
        "summarize-vcf-metrics",
        "validate-vcf-input",
        "verify-vcf-tool-wrapper",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
