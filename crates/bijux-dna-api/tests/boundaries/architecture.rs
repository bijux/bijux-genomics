use std::collections::BTreeSet;

#[test]
fn api_tree_matches_architecture_contract() {
    let root = crate::support::crate_root("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    assert_root_tree(&root);
    assert_docs_tree(&root);
    assert_runtime_tree(&root);
    assert_support_tree(&root);
    assert_v1_tree(&root);
    assert_test_tree(&root);
}

#[test]
fn manifest_dependency_graph_has_no_duplicate_edges() {
    let root = crate::support::crate_root("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let manifest = root.join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest.display()));
    let parsed: toml::Value = toml::from_str(&content)
        .unwrap_or_else(|err| panic!("parse {}: {err}", manifest.display()));

    let dependencies = dependency_names(&parsed, "dependencies");
    let dev_dependencies = dependency_names(&parsed, "dev-dependencies");
    let duplicates = dependencies.intersection(&dev_dependencies).cloned().collect::<Vec<_>>();

    assert!(
        duplicates.is_empty(),
        "normal and dev dependencies must not duplicate edges: {duplicates:?}",
    );
    assert!(
        !dependencies.contains("bijux-dna-bench"),
        "bijux-dna-api must not depend on bijux-dna-bench unless source code imports its API",
    );
}

fn assert_root_tree(root: &std::path::Path) {
    assert_dir_entries(
        root,
        &["Cargo.toml", "README.md", "docs/", "src/", "tests/"],
        "api crate root must stay minimal and intentional",
    );
    assert_dir_entries(
        &root.join("src"),
        &["internal/", "lib.rs", "runtime/", "support/", "surface/", "v1/"],
        "api src tree must match the documented architecture",
    );
    assert_dir_entries(
        &root.join("src/surface"),
        &["explain.rs", "mod.rs", "request_contracts.rs", "versioning.rs"],
        "api surface tree must stay focused on stable contracts",
    );
}

fn assert_docs_tree(root: &std::path::Path) {
    assert_dir_entries(
        &root.join("docs"),
        &[
            "API.md",
            "ARCHITECTURE.md",
            "BOUNDARY.md",
            "CHANGE_RULES.md",
            "COMMANDS.md",
            "FEATURES.md",
            "PUBLIC_API.md",
            "REQUEST_FLOW.md",
            "RUNTIME_OPERATOR_COMMANDS.md",
            "SECURITY.md",
            "TESTS.md",
        ],
        "api docs must stay in the governed crate docs directory",
    );

    let misplaced_docs = markdown_files_below(&root.join("tests"));
    assert!(
        misplaced_docs.is_empty(),
        "test documentation belongs in docs/TESTS.md, found: {misplaced_docs:?}",
    );
}

fn assert_runtime_tree(root: &std::path::Path) {
    assert_dir_entries(
        &root.join("src/runtime"),
        &[
            "cross_runtime.rs",
            "execution_kernel.rs",
            "invocation_policy/",
            "invocation_policy.rs",
            "mod.rs",
            "persistence.rs",
            "run/",
            "validation.rs",
        ],
        "api runtime tree must stay decomposed by enduring concern",
    );
    assert_dir_entries(
        &root.join("src/runtime/invocation_policy"),
        &["config.rs", "contracts.rs", "models.rs", "resilience.rs"],
        "api invocation policy support tree must stay explicit",
    );
    assert_dir_entries(
        &root.join("src/runtime/run"),
        &["execution/", "execution_support.rs", "mod.rs", "planning/", "reporting/"],
        "api runtime run tree must separate execution, planning, and reporting",
    );
    assert_dir_entries(
        &root.join("src/runtime/run/planning"),
        &["mod.rs", "planning_support.rs", "profile_selection.rs", "run_bootstrap.rs"],
        "api runtime planning tree must separate selection, bootstrap, and planning support",
    );
    assert_dir_entries(
        &root.join("src/runtime/run/execution"),
        &["mod.rs", "stage_execution.rs"],
        "api runtime execution tree must keep the execution entry explicit",
    );
}

fn assert_support_tree(root: &std::path::Path) {
    assert_dir_entries(
        &root.join("src/support"),
        &[
            "benchmark_runtime.rs",
            "mod.rs",
            "qa/",
            "reference_resolution/",
            "tool_selection.rs",
            "workspace/",
        ],
        "api support tree must stay partitioned by concern",
    );
    assert_dir_entries(
        &root.join("src/support/workspace"),
        &["mod.rs", "registry.rs", "repo_root.rs"],
        "api workspace support tree must isolate repository-scoped asset resolution",
    );
}

fn assert_v1_tree(root: &std::path::Path) {
    assert_dir_entries(
        &root.join("src/v1"),
        &[
            "api/",
            "bam/",
            "bench/",
            "env/",
            "fastq/",
            "mod.rs",
            "pipelines/",
            "plan.rs",
            "report/",
            "run/",
            "shared.rs",
            "vcf.rs",
        ],
        "api v1 tree must stay curated",
    );
    assert_dir_entries(
        &root.join("src/v1/api"),
        &["front_door.rs", "mod.rs"],
        "api v1 front door must stay isolated in its own namespace",
    );
    assert_dir_entries(
        &root.join("src/v1/bam"),
        &["domain.rs", "feature_flags.rs", "mod.rs", "plan.rs", "stage_planning/"],
        "api v1 bam tree must separate planning internals from the public namespace",
    );
    assert_dir_entries(
        &root.join("src/v1/bam/stage_planning"),
        &[
            "alignment_qc.rs",
            "damage_recalibration.rs",
            "downstream.rs",
            "mod.rs",
            "stage_arguments.rs",
        ],
        "api v1 bam stage planning tree must stay explicit",
    );
    assert_dir_entries(
        &root.join("src/v1/bench"),
        &["exports.rs", "mod.rs"],
        "api v1 benchmark tree must stay isolated in its own namespace",
    );
    assert_dir_entries(
        &root.join("src/v1/env"),
        &["mod.rs", "runtime.rs"],
        "api v1 environment tree must stay isolated in its own namespace",
    );
    assert_dir_entries(
        &root.join("src/v1/fastq"),
        &["domain.rs", "mod.rs"],
        "api v1 fastq tree must stay isolated in its own namespace",
    );
    assert_dir_entries(
        &root.join("src/v1/pipelines"),
        &["mod.rs", "registry.rs"],
        "api v1 pipelines tree must stay isolated in its own namespace",
    );
    assert_dir_entries(
        &root.join("src/v1/run"),
        &[
            "entrypoints.rs",
            "mod.rs",
            "operator_failure.rs",
            "request_contracts.rs",
            "runtime_support.rs",
            "stage_assets.rs",
        ],
        "api v1 run tree must separate failure contracts from runtime entrypoints",
    );
    assert_dir_entries(
        &root.join("src/v1/report"),
        &["analysis_exports.rs", "html_bundle.rs", "mod.rs", "request_contracts.rs"],
        "api v1 report tree must separate html rendering from report entrypoints",
    );
}

fn assert_test_tree(root: &std::path::Path) {
    assert_dir_entries(
        &root.join("tests"),
        &[
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "guardrails.rs",
            "schemas/",
            "schemas.rs",
            "snapshots/",
            "support/",
        ],
        "api tests tree must stay grouped by enduring test intent",
    );
    assert_dir_entries(
        &root.join("tests/support"),
        &["workspace_paths.rs"],
        "api test support must keep shared helpers out of suite roots",
    );
    assert_contracts_tree(root);
}

fn assert_contracts_tree(root: &std::path::Path) {
    assert_dir_entries(
        &root.join("tests/boundaries"),
        &[
            "architecture.rs",
            "command_inventory.rs",
            "dependency_graph.rs",
            "docs_layout.rs",
            "guardrails/",
            "guardrails.rs",
            "v1_cross_guardrails.rs",
        ],
        "api boundary tests must cover architecture, docs, dependencies, and v1 guardrails",
    );
    assert_dir_entries(
        &root.join("tests/contracts"),
        &all_contract_entries(),
        "api contract tests must stay grouped by enduring public v1 behavior",
    );
    assert_bam_contracts_tree(root);
    assert_fastq_contracts_tree(root);
    assert_cross_contracts_tree(root);
    assert_dir_entries(
        &root.join("tests/schemas"),
        &[
            "v1_cross_api_stability.rs",
            "v1_cross_contract_handshake.rs",
            "v1_cross_docs_schema_snapshots.rs",
            "v1_cross_public_surface.rs",
            "v1_operator_failure_contract.rs",
            "v1_route_version_inventory.rs",
        ],
        "api schema tests must stay split by stable schema surface",
    );
}

fn assert_bam_contracts_tree(root: &std::path::Path) {
    assert_dir_contains_entries(
        &root.join("tests/contracts"),
        &bam_contract_entries(),
        "api BAM contract tests must stay split by public v1 behavior",
    );
}

fn assert_fastq_contracts_tree(root: &std::path::Path) {
    assert_dir_contains_entries(
        &root.join("tests/contracts"),
        &fastq_contract_entries(),
        "api FASTQ contract tests must stay split by public v1 behavior",
    );
}

fn assert_cross_contracts_tree(root: &std::path::Path) {
    assert_dir_contains_entries(
        &root.join("tests/contracts"),
        &cross_contract_entries(),
        "api cross-cutting contract tests must stay split by public v1 behavior",
    );
}

fn assert_dir_entries(path: &std::path::Path, expected: &[&str], message: &str) {
    let entries = dir_entries(path);
    let expected: BTreeSet<_> = expected.iter().copied().map(str::to_string).collect();
    assert_eq!(entries, expected, "{message}");
}

fn assert_dir_contains_entries(path: &std::path::Path, expected_subset: &[&str], message: &str) {
    let entries = dir_entries(path);
    let expected_subset: BTreeSet<_> =
        expected_subset.iter().copied().map(str::to_string).collect();
    let missing = expected_subset.difference(&entries).cloned().collect::<Vec<_>>();
    assert!(missing.is_empty(), "{message}: missing {missing:?}");
}

fn all_contract_entries() -> Vec<&'static str> {
    let mut entries = Vec::new();
    entries.extend_from_slice(bam_contract_entries());
    entries.extend_from_slice(fastq_contract_entries());
    entries.extend_from_slice(cross_contract_entries());
    entries
}

fn bam_contract_entries() -> &'static [&'static str] {
    &[
        "v1_bam_local_align_plan.rs",
        "v1_bam_local_authenticity_smoke.rs",
        "v1_bam_local_bias_mitigation_smoke.rs",
        "v1_bam_local_complexity_smoke.rs",
        "v1_bam_local_contamination_plan.rs",
        "v1_bam_local_contamination_smoke.rs",
        "v1_bam_local_coverage_smoke.rs",
        "v1_bam_local_damage_smoke.rs",
        "v1_bam_local_duplication_metrics_smoke.rs",
        "v1_bam_local_endogenous_content_smoke.rs",
        "v1_bam_local_filter_smoke.rs",
        "v1_bam_local_gc_bias_smoke.rs",
        "v1_bam_local_genotyping_plan.rs",
        "v1_bam_local_haplogroups_plan.rs",
        "v1_bam_local_haplogroups_smoke.rs",
        "v1_bam_local_insert_size_smoke.rs",
        "v1_bam_local_kinship_smoke.rs",
        "v1_bam_local_length_filter_smoke.rs",
        "v1_bam_local_mapping_summary_smoke.rs",
        "v1_bam_local_mapq_filter_smoke.rs",
        "v1_bam_local_markdup_smoke.rs",
        "v1_bam_local_overlap_correction_smoke.rs",
        "v1_bam_local_qc_pre_smoke.rs",
        "v1_bam_local_recalibration_smoke.rs",
        "v1_bam_local_sex_smoke.rs",
        "v1_bam_local_sex_tool_smoke.rs",
        "v1_bam_local_validate_smoke.rs",
    ]
}

fn fastq_contract_entries() -> &'static [&'static str] {
    &[
        "fastq_amplicon_governance_contract.rs",
        "v1_fastq_local_cluster_otus_smoke.rs",
        "v1_fastq_local_correct_errors_smoke.rs",
        "v1_fastq_local_deplete_host_plan.rs",
        "v1_fastq_local_deplete_reference_contaminants_plan.rs",
        "v1_fastq_local_deplete_rrna_plan.rs",
        "v1_fastq_local_detect_adapters_smoke.rs",
        "v1_fastq_local_detect_duplicates_premerge_smoke.rs",
        "v1_fastq_local_estimate_library_complexity_prealign_smoke.rs",
        "v1_fastq_local_extract_umis_smoke.rs",
        "v1_fastq_local_filter_low_complexity_smoke.rs",
        "v1_fastq_local_filter_reads_smoke.rs",
        "v1_fastq_local_index_reference_plan.rs",
        "v1_fastq_local_infer_asvs_smoke.rs",
        "v1_fastq_local_merge_pairs_smoke.rs",
        "v1_fastq_local_normalize_abundance_smoke.rs",
        "v1_fastq_local_normalize_primers_smoke.rs",
        "v1_fastq_local_profile_overrepresented_sequences_smoke.rs",
        "v1_fastq_local_profile_read_lengths_smoke.rs",
        "v1_fastq_local_profile_reads_smoke.rs",
        "v1_fastq_local_remove_chimeras_smoke.rs",
        "v1_fastq_local_remove_duplicates_smoke.rs",
        "v1_fastq_local_screen_taxonomy_plan.rs",
        "v1_fastq_local_trim_polyg_tails_smoke.rs",
        "v1_fastq_local_trim_reads_smoke.rs",
        "v1_fastq_local_trim_terminal_damage_smoke.rs",
        "v1_fastq_local_validate_reads_smoke.rs",
        "v1_fastq_small_integration.rs",
    ]
}

fn cross_contract_entries() -> &'static [&'static str] {
    &[
        "v1_cross_contract_spine.rs",
        "v1_cross_explain_roundtrip.rs",
        "v1_cross_profile_contracts.rs",
        "v1_cross_public_contract.rs",
        "v1_dry_run_manifest.rs",
        "v1_plan_manifest_contract.rs",
        "v1_report_evidence.rs",
        "v1_route_adapter_contract.rs",
        "v1_run_bundle_verifier.rs",
        "v1_run_cache_explain.rs",
        "v1_run_failure_injection.rs",
        "v1_run_replay_failed.rs",
        "v1_run_replay_success.rs",
        "v1_run_workflows.rs",
        "v1_status_evidence.rs",
    ]
}

fn dir_entries(path: &std::path::Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            name != ".DS_Store" && name != "out"
        })
        .map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect()
}

fn markdown_files_below(path: &std::path::Path) -> Vec<String> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension == "md"))
        .map(|entry| entry.path().display().to_string())
        .collect()
}

fn dependency_names(parsed: &toml::Value, table_name: &str) -> BTreeSet<String> {
    parsed
        .get(table_name)
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default()
}
