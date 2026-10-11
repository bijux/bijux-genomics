use crate::readiness::{pca::render_vcf_pca_ready, ReadinessCommandPaths};
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::{
    model::{
        commands::{VcfRenderedCommandRow, VcfRenderedCommandStep},
        pca::{PcaAdapterEvidence, PcaAdapterOutputProof},
        scope::VcfActiveStageToolMatrixRow,
    },
    snapshot::{VcfEvidenceRows, VcfEvidenceSnapshot, VcfStageReadinessBindingKey},
};
fn inputs() -> crate::inputs::PopulationBenchmarkInputs {
    crate::test_support::inputs()
}
fn retained(tool: &str) -> VcfActiveStageToolMatrixRow {
    let inputs = inputs();
    let rows = inputs.supported_rows().to_vec();
    let row = rows
        .iter()
        .find(|row| row.stage_id == "vcf.pca" && row.tool_id == tool)
        .expect("stage declaration");
    let descriptors=bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors().expect("schemas");
    let schema = descriptors.iter().find(|row| row.stage_id == "vcf.pca").expect("stage schema");
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
fn observation(tool: &str) -> PcaAdapterEvidence {
    let row = retained(tool);
    let argv = vec![tool.to_owned(), "--pca".to_owned()];
    let command = bijux_dna_bench_vcf_evidence::commands::projection::render_shell_command(&argv);
    PcaAdapterEvidence {
        binding: VcfStageReadinessBindingKey {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id,
            asset_profile_id: row.asset_profile_id,
        },
        adapter_id: row.adapter_id,
        parser_id: row.parser_id,
        schema_id: row.schema_id,
        output: PcaAdapterOutputProof {
            output_proof_path: "adapter.json".to_owned(),
            benchmark_status: "not_benchmark_ready".to_owned(),
            raw_outputs: Vec::new(),
            normalized_metrics_outputs: Vec::new(),
            manifest_output: "stage-result.json".to_owned(),
            index_outputs: Vec::new(),
        },
        command: VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_owned(),
            benchmark_status: "benchmark_ready".to_owned(),
            command_source: "admitted-test-observation".to_owned(),
            command_steps: vec![VcfRenderedCommandStep {
                step_id: "pca".to_owned(),
                step_kind: "process".to_owned(),
                consumes_previous_stdout: false,
                argv,
                command: command.clone(),
            }],
            script_commands: vec![command],
            reason: "test observation".to_owned(),
        },
    }
}
fn snapshot(commands: bool) -> Result<VcfEvidenceSnapshot> {
    VcfEvidenceSnapshot::admit(VcfEvidenceRows {
        retained: vec![retained("plink2"), retained("eigensoft")],
        commands: if commands {
            vec![observation("plink2").command, observation("eigensoft").command]
        } else {
            Vec::new()
        },
        ..VcfEvidenceRows::default()
    })
}
fn paths() -> ReadinessCommandPaths {
    ReadinessCommandPaths {
        output_path: "pca.sh".to_owned(),
        argv_output_path: "pca.argv.jsonl".to_owned(),
    }
}
#[test]
fn mismatched_missing_or_repeated_pca_adapter_evidence_refuses_before_effects() -> Result<()> {
    let inputs = inputs();
    let snapshot = snapshot(true)?;
    for fault in 0..8 {
        let mut evidence = vec![observation("plink2"), observation("eigensoft")];
        match fault {
            0 => evidence[0].binding.corpus_id = "foreign".to_owned(),
            1 => evidence[0].parser_id = "foreign".to_owned(),
            2 => evidence[0].schema_id = "foreign".to_owned(),
            3 => evidence[0].command.command_steps[0].argv.push("--foreign".to_owned()),
            4 => evidence[0].output.output_proof_path = " padded".to_owned(),
            5 => evidence.push(evidence[0].clone()),
            6 => {
                evidence.pop();
            }
            _ => evidence[0].binding.stage_id = "vcf.qc".to_owned(),
        }
        let root = tempfile::tempdir()?;
        let output = root.path().join("readiness.json");
        std::fs::write(&output, b"previous readiness")?;
        let error =
            render_vcf_pca_ready(&inputs, &snapshot, &evidence, &paths(), root.path(), &output)
                .err()
                .map(|error| error.to_string());
        assert!(error.is_some_and(|message| message.contains("PCA")), "fault {fault}");
        assert_eq!(std::fs::read(&output)?, b"previous readiness");
        assert_eq!(std::fs::read_dir(root.path())?.count(), 1);
    }
    Ok(())
}
#[test]
fn absent_pca_command_and_scientific_observations_publish_incomplete_report() -> Result<()> {
    let inputs = inputs();
    let snapshot = snapshot(false)?;
    let evidence = vec![observation("plink2"), observation("eigensoft")];
    let root = tempfile::tempdir()?;
    let output = root.path().join("readiness.json");
    assert!(render_vcf_pca_ready(&inputs, &snapshot, &evidence, &paths(), root.path(), &output)
        .is_err());
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(output)?)?;
    assert_eq!(report["ok"], false);
    assert_eq!(report["retained_row_count"], 2);
    assert_eq!(report["incomplete_row_count"], 2);
    for row in report["rows"].as_array().expect("rows") {
        assert_eq!(row["command_ready"], false);
        assert_eq!(row["smoke_ready"], false);
        assert_eq!(row["coverage_status"], "incomplete");
        let missing = row["missing_surfaces"].as_array().expect("missing surfaces");
        assert!(missing.iter().any(|value| value == "vcf_rendered_commands"));
        assert!(missing.iter().any(|value| value == "local_vcf_pca_smoke"));
    }
    Ok(())
}
