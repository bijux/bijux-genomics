use bijux_dna_policies::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = crate::support::crate_root("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(&crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}

#[test]
fn api_has_no_planning_policy_keywords() {
    let src_dir = crate::support::crate_src("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate src: {err}"));
    let denylist = ["smart_pipeline", "tool_list", "stage ordering", "bijux_exec"];
    let allowlist_paths = [
        "/src/explain.rs",
        "/src/internal/fastq/stages/preprocess.rs",
        "/src/internal/fastq/stages/stats_neutral.rs",
        "/src/internal/handlers/bam.rs",
        "/src/internal/handlers/cross/bam_exec.rs",
        "/src/internal/handlers/cross/bam_exec_contracts.rs",
        "/src/internal/handlers/cross/bam_exec_metrics_helpers.rs",
        "/src/internal/handlers/cross/bam_exec_stage_runtime.rs",
        "/src/internal/handlers/fastq/summary.rs",
        "/src/internal/handlers/fastq/summary_rendering.rs",
        "/src/internal/handlers/fastq/summary_contracts.rs",
    ];
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|err| panic!("read {}: {err}", entry.path().display()));
        let path_str = entry.path().to_string_lossy();
        if allowlist_paths.iter().any(|suffix| path_str.ends_with(suffix)) {
            continue;
        }
        for needle in &denylist {
            if content.contains(needle) {
                offenders.push(format!("{}::{needle}", entry.path().display()));
            }
        }
    }
    assert!(offenders.is_empty(), "API must not embed planning policy: {offenders:?}");
}
