use bijux_pipelines::registry::PipelineRegistry;

#[test]
fn pipeline_registry_order_is_stable() {
    let registry = PipelineRegistry::v1();
    let ids: Vec<String> = registry
        .list(true)
        .into_iter()
        .map(|p| p.id.to_string())
        .collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "pipeline registry must be sorted by id");
}
