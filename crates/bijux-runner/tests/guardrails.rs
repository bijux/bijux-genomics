use bijux_guardrails::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}

#[test]
fn runner_has_no_domain_keywords() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = crate_root.join("src");
    let denylist = [
        "fastq.",
        "bam.",
        "qc_",
        "qc.",
        "retention",
        "adapter",
        "tool_list",
        "normalize_",
    ];
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        for needle in &denylist {
            if content.contains(needle) {
                offenders.push(format!("{}::{needle}", entry.path().display()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "runner must not contain domain keywords or stage IDs: {offenders:?}"
    );
}
