use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn core_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_top_level_layout(root);
    assert_docs_layout(root);
    assert_contract_layout(root);
    assert_identity_layout(root);
    assert_api_layout(root);
    assert_catalog_layout(root);
    assert_test_layout(root);
}

fn assert_top_level_layout(root: &Path) {
    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "crate root must stay minimal and intentional"
    );
    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "contract/",
            "foundation/",
            "id_catalog/",
            "ids/",
            "lib.rs",
            "metrics/",
            "prelude/",
            "public_api/",
        ]),
        "src tree must match the documented core layout"
    );
}

fn assert_docs_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("docs")),
        entries([
            "ARCHITECTURE.md",
            "BOUNDARY.md",
            "CHANGE_RULES.md",
            "COMMANDS.md",
            "CONTRACTS.md",
            "CONTRACT_MAP.md",
            "INVARIANTS.md",
            "PUBLIC_API.md",
            "SERIALIZATION.md",
            "TESTS.md",
        ]),
        "core docs must stay at the 10-document allowance and live under docs/"
    );

    let misplaced_docs = markdown_files_outside_docs(root);
    assert!(
        misplaced_docs.is_empty(),
        "crate markdown outside docs/ must be limited to root README.md: {misplaced_docs:?}",
    );
}

fn assert_contract_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("src/contract/execution")),
        entries([
            "OWNER.toml",
            "contract.rs",
            "graph.rs",
            "io.rs",
            "manifest.rs",
            "mod.rs",
            "policy.rs",
            "record.rs",
        ]),
        "execution contracts must stay partitioned by execution concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract/run")),
        entries([
            "OWNER.toml",
            "domain.rs",
            "index.rs",
            "metadata.rs",
            "mod.rs",
            "provenance.rs",
            "spec.rs",
        ]),
        "run contracts must stay partitioned by run concern"
    );
    assert_eq!(
        dir_entries(&root.join("src/contract/tooling/selection")),
        entries(["mod.rs"]),
        "tooling selection tree must stay focused on selection policy"
    );
}

fn assert_identity_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("src/ids")),
        entries(["OWNER.toml", "domain_model.rs", "mod.rs", "parsing/", "typed/"]),
        "ids tree must keep typed ids, parsing, and semantic models separated"
    );
    assert_eq!(
        dir_entries(&root.join("src/ids/parsing")),
        entries(["OWNER.toml", "mod.rs", "pipeline.rs", "stage.rs", "symbolic.rs", "tool.rs"]),
        "parsing tree must stay partitioned by identifier family"
    );

    assert_eq!(
        dir_entries(&root.join("src/ids/typed")),
        entries([
            "OWNER.toml",
            "artifact.rs",
            "mod.rs",
            "pipeline.rs",
            "run.rs",
            "stage.rs",
            "tool.rs",
        ]),
        "typed id tree must stay partitioned by identifier family"
    );
}

fn assert_api_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("src/prelude")),
        entries([
            "OWNER.toml",
            "catalog_surface.rs",
            "contract_surface.rs",
            "foundation_surface.rs",
            "identity_surface.rs",
            "metric_surface.rs",
            "mod.rs",
            "stable_surface.rs",
        ]),
        "prelude tree must stay grouped by source area"
    );
    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries([
            "OWNER.toml",
            "catalog/",
            "contracts/",
            "ergonomics/",
            "identity/",
            "metrics/",
            "mod.rs",
        ]),
        "public api tree must stay curated"
    );
    assert_eq!(
        dir_entries(&root.join("src/public_api/contracts")),
        entries(["mod.rs"]),
        "public api contracts tree must stay focused"
    );
    assert_eq!(
        dir_entries(&root.join("src/public_api/catalog")),
        entries(["mod.rs"]),
        "public api catalog tree must stay focused"
    );
    assert_eq!(
        dir_entries(&root.join("src/public_api/identity")),
        entries(["mod.rs"]),
        "public api identity tree must stay focused"
    );
    assert_eq!(
        dir_entries(&root.join("src/public_api/metrics")),
        entries(["mod.rs"]),
        "public api metrics tree must stay focused"
    );
    assert_eq!(
        dir_entries(&root.join("src/public_api/ergonomics")),
        entries(["mod.rs"]),
        "public api ergonomics tree must stay focused"
    );
}

fn assert_catalog_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("src/foundation/command")),
        entries(["command_spec.rs", "container_image_ref.rs", "mod.rs"]),
        "foundation command tree must separate command templates from container image contracts"
    );
    assert_eq!(
        dir_entries(&root.join("src/id_catalog")),
        entries(["OWNER.toml", "mod.rs", "pipeline/", "stage/", "tool/"]),
        "identifier catalog must stay partitioned by catalog concern"
    );
    assert_eq!(
        dir_entries(&root.join("src/id_catalog/pipeline")),
        entries([
            "OWNER.toml",
            "bam.rs",
            "bam_to_vcf.rs",
            "fastq.rs",
            "fastq_to_bam.rs",
            "fastq_to_vcf.rs",
            "mod.rs",
            "vcf.rs",
        ]),
        "pipeline catalog must stay partitioned by graph concern"
    );
    assert_eq!(
        dir_entries(&root.join("src/id_catalog/stage")),
        entries([
            "OWNER.toml",
            "bam.rs",
            "core.rs",
            "fastq.rs",
            "mod.rs",
            "prefixes.rs",
            "report.rs",
            "vcf.rs",
        ]),
        "stage catalog must stay partitioned by domain and shared concern"
    );
    assert_eq!(
        dir_entries(&root.join("src/id_catalog/tool")),
        entries(["OWNER.toml", "bam.rs", "fastq.rs", "mod.rs", "shared.rs", "vcf.rs"]),
        "tool catalog must stay partitioned by workflow concern"
    );
}

fn assert_test_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("tests")),
        entries([
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
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

    assert_eq!(
        dir_entries(&root.join("tests/boundaries")),
        entries([
            "architecture_tree.rs",
            "command_inventory.rs",
            "core_scope_guardrail.rs",
            "dependency_graph.rs",
            "docs_layout.rs",
            "execution_graph_purity.rs",
            "guardrails.rs",
            "layering.rs",
        ]),
        "boundary tests must stay partitioned by architecture concern"
    );

    assert_eq!(
        dir_entries(&root.join("tests/contracts")),
        entries(["execution/", "identity/", "surface/"]),
        "contract tests must stay partitioned by execution, identity, and surface concern"
    );

    assert_eq!(
        dir_entries(&root.join("tests/contracts/execution")),
        entries([
            "execution_contract_validation_contracts.rs",
            "execution_graph_validate.rs",
            "execution_plan_contract.rs",
            "planning_manifest_contract.rs",
            "schema_compatibility_contracts.rs",
        ]),
        "execution contract tests must stay focused on graph and output contracts"
    );

    assert_eq!(
        dir_entries(&root.join("tests/contracts/identity")),
        entries([
            "hashing_identity.rs",
            "prelude_snapshot.rs",
            "reproducibility_identity.rs",
            "run_index.rs",
            "run_metadata.rs",
        ]),
        "identity contract tests must stay focused on stable identity surfaces"
    );

    assert_eq!(
        dir_entries(&root.join("tests/contracts/surface")),
        entries([
            "canonicalization.rs",
            "contract_surface_semantics_contracts.rs",
            "metrics_ids_selection_contracts.rs",
            "sanity.rs",
        ]),
        "surface contract tests must stay focused on public contract behavior"
    );

    assert_eq!(
        dir_entries(&root.join("tests/schemas")),
        entries([
            "docs_public_api.rs",
            "public_api_lock.rs",
            "public_module_tree.rs",
            "public_surface.rs",
            "public_surface_lock.rs",
        ]),
        "schema tests must stay focused on docs and public-surface locks"
    );

    assert_eq!(
        dir_entries(&root.join("tests/semantics")),
        entries(["ids/", "ids.rs", "input_assessment.rs", "metrics/", "metrics.rs"]),
        "semantic tests must stay partitioned by core model concern"
    );

    assert_eq!(
        dir_entries(&root.join("tests/semantics/ids")),
        entries(["catalogs.rs", "conversions.rs", "smoke.rs"]),
        "identifier semantic tests must stay partitioned by catalog and conversion concern"
    );

    assert_eq!(
        dir_entries(&root.join("tests/semantics/metrics")),
        entries(["registry.rs"]),
        "metric semantic tests must stay focused on registry behavior"
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
