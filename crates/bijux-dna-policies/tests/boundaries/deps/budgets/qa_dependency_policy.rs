#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeSet;

use cargo_metadata::MetadataCommand;

#[test]
fn policy__boundaries__qa_dependency_policy__qa_dependencies_stay_on_execution_surfaces() {
    let metadata = MetadataCommand::new().exec().expect("cargo metadata");
    let qa = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-dna-containers-qa")
        .expect("bijux-dna-containers-qa missing");
    let mut consumers = BTreeSet::new();
    let resolve = metadata.resolve.as_ref().expect("resolve graph missing");
    for node in &resolve.nodes {
        if node.id == qa.id {
            continue;
        }
        if node.deps.iter().any(|dep| dep.pkg == qa.id) {
            let pkg =
                metadata.packages.iter().find(|pkg| pkg.id == node.id).expect("package missing");
            consumers.insert(pkg.name.clone());
        }
    }
    let expected = BTreeSet::from(["bijux-dna-api".to_string(), "bijux-dna-runner".to_string()]);
    bijux_dna_policies::policy_assert_eq!(
        consumers,
        expected,
        "bijux-dna-containers-qa must be consumed only by the stable API composition and the one-shot execution owner declared in TARGET_ARCHITECTURE.md and docs/10-architecture/BOUNDARY_MAP.md"
    );
}
