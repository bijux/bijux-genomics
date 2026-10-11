#![cfg(test)]
#[test]
fn missing_smoke_surface_marks_imputation_metrics_row_incomplete() {
    let inputs = crate::test_support::inputs();
    let declaration = inputs
        .matrix_rows()
        .find(|row| row.stage_id == "vcf.imputation_metrics")
        .expect("imputation metrics template");
    let schemas=bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors().expect("schemas");
    let schema = schemas.iter().find(|row| row.stage_id == declaration.stage_id).expect("schema");
    let retained = bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow {
        stage_id: declaration.stage_id.clone(),
        tool_id: declaration.tool_id.clone(),
        corpus_id: declaration.corpus_id.clone(),
        asset_profile_id: declaration.asset_profile_id.clone(),
        adapter_id: declaration.adapter_id.clone(),
        parser_id: declaration.parser_id.clone(),
        schema_id: schema.schema_id.clone(),
        tool_status: "supported".into(),
        stage_support_status: "supported".into(),
        scope_state: "active".into(),
        scope_detail: "governed declaration".into(),
        scope_proof_path: "scope.tsv".into(),
        reason: "fixture observation".into(),
    };
    let snapshot = bijux_dna_bench_vcf_evidence::snapshot::VcfEvidenceSnapshot::admit(
        bijux_dna_bench_vcf_evidence::snapshot::VcfEvidenceRows {
            retained: vec![retained],
            ..Default::default()
        },
    )
    .expect("snapshot");
    let binding =
        snapshot.stage_bindings("vcf.imputation_metrics").expect("stage").first().expect("binding");
    let commands = super::super::ReadinessCommandPaths {
        output_path: "commands.tsv".into(),
        argv_output_path: "commands.jsonl".into(),
    };
    let row = super::projection::build_vcf_imputation_metrics_ready_row(&commands, binding, None);
    assert_eq!(row.coverage_status, "incomplete");
    assert!(row
        .missing_surfaces
        .iter()
        .any(|surface| surface == "local_vcf_imputation_metrics_smoke"));
}
