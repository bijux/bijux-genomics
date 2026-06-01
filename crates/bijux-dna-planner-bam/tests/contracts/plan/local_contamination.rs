use anyhow::Result;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_contamination_plan_uses_governed_bam_reference_and_panel_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_bam::stage_api::local_contamination_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "bam.contamination");
    assert_eq!(plan.tool_id.as_str(), "verifybamid2");
    assert_eq!(plan.resources.threads, 2);
    assert_eq!(plan.resources.mem_gb, 8);
    assert_eq!(plan.out_dir, PathBuf::from("target/local-ready/bam.contamination"));

    let bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "bam")
        .unwrap_or_else(|| panic!("bam input missing from local-ready plan"));
    assert_eq!(bam.path, PathBuf::from("assets/toy/core-v1/bam/contamination_panel_screen.sam"));

    let bai = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "bam_bai")
        .unwrap_or_else(|| panic!("bam_bai input missing from local-ready plan"));
    assert_eq!(
        bai.path,
        PathBuf::from("assets/toy/core-v1/bam/contamination_panel_screen.sam.bai")
    );

    let reference = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference")
        .unwrap_or_else(|| panic!("reference input missing from local-ready plan"));
    assert_eq!(
        reference.path,
        PathBuf::from("assets/reference/host/references/toy_host_reference.fasta")
    );

    let reference_panel = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference_panel")
        .unwrap_or_else(|| panic!("reference_panel input missing from local-ready plan"));
    assert_eq!(
        reference_panel.path,
        PathBuf::from("assets/reference/host/references/toy_human_contamination_panel.dat")
    );

    let contamination_report = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "contamination_report")
        .unwrap_or_else(|| panic!("contamination_report output missing from local-ready plan"));
    assert_eq!(
        contamination_report.path,
        PathBuf::from("target/local-ready/bam.contamination/contamination.json")
    );

    assert_eq!(plan.params["scope"], serde_json::json!("nuclear"));
    assert_eq!(
        plan.params["assumptions"],
        serde_json::json!(
            "toy host reference with governed population-af panel for local contamination planning"
        )
    );
    assert_eq!(
        plan.params["reference_panels"],
        serde_json::json!(["assets/reference/host/references/toy_human_contamination_panel.dat"])
    );
    assert_eq!(plan.params["sample_id"], serde_json::json!("core-v1-contamination-panel-screen"));
    assert_eq!(plan.params["tool"], serde_json::json!("verifybamid2"));
    assert_eq!(plan.effective_params["chromosome_system"], serde_json::json!("xy"));
    assert_eq!(plan.effective_params["minimum_mean_coverage"], serde_json::json!(0.5));

    let command =
        plan.command.template.iter().last().unwrap_or_else(|| {
            panic!("bam.contamination command template must contain a shell body")
        });
    assert!(
        command.contains("assets/toy/core-v1/bam/contamination_panel_screen.sam.bai")
            && command.contains("assets/reference/host/references/toy_host_reference.fasta")
            && command.contains("assets/reference/host/references/toy_human_contamination_panel.dat")
            && command.contains("target/local-ready/bam.contamination/contamination"),
        "local-ready contamination command must carry the governed BAI, reference, panel, and output prefix"
    );

    Ok(())
}

#[test]
fn local_contamination_plan_stage_api_surface_stays_callable() {
    let _: fn(&std::path::Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_bam::stage_api::local_contamination_plan;
}
