use bijux_dna_policies::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}

#[test]
fn planner_fastq_layout_matches_documented_architecture() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let expected_files = [
        "surface.rs",
        "preprocess/mod.rs",
        "preprocess/planning.rs",
        "preprocess/policy.rs",
        "selection/facade.rs",
        "compose/mod.rs",
        "compose/input_resolution.rs",
        "compose/models.rs",
        "compose/stage_params.rs",
        "tool_adapters/stages/transform/trim_reads/mod.rs",
        "tool_adapters/stages/transform/trim_reads/config.rs",
        "tool_adapters/stages/transform/trim_reads/reporting.rs",
    ];
    let legacy_files = [
        "preprocess_planning.rs",
        "preprocess_policy.rs",
        "plan_compose.rs",
        "tool_selection_facade.rs",
        "tool_adapters/stages/transform/trim_reads.rs",
    ];

    for relative_path in expected_files {
        assert!(
            root.join(relative_path).is_file(),
            "expected architecture file is missing: {relative_path}"
        );
    }

    for relative_path in legacy_files {
        assert!(
            !root.join(relative_path).exists(),
            "legacy architecture file should be absent: {relative_path}"
        );
    }
}
