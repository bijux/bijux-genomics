use cargo_metadata::MetadataCommand;

#[test]
fn runner_has_no_engine_dependency() {
    let metadata = MetadataCommand::default()
        .exec()
        .unwrap_or_else(|err| panic!("load cargo metadata: {err}"));
    let runner = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-runner")
        .unwrap_or_else(|| panic!("bijux-runner missing"));
    let engine = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-engine")
        .unwrap_or_else(|| panic!("bijux-engine missing"));
    let resolve = metadata
        .resolve
        .as_ref()
        .unwrap_or_else(|| panic!("resolve graph missing"));
    let node = resolve
        .nodes
        .iter()
        .find(|node| node.id == runner.id)
        .unwrap_or_else(|| panic!("runner node missing"));
    let has_edge = node.deps.iter().any(|dep| dep.pkg == engine.id);
    assert!(!has_edge, "bijux-runner must not depend on bijux-engine");
}
