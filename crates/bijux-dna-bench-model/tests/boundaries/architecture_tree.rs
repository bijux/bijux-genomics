use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn bench_model_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_docs_tree(root);
    assert_commands_doc(root);
    assert_dependency_graph(root);
    assert_source_owners(root);
    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "compare/",
            "contract/",
            "diagnostics/",
            "lib.rs",
            "model/",
            "policy/",
            "public_api/",
            "stats/",
        ]),
        "src tree must match the documented benchmark model layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/compare")),
        entries(["diff.rs", "mod.rs", "report.rs", "stable_surface.rs", "stratify.rs"]),
        "compare tree must separate diff execution from report contracts"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract")),
        entries(["mod.rs", "records.rs", "schema_versions.rs", "suite/"]),
        "contract tree must separate record validators, schema ids, and suite rules"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract/suite")),
        entries([
            "analysis.rs",
            "diversity.rs",
            "edge_ports.rs",
            "governance.rs",
            "graph.rs",
            "mod.rs",
            "param_bindings.rs",
            "validation/",
        ]),
        "suite contract tree must stay partitioned by enduring concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract/suite/validation")),
        entries([
            "declared_stage_nodes.rs",
            "edge_contracts.rs",
            "mod.rs",
            "stage_contracts.rs",
            "suite_validation.rs",
        ]),
        "suite validation tree must separate orchestration, shared node contracts, and rule families"
    );

    assert_eq!(
        dir_entries(&root.join("src/policy")),
        entries(["gate_policy/", "mod.rs", "outcomes.rs"]),
        "policy tree must separate evaluation from policy outcomes"
    );

    assert_eq!(
        dir_entries(&root.join("src/diagnostics")),
        entries(["error_taxonomy.rs", "mod.rs"]),
        "diagnostics tree must stay focused on stable error contracts"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["mod.rs", "stable_surface.rs"]),
        "public api tree must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("src/model/suite/support")),
        entries([
            "analysis_requirements.rs",
            "dataset_spec.rs",
            "diversity_requirements.rs",
            "mod.rs",
            "replicate_policy.rs",
            "stratification_requirement.rs",
        ]),
        "suite support tree must separate durable contract families"
    );

    assert_eq!(
        dir_entries(&root.join("src/stats/robust_estimators")),
        entries(["contracts.rs", "mod.rs"]),
        "robust estimators must separate typed stats contracts from estimator functions"
    );

    assert_eq!(
        dir_entries(&root.join("tests")),
        entries([
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "determinism.rs",
            "fixtures/",
            "guardrails.rs",
            "schemas/",
            "schemas.rs",
            "semantics/",
            "semantics.rs",
            "snapshots/",
        ]),
        "test tree must stay organized by enduring intent"
    );
}

fn assert_commands_doc(root: &Path) {
    let commands_path = root.join("docs").join("COMMANDS.md");
    let commands_doc = std::fs::read_to_string(&commands_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_path.display()));
    let required_operations = [
        "`validate-suite`",
        "`validate-observation`",
        "`validate-summary`",
        "`validate-decision`",
        "`compare-summaries`",
        "`gate-policy-decide`",
        "`robust-stats`",
        "`bootstrap-ci`",
        "`mad-outliers`",
    ];
    for operation in required_operations {
        assert!(
            commands_doc.contains(operation),
            "COMMANDS.md must list managed operation {operation}",
        );
    }
    for entrypoint in [
        "contract::validate_suite",
        "contract::validate_observation",
        "contract::validate_summary",
        "contract::validate_decision",
        "compare::compare_summaries",
        "GatePolicy::decide",
        "stats::robust_stats",
        "stats::bootstrap_ci",
        "stats::mad_outliers",
    ] {
        assert!(
            commands_doc.contains(entrypoint),
            "COMMANDS.md must link operation to Rust entrypoint {entrypoint}",
        );
    }
}

fn assert_source_owners(root: &Path) {
    let source_root = root.join("src");
    let mut files = Vec::new();
    collect_rs_files(&source_root, &mut files);
    let mut missing_owner = Vec::new();
    for file in files {
        let content = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("read {}: {err}", file.display()));
        let header = content.lines().take(12).collect::<Vec<_>>().join("\n");
        if !header.contains("Owner: bijux-dna-bench-model") {
            missing_owner.push(file.display().to_string());
        }
        assert!(
            !header.contains("Owner: bijux-dna-bench\n"),
            "{} must use the bench-model owner, not the bench crate owner",
            file.display(),
        );
    }
    assert!(
        missing_owner.is_empty(),
        "source files must declare Owner: bijux-dna-bench-model in the first 12 lines: {missing_owner:?}",
    );
}

fn assert_dependency_graph(root: &Path) {
    let cargo_toml_path = root.join("Cargo.toml");
    let cargo_toml = std::fs::read_to_string(&cargo_toml_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", cargo_toml_path.display()));
    let dependencies = manifest_dependencies(&cargo_toml, "[dependencies]");
    let dev_dependencies = manifest_dependencies(&cargo_toml, "[dev-dependencies]");

    let duplicate_edges: Vec<_> =
        dependencies.intersection(&dev_dependencies).map(String::as_str).collect();
    assert!(
        duplicate_edges.is_empty(),
        "dependencies must not be duplicated in dev-dependencies: {duplicate_edges:?}",
    );

    let forbidden = ["bijux-dna-api", "bijux-dna-bench", "bijux-dna-runner", "bijux-dna-runtime"];
    for name in forbidden {
        assert!(
            !dependencies.contains(name) && !dev_dependencies.contains(name),
            "bench-model must not depend on downstream orchestration crate {name}",
        );
    }
}

fn assert_docs_tree(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("docs")),
        entries([
            "ARCHITECTURE.md",
            "BOUNDARY.md",
            "CHANGE_RULES.md",
            "COMMANDS.md",
            "DECISION_EXPLAINABILITY.md",
            "DETERMINISM.md",
            "GATE_POLICY.md",
            "PUBLIC_API.md",
            "STATISTICS.md",
            "TESTS.md",
        ]),
        "bench model docs must stay at the 10-document allowance and live under docs/"
    );

    let misplaced_docs = markdown_files_outside_docs(root);
    assert!(
        misplaced_docs.is_empty(),
        "crate markdown outside docs/ must be limited to root README.md: {misplaced_docs:?}",
    );
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
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

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}

fn manifest_dependencies(manifest: &str, section: &str) -> BTreeSet<String> {
    let mut in_section = false;
    let mut deps = BTreeSet::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
            continue;
        }
        if !in_section || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };
        deps.insert(name.trim().to_string());
    }
    deps
}

fn markdown_files_outside_docs(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_markdown_files(root, root, &mut files);
    files
}

fn collect_markdown_files(root: &Path, path: &Path, files: &mut Vec<String>) {
    let entries =
        std::fs::read_dir(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(root, &path, files);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "md")
            && path != root.join("README.md")
            && !path.starts_with(root.join("docs"))
        {
            files.push(path.display().to_string());
        }
    }
}

fn collect_rs_files(path: &Path, files: &mut Vec<std::path::PathBuf>) {
    let entries =
        std::fs::read_dir(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}
