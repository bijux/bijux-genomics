use cargo_metadata::MetadataCommand;

#[test]
fn engine_has_no_runner_dependency() {
    let metadata = MetadataCommand::default()
        .exec()
        .unwrap_or_else(|err| panic!("load cargo metadata: {err}"));
    let engine = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-engine")
        .unwrap_or_else(|| panic!("bijux-engine missing"));
    let runner = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-runner")
        .unwrap_or_else(|| panic!("bijux-runner missing"));
    let resolve = metadata
        .resolve
        .as_ref()
        .unwrap_or_else(|| panic!("resolve graph missing"));
    let node = resolve
        .nodes
        .iter()
        .find(|node| node.id == engine.id)
        .unwrap_or_else(|| panic!("engine node missing"));
    let has_edge = node.deps.iter().any(|dep| dep.pkg == runner.id);
    assert!(!has_edge, "bijux-engine must not depend on bijux-runner");
}
