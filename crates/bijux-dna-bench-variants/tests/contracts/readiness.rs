use crate::{
    inputs::VariantBenchmarkInputs,
    readiness::{self, ReadinessCommandPaths},
};
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::{
    model::scope::VcfActiveStageToolMatrixRow,
    snapshot::{VcfEvidenceRows, VcfEvidenceSnapshot},
};
use std::path::Path;
const STAGES: [&str; 4] = ["vcf.qc", "vcf.stats", "vcf.filter", "vcf.damage_filter"];
fn retained(stage: &str) -> VcfActiveStageToolMatrixRow {
    let inputs = inputs();
    let rows = inputs.governed().chain(inputs.qc()).cloned().collect::<Vec<_>>();
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
    inputs: &VariantBenchmarkInputs,
    snapshot: &VcfEvidenceSnapshot,
    commands: &ReadinessCommandPaths,
    root: &Path,
    output: &Path,
) -> Result<()> {
    match stage {
        "vcf.qc" => {
            readiness::qc::render_vcf_qc_ready(inputs, snapshot, commands, root, output).map(drop)
        }
        "vcf.stats" => {
            readiness::stats::render_vcf_stats_ready(inputs, snapshot, commands, root, output)
                .map(drop)
        }
        "vcf.filter" => {
            readiness::filter::render_vcf_filter_ready(inputs, snapshot, commands, root, output)
                .map(drop)
        }
        "vcf.damage_filter" => readiness::damage_filter::render_vcf_damage_filter_ready(
            inputs, snapshot, commands, root, output,
        )
        .map(drop),
        _ => anyhow::bail!("unknown stage"),
    }
}
#[test]
fn mismatched_snapshot_refuses_before_variant_or_publication_effects() -> Result<()> {
    let inputs = inputs();
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
fn missing_evidence_publishes_failed_readiness() -> Result<()> {
    let inputs = inputs();
    for stage in STAGES {
        let snapshot = snapshot(retained(stage))?;
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
        assert_eq!(row["command_ready"], false);
        assert_eq!(row["coverage_status"], "incomplete");
        assert!(row["missing_surfaces"]
            .as_array()
            .expect("missing surfaces")
            .iter()
            .any(|value| value == "vcf_rendered_commands"));
    }
    Ok(())
}

fn inputs() -> VariantBenchmarkInputs {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .parent()
        .expect("repository");
    let matrix: bijux_dna_bench_model::LocalVcfStageMatrixConfig = toml::from_str(
        &std::fs::read_to_string(root.join("configs/bench/local/vcf-stage-matrix.toml"))
            .expect("matrix"),
    )
    .expect("parse matrix");
    bijux_dna_bench_model::validate_local_vcf_stage_matrix_config(&matrix).expect("admit matrix");
    let governed = matrix
        .rows
        .into_iter()
        .filter(|row| {
            ["vcf.stats", "vcf.filter", "vcf.damage_filter"].contains(&row.stage_id.as_str())
        })
        .collect();
    VariantBenchmarkInputs::new(governed, crate::test_support::rows().1)
        .expect("admit variant declarations")
}
