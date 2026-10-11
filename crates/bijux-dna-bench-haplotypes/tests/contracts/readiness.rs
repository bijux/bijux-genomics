use crate::{
    inputs::HaplotypeBenchmarkInputs,
    readiness::{self, ReadinessCommandPaths},
};
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::{
    model::scope::VcfActiveStageToolMatrixRow,
    snapshot::{VcfEvidenceRows, VcfEvidenceSnapshot},
};
use std::path::Path;
const STAGES: [&str; 2] = ["vcf.prepare_reference_panel", "vcf.imputation_metrics"];
fn retained(stage: &str) -> VcfActiveStageToolMatrixRow {
    let inputs = inputs();
    let rows = inputs.matrix_rows().cloned().collect::<Vec<_>>();
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
    inputs: &HaplotypeBenchmarkInputs,
    snapshot: &VcfEvidenceSnapshot,
    commands: &ReadinessCommandPaths,
    root: &Path,
    output: &Path,
) -> Result<()> {
    match stage {
        "vcf.prepare_reference_panel" => {
            readiness::reference_panel::render_vcf_prepare_reference_panel_ready(
                inputs, snapshot, commands, root, output,
            )
            .map(drop)
        }
        "vcf.imputation_metrics" => {
            readiness::imputation_metrics::render_vcf_imputation_metrics_ready(
                inputs, snapshot, commands, root, output,
            )
            .map(drop)
        }
        _ => anyhow::bail!("unknown stage"),
    }
}
#[test]
fn mismatched_snapshot_refuses_before_haplotype_or_publication_effects() -> Result<()> {
    let inputs = inputs();
    for stage in STAGES {
        for field in ["corpus", "asset", "adapter", "parser", "schema"] {
            let mut row = retained(stage);
            match field {
                "corpus" => row.corpus_id = "foreign".to_owned(),
                "asset" => row.asset_profile_id = "foreign".to_owned(),
                "adapter" => row.adapter_id = "foreign".to_owned(),
                "parser" => row.parser_id = "foreign".to_owned(),
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
    let inputs = inputs();
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
fn missing_evidence_preserves_each_stages_publication_contract() -> Result<()> {
    let inputs = inputs();
    for stage in STAGES {
        let snapshot = snapshot(retained(stage))?;
        let root = tempfile::tempdir()?;
        let output = root.path().join("readiness.json");
        std::fs::write(&output, b"previous readiness")?;
        assert!(render(stage, &inputs, &snapshot, &command_paths(), root.path(), &output).is_err());
        if stage == "vcf.prepare_reference_panel" {
            assert_eq!(std::fs::read(&output)?, b"previous readiness");
        } else {
            let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&output)?)?;
            assert_eq!(report["ok"], false);
            assert_eq!(report["incomplete_row_count"], 1);
            assert_eq!(report["rows"][0]["coverage_status"], "incomplete");
        }
    }
    Ok(())
}
#[test]
fn unregistered_backend_refuses_before_effects() -> Result<()> {
    let inputs = inputs();
    for stage in STAGES {
        let mut row = retained(stage);
        row.tool_id = "unregistered".to_owned();
        let snapshot = snapshot(row)?;
        let root = tempfile::tempdir()?;
        let error = render(
            stage,
            &inputs,
            &snapshot,
            &command_paths(),
            root.path(),
            Path::new("readiness.json"),
        )
        .expect_err("unsupported tool");
        assert!(error.to_string().contains("no canonical binding"));
        assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    }
    Ok(())
}

fn inputs() -> HaplotypeBenchmarkInputs {
    crate::test_support::inputs()
}
