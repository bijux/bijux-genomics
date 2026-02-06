use cargo_metadata::MetadataCommand;

#[test]
fn runner_has_no_engine_dependency() {
    let metadata = MetadataCommand::new().exec().expect("load cargo metadata");
    let runner = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-runner")
        .expect("bijux-runner missing");
    let engine = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-engine")
        .expect("bijux-engine missing");
    let resolve = metadata.resolve.as_ref().expect("resolve graph missing");
    let node = resolve
        .nodes
        .iter()
        .find(|node| node.id == runner.id)
        .expect("runner node missing");
    let has_edge = node.deps.iter().any(|dep| dep.pkg == engine.id);
    assert!(!has_edge, "bijux-runner must not depend on bijux-engine");
}
