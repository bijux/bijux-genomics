//! Public benchmark aliases preserve canonical stage wire identity.
#[test]
fn canonical_stage_aliases_preserve_wire_identity() -> anyhow::Result<()> {
    let readiness = bijux_dna_bench_model::BenchmarkLocalStageReadinessKind::Smoke;
    let canonical: bijux_dna_bench_stage::BenchmarkLocalStageReadinessKind = readiness;
    assert_eq!(serde_json::to_string(&canonical)?, "\"smoke\"");
    assert_eq!(
        serde_json::to_value(bijux_dna_bench_model::benchmark_local_stage_commands_schema())?,
        serde_json::to_value(bijux_dna_bench_stage::benchmark_local_stage_commands_schema())?
    );
    Ok(())
}
