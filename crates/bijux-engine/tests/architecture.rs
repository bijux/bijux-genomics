use cargo_metadata::MetadataCommand;

#[test]
fn engine_has_no_runner_dependency() {
    let metadata = MetadataCommand::new().exec().expect("load cargo metadata");
    let engine = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-engine")
        .expect("bijux-engine missing");
    let runner = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-runner")
        .expect("bijux-runner missing");
    let resolve = metadata.resolve.as_ref().expect("resolve graph missing");
    let node = resolve
        .nodes
        .iter()
        .find(|node| node.id == engine.id)
        .expect("engine node missing");
    let has_edge = node.deps.iter().any(|dep| dep.pkg == runner.id);
    assert!(!has_edge, "bijux-engine must not depend on bijux-runner");
}
