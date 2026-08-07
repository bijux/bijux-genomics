use std::collections::BTreeSet;

#[test]
fn crate_root_and_docs_match_architecture_contract() {
    let root = crate_root("bijux-dna-containers");

    assert_eq!(
        dir_entries(&root),
        btree_set(&["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("docs")),
        btree_set(&[
            "ARCHITECTURE.md",
            "BOUNDARY.md",
            "COMMANDS.md",
            "CONTRACTS.md",
            "DEPENDENCIES.md",
            "EFFECTS.md",
            "ENV_REFERENCE.md",
            "INDEX.md",
            "PUBLIC_API.md",
            "TESTS.md",
        ]),
        "docs must stay within the 10-document allowance"
    );
}

#[test]
fn source_tree_matches_architecture_contract() {
    let root = crate_root("bijux-dna-containers");

    assert_eq!(
        dir_entries(&root.join("src")),
        btree_set(&[
            "build/",
            "evidence/",
            "execution_contract/",
            "lib.rs",
            "public_api/",
            "resolve/",
            "runtime_spec/",
            "site_policy/",
        ]),
        "src tree must match the documented environment layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/evidence/tool_contract")),
        btree_set(&[
            "catalog.rs",
            "catalog_lock.rs",
            "catalog_validation.rs",
            "inspection.rs",
            "labels.rs",
            "mod.rs",
            "model.rs",
            "preparation.rs",
            "recipe_identity.rs",
            "registry_projection.rs",
            "schema.rs",
            "smoke.rs",
            "validation.rs",
            "version_catalog.rs",
            "version_lock.rs",
        ]),
        "tool contracts must separate source, rendering, inspection, and admission concerns"
    );

    assert_eq!(
        dir_entries(&root.join("src/runtime_spec")),
        btree_set(&["compatibility.rs", "mod.rs", "model.rs"]),
        "runtime_spec tree must stay decomposed by runtime concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/build")),
        btree_set(&[
            "apptainer.rs",
            "builder.rs",
            "campaign.rs",
            "defaults.rs",
            "entrypoints.rs",
            "mod.rs",
            "models.rs",
            "publication.rs",
            "stable_surface.rs",
            "version_parser.rs",
        ]),
        "build tree must remain decomposed by concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolve")),
        btree_set(&[
            "cache/",
            "catalog/",
            "commands.rs",
            "entrypoints.rs",
            "facade.rs",
            "mod.rs",
            "platform.rs",
            "reference/",
            "shell.rs",
            "smoke.rs",
            "stable_surface.rs",
            "types/",
        ]),
        "resolve tree must remain decomposed by environment concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolve/cache")),
        btree_set(&["image_paths.rs", "mod.rs", "root.rs"]),
        "resolve cache tree must stay split by cache concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolve/catalog")),
        btree_set(
            &["catalog_loader.rs", "image_resolution.rs", "mod.rs", "registry_hydration.rs",]
        ),
        "resolve catalog tree must stay split by catalog concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolve/reference")),
        btree_set(&["digest.rs", "index_preparation.rs", "mod.rs"]),
        "resolve reference tree must stay split by reference concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolve/types")),
        btree_set(&["errors.rs", "image.rs", "mod.rs", "platform.rs", "runtime.rs"]),
        "resolve types tree must stay split by model concern"
    );
}

#[test]
fn execution_policy_tree_matches_architecture_contract() {
    let root = crate_root("bijux-dna-containers");
    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        btree_set(&["mod.rs", "stable_surface.rs"]),
        "public api tree must keep the stable surface explicit"
    );
    assert_eq!(
        dir_entries(&root.join("src/execution_contract")),
        btree_set(&["credential.rs", "mod.rs", "pipeline/"]),
        "execution contracts must remain strict typed admission surfaces"
    );
    assert_eq!(
        dir_entries(&root.join("src/execution_contract/pipeline")),
        btree_set(&[
            "admission.rs",
            "currentization.rs",
            "mod.rs",
            "model.rs",
            "schema.rs",
            "validation.rs",
        ]),
        "pipeline contracts must separate models, schemas, validation, site admission, and currentization"
    );
    assert_eq!(
        dir_entries(&root.join("src/site_policy")),
        btree_set(&["mod.rs", "model.rs", "repository_freeze.rs", "schema.rs", "validation.rs"]),
        "site policy must separate declarations, schemas, validation, and repository effects"
    );
}

#[test]
fn inventory_source_tree_matches_ownership_contract() {
    let root = crate_root("bijux-dna-containers");
    assert_eq!(
        dir_entries(&root.join("src/evidence")),
        btree_set(&[
            "chain.rs",
            "external_artifact/",
            "image_activation/",
            "image_storage/",
            "inventory/",
            "load.rs",
            "mod.rs",
            "model.rs",
            "qualification_registry/",
            "run_evidence/",
            "runtime_verification/",
            "schema.rs",
            "selection.rs",
            "tool_contract/",
            "validation.rs",
        ]),
        "evidence tree must separate models, admission, validation, and schemas"
    );
    assert_eq!(
        dir_entries(&root.join("src/evidence/image_activation")),
        btree_set(&["mod.rs", "model.rs", "schema.rs", "workflow.rs"]),
        "image activation must separate persisted models, lifecycle, and schemas"
    );
    assert_eq!(
        dir_entries(&root.join("src/evidence/qualification_registry")),
        btree_set(&["mod.rs", "model.rs", "schema.rs", "workflow.rs"]),
        "qualification registry must separate persisted models, derivation, and schemas"
    );
    assert_eq!(
        dir_entries(&root.join("src/evidence/image_storage")),
        btree_set(&["mod.rs", "model.rs", "report.rs", "schema.rs", "workflow.rs"]),
        "image storage must separate persisted models, lifecycle, report, and schemas"
    );
    assert_eq!(
        dir_entries(&root.join("src/evidence/inventory")),
        btree_set(&["derivation.rs", "mod.rs", "report.rs", "source.rs"]),
        "inventory must separate execution-site inputs, admission decisions, and reporting"
    );
}

#[test]
fn test_tree_matches_architecture_contract() {
    let root = crate_root("bijux-dna-containers");

    assert_eq!(
        dir_entries(&root.join("tests")),
        btree_set(&[
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "determinism.rs",
            "fixtures/",
            "guardrails.rs",
            "public_api_docs.rs",
            "schemas/",
            "schemas.rs",
        ]),
        "test tree must stay grouped by taxonomy without nested README docs"
    );
}

fn crate_root(crate_name: &str) -> std::path::PathBuf {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let actual = root.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    assert_eq!(actual, crate_name, "unexpected integration-test crate root");
    root
}

fn dir_entries(path: &std::path::Path) -> BTreeSet<String> {
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

fn btree_set(entries: &[&str]) -> BTreeSet<String> {
    entries.iter().map(|entry| (*entry).to_string()).collect()
}
