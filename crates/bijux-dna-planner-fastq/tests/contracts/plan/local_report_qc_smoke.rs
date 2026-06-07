use anyhow::Result;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_report_qc_smoke_plan_uses_governed_qc_fixture_bundle() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_fastq::stage_api::local_report_qc_smoke_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "fastq.report_qc");
    assert_eq!(plan.tool_id.as_str(), "multiqc");
    assert_eq!(plan.resources.threads, 2);
    assert_eq!(plan.out_dir, PathBuf::from("runs/bench/local-smoke/fastq.report_qc"));
    assert_eq!(plan.io.inputs.len(), 3);

    let detect_adapters = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "fastq.detect_adapters.tool.fastqc.report_json")
        .unwrap_or_else(|| panic!("detect_adapters governed QC input missing"));
    assert_eq!(
        detect_adapters.path,
        repo_root
            .join("assets/toy/core-v1/fastq/report_qc/contributors/detect_adapters.report.json")
    );

    let manifest_output = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "governed_qc_inputs_manifest")
        .unwrap_or_else(|| panic!("governed_qc_inputs_manifest output missing"));
    assert_eq!(
        manifest_output.path,
        PathBuf::from("runs/bench/local-smoke/fastq.report_qc/governed_qc_inputs_manifest.json")
    );

    let report_output = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "report_json")
        .unwrap_or_else(|| panic!("report_json output missing"));
    assert_eq!(
        report_output.path,
        PathBuf::from("runs/bench/local-smoke/fastq.report_qc/report_qc_report.json")
    );

    assert_eq!(plan.effective_params["aggregation_engine"], serde_json::json!("multiqc"));
    assert_eq!(
        plan.effective_params["aggregation_scope"],
        serde_json::json!("governed_qc_artifacts")
    );
    assert!(
        plan.command.template[2].contains(
            "multiqc -o 'runs/bench/local-smoke/fastq.report_qc/multiqc_data' -n multiqc_report.html"
        ),
        "local report_qc smoke plan must target the governed output directory"
    );

    Ok(())
}

#[test]
fn local_report_qc_smoke_plan_stage_api_surface_stays_callable() {
    let _: fn(&Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_report_qc_smoke_plan;
}
