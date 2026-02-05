use bijux_core::metrics_registry::{metrics_schema_for_stage, FASTQ_METRICS_SCHEMAS};

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
