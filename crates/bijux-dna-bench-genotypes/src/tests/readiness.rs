#![cfg(test)]
use crate::{
    inputs::GenotypeBenchmarkInputs,
    readiness::{self, ReadinessCommandPaths},
};
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::{
    model::scope::VcfActiveStageToolMatrixRow,
    snapshot::{VcfEvidenceRows, VcfEvidenceSnapshot},
};
use std::path::Path;
const STAGES: [&str; 5] =
    ["vcf.call", "vcf.call_gl", "vcf.call_diploid", "vcf.call_pseudohaploid", "vcf.gl_propagation"];
fn retained(stage: &str) -> VcfActiveStageToolMatrixRow {
    let rows = crate::test_support::rows();
    let row = rows.iter().find(|row| row.stage_id == stage).expect("stage declaration");
    let descriptors=bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors().expect("schemas");
    let schema = descriptors.iter().find(|row| row.stage_id == stage).expect("stage schema");
    VcfActiveStageToolMatrixRow {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
        adapter_id: row.adapter_id.clone(),
        parser_id: row.parser_id.clone(),
        schema_id: schema.schema_id.clone(),
        tool_status: "production".to_owned(),
        stage_support_status: "supported".to_owned(),
        scope_state: "active".to_owned(),
        scope_detail: "governed declaration".to_owned(),
        scope_proof_path: "scope.tsv".to_owned(),
        reason: "fixture observation".to_owned(),
    }
}
fn snapshot(row: VcfActiveStageToolMatrixRow) -> Result<VcfEvidenceSnapshot> {
    VcfEvidenceSnapshot::admit(VcfEvidenceRows {
        retained: vec![row],
        ..VcfEvidenceRows::default()
    })
}
fn command_paths() -> ReadinessCommandPaths {
    ReadinessCommandPaths {
        output_path: "commands.tsv".to_owned(),
        argv_output_path: "commands.json".to_owned(),
    }
}
fn render(
    stage: &str,
    inputs: &GenotypeBenchmarkInputs,
    snapshot: &VcfEvidenceSnapshot,
    commands: &ReadinessCommandPaths,
    root: &Path,
    output: &Path,
) -> Result<()> {
    match stage {
        "vcf.call" => {
            readiness::call::render_vcf_call_ready(inputs, snapshot, commands, root, output)
                .map(drop)
        }
        "vcf.call_gl" => {
            readiness::call_gl::render_vcf_call_gl_ready(inputs, snapshot, commands, root, output)
                .map(drop)
        }
        "vcf.call_diploid" => readiness::call_diploid::render_vcf_call_diploid_ready(
            inputs, snapshot, commands, root, output,
        )
        .map(drop),
        "vcf.call_pseudohaploid" => {
            readiness::call_pseudohaploid::render_vcf_call_pseudohaploid_ready(
                inputs, snapshot, commands, root, output,
            )
            .map(drop)
        }
        "vcf.gl_propagation" => readiness::gl_propagation::render_vcf_gl_propagation_ready(
            inputs, snapshot, commands, root, output,
        )
        .map(drop),
        _ => anyhow::bail!("unknown stage"),
    }
}
#[test]
fn mismatched_snapshot_refuses_before_genotype_or_publication_effects() -> Result<()> {
    let inputs = crate::test_support::inputs();
    for stage in STAGES {
        for field in ["corpus", "asset", "schema"] {
            let mut row = retained(stage);
            match field {
                "corpus" => row.corpus_id = "foreign".to_owned(),
                "asset" => row.asset_profile_id = "foreign".to_owned(),
                _ => row.schema_id = "foreign".to_owned(),
            }
            let snapshot = snapshot(row)?;
            let root = tempfile::tempdir()?;
            let output = root.path().join("readiness.json");
            std::fs::write(&output, b"previous readiness")?;
            let error = render(stage, &inputs, &snapshot, &command_paths(), root.path(), &output)
                .err()
                .map(|error| error.to_string());
            assert!(
                error.is_some_and(|message| message.contains("snapshot disagrees")),
                "{stage}/{field}"
            );
            assert_eq!(std::fs::read(&output)?, b"previous readiness");
            assert_eq!(std::fs::read_dir(root.path())?.count(), 1);
        }
    }
    Ok(())
}
#[test]
fn blank_command_publication_identity_refuses_before_effects() -> Result<()> {
    let inputs = crate::test_support::inputs();
    for stage in STAGES {
        let snapshot = snapshot(retained(stage))?;
        let root = tempfile::tempdir()?;
        let output = root.path().join("readiness.json");
        let mut commands = command_paths();
        commands.argv_output_path.clear();
        let error = render(stage, &inputs, &snapshot, &commands, root.path(), &output)
            .err()
            .map(|error| error.to_string());
        assert!(error.is_some_and(|message| message.contains("publication paths")));
        assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    }
    Ok(())
}
#[test]
fn unavailable_smoke_publishes_failed_readiness_with_missing_surface() -> Result<()> {
    let inputs = crate::test_support::inputs();
    for stage in STAGES {
        let mut row = retained(stage);
        row.tool_id = "unregistered".to_owned();
        let snapshot = snapshot(row)?;
        let root = tempfile::tempdir()?;
        let output = root.path().join("readiness.json");
        assert!(
            render(stage, &inputs, &snapshot, &command_paths(), root.path(), &output).is_err(),
            "{stage}"
        );
        let report: serde_json::Value = serde_json::from_slice(&std::fs::read(output)?)?;
        assert_eq!(report["ok"], false);
        assert_eq!(report["incomplete_row_count"], 1);
        assert_eq!(report["violation_count"], 1);
        let row = &report["rows"][0];
        assert_eq!(row["smoke_ready"], false);
        assert_eq!(row["coverage_status"], "incomplete");
        let expected =
            format!("local_vcf_{}_smoke", stage.strip_prefix("vcf.").expect("VCF stage"));
        assert!(
            row["missing_surfaces"]
                .as_array()
                .expect("missing surfaces")
                .iter()
                .any(|value| value == &expected),
            "{stage}"
        );
    }
    Ok(())
}
