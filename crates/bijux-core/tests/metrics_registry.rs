use bijux_core::metrics_registry::{
    metric_semantics, metrics_schema_for_stage, FASTQ_METRICS_SCHEMAS,
};

#[test]
fn metrics_registry_resolves_all_fastq_schemas() {
    for schema in FASTQ_METRICS_SCHEMAS {
        let Some(resolved) = metrics_schema_for_stage(schema.stage_id) else {
            panic!("metrics schema lookup should resolve");
        };
        assert_eq!(resolved.schema, schema.schema);
        assert_eq!(resolved.version, schema.version);
    }
}

#[test]
fn metrics_registry_exposes_compare_semantics() {
    let metric_ids = [
        "runtime_s",
        "memory_mb",
        "read_retention",
        "base_retention",
        "merge_rate",
        "error_reduction_proxy",
    ];
    for metric_id in metric_ids {
        assert!(
            metric_semantics(metric_id).is_some(),
            "missing semantics for {metric_id}"
        );
    }
}
