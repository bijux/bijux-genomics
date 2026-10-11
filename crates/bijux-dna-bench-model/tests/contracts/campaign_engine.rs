#[test]
fn campaign_engine_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        (
            "case-matrix-v1.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_case_matrix_v1_schema())?,
        ),
        (
            "case-matrix.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_case_matrix_schema())?,
        ),
        (
            "stage-normalization.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_stage_normalization_schema())?,
        ),
        (
            "admitted-case-observation.schema.json",
            serde_json::to_value(
                bijux_dna_bench_model::benchmark_admitted_case_observation_schema(),
            )?,
        ),
        (
            "admitted-case-observation-v2.schema.json",
            serde_json::to_value(
                bijux_dna_bench_model::benchmark_admitted_case_observation_v2_schema(),
            )?,
        ),
        (
            "admitted-case-observation-set.schema.json",
            serde_json::to_value(
                bijux_dna_bench_model::benchmark_admitted_case_observation_set_schema(),
            )?,
        ),
        (
            "case-aggregation.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_case_aggregation_schema())?,
        ),
        (
            "case-aggregation-v2.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_case_aggregation_v2_schema())?,
        ),
        (
            "case-aggregation-set.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_case_aggregation_set_schema())?,
        ),
    ];
    let schema_root = repository_root.join("benchmarks/contracts/schemas");
    for (name, generated) in schemas {
        let path = schema_root.join(name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert_eq!(checked, generated, "campaign engine schema drifted: {name}");
    }
    Ok(())
}

#[test]
fn campaign_engine_schema_versions_are_exact() -> anyhow::Result<()> {
    for (schema, expected) in [
        (
            bijux_dna_bench_model::benchmark_case_matrix_schema(),
            bijux_dna_bench_model::BENCHMARK_CASE_MATRIX_SCHEMA_V2,
        ),
        (
            bijux_dna_bench_model::benchmark_stage_normalization_schema(),
            bijux_dna_bench_model::BENCHMARK_STAGE_NORMALIZATION_SCHEMA_V1,
        ),
        (
            bijux_dna_bench_model::benchmark_admitted_case_observation_schema(),
            bijux_dna_bench_model::BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1,
        ),
        (
            bijux_dna_bench_model::benchmark_admitted_case_observation_v2_schema(),
            bijux_dna_bench_model::BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2,
        ),
        (
            bijux_dna_bench_model::benchmark_admitted_case_observation_set_schema(),
            bijux_dna_bench_model::BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1,
        ),
        (
            bijux_dna_bench_model::benchmark_case_aggregation_schema(),
            bijux_dna_bench_model::BENCHMARK_CASE_AGGREGATION_SCHEMA_V1,
        ),
        (
            bijux_dna_bench_model::benchmark_case_aggregation_v2_schema(),
            bijux_dna_bench_model::BENCHMARK_CASE_AGGREGATION_SCHEMA_V2,
        ),
        (
            bijux_dna_bench_model::benchmark_case_aggregation_set_schema(),
            bijux_dna_bench_model::BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1,
        ),
    ] {
        let value = serde_json::to_value(schema)?;
        assert_eq!(value["properties"]["schema_version"]["const"], expected);
    }
    Ok(())
}
