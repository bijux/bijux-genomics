#![allow(non_snake_case)]
#[path = "../../../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__domain_catalog_symmetry_policy__all_domains_export_catalogs_and_presets() {
    let root = support::workspace_root();
    let checks = [
        (
            "crates/bijux-dna-domain-fastq/src/lib.rs",
            [
                "FASTQ_STAGE_ID_CATALOG",
                "FASTQ_PARAMS_CATALOG",
                "FASTQ_METRICS_CATALOG",
                "FastqInvariantsPreset",
            ],
        ),
        (
            "crates/bijux-dna-domain-bam/src/lib.rs",
            [
                "BAM_STAGE_ID_CATALOG",
                "BAM_PARAMS_CATALOG",
                "BAM_METRICS_CATALOG",
                "BamInvariantsPreset",
            ],
        ),
        (
            "crates/bijux-dna-domain-vcf/src/lib.rs",
            [
                "VCF_STAGE_ID_CATALOG",
                "VCF_PARAMS_CATALOG",
                "VCF_METRICS_CATALOG",
                "VcfInvariantsPreset",
            ],
        ),
    ];
    let mut missing = Vec::new();
    for (path, symbols) in checks {
        let full = root.join(path);
        let raw = std::fs::read_to_string(&full)
            .unwrap_or_else(|_| panic!("read source file {}", full.display()));
        for symbol in symbols {
            if !raw.contains(symbol) {
                missing.push(format!("{path} missing `{symbol}`"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "domain catalog symmetry violations:\n{}",
        missing.join("\n")
    );
}
