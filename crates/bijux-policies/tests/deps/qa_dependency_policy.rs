use cargo_metadata::MetadataCommand;

#[test]
fn production_crates_do_not_depend_on_qa() {
    let metadata = MetadataCommand::new().exec().expect("cargo metadata");
    let qa = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-environment-qa")
        .expect("bijux-environment-qa missing");
    let mut offenders = Vec::new();
    let resolve = metadata.resolve.as_ref().expect("resolve graph missing");
    for node in &resolve.nodes {
        if node.id == qa.id {
            continue;
        }
        if node.deps.iter().any(|dep| dep.pkg == qa.id) {
            let pkg = metadata
                .packages
                .iter()
                .find(|pkg| pkg.id == node.id)
                .expect("package missing");
            offenders.push(pkg.name.clone());
        }
    }
    assert!(
        offenders.is_empty(),
        "production crates must not depend on bijux-environment-qa.\n\
Keep QA isolated and invoked manually.\n\
See STYLE.md.\n\
Offenders: {offenders:?}"
    );
}
