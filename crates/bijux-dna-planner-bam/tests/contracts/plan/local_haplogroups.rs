#![cfg(feature = "bam_downstream")]

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
fn local_haplogroups_plan_uses_governed_bam_reference_and_panel_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_bam::stage_api::local_haplogroups_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "bam.haplogroups");
    assert_eq!(plan.tool_id.as_str(), "yleaf");
    assert_eq!(plan.resources.threads, 2);
    assert_eq!(plan.resources.mem_gb, 8);
    assert_eq!(plan.out_dir, PathBuf::from("target/local-ready/bam.haplogroups"));

    let bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "bam")
        .unwrap_or_else(|| panic!("bam input missing from local-ready haplogroups plan"));
    assert_eq!(bam.path, PathBuf::from("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam"));

    let bai = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "bam_bai")
        .unwrap_or_else(|| panic!("bam_bai input missing from local-ready haplogroups plan"));
    assert_eq!(
        bai.path,
        PathBuf::from("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam.bai")
    );

    let reference = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference")
        .unwrap_or_else(|| panic!("reference input missing from local-ready haplogroups plan"));
    assert_eq!(
        reference.path,
        PathBuf::from("assets/reference/host/references/toy_human_y_reference.fasta")
    );

    let reference_panel = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference_panel")
        .unwrap_or_else(|| {
            panic!("reference_panel input missing from local-ready haplogroups plan")
        });
    assert_eq!(
        reference_panel.path,
        PathBuf::from("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
    );

    let haplogroups_report = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "haplogroups")
        .unwrap_or_else(|| panic!("haplogroups output missing from local-ready plan"));
    assert_eq!(
        haplogroups_report.path,
        PathBuf::from("target/local-ready/bam.haplogroups/haplogroups.json")
    );

    assert_eq!(
        plan.params["reference_panel_id"],
        serde_json::json!("toy-human-y-hg38")
    );
    assert_eq!(
        plan.params["reference_panel"],
        serde_json::json!("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
    );
    assert_eq!(
        plan.params["reference_fasta"],
        serde_json::json!("assets/reference/host/references/toy_human_y_reference.fasta")
    );
    assert_eq!(plan.params["reference_build"], serde_json::json!("hg38"));
    assert_eq!(
        plan.params["population_scope"],
        serde_json::json!("human_y_haplogroup_panel")
    );
    assert_eq!(
        plan.params["coverage_gate"],
        serde_json::json!({ "min_coverage": 2.0 })
    );
    assert_eq!(
        plan.params["sample_id"],
        serde_json::json!("core-v1-haplogroups-y-panel-screen")
    );
    assert_eq!(plan.params["tool"], serde_json::json!("yleaf"));
    assert_eq!(plan.effective_params["min_coverage"], serde_json::json!(2.0));

    let command = plan
        .command
        .template
        .iter()
        .last()
        .unwrap_or_else(|| panic!("bam.haplogroups command template must contain a shell body"));
    assert!(
        command.contains("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam")
            && command.contains("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam.bai")
            && command.contains("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
            && command.contains("target/local-ready/bam.haplogroups/haplogroups")
            && command.contains("--reference_genome hg38"),
        "local-ready haplogroups command must carry the governed BAM, BAI, panel, output prefix, and reference build"
    );

    Ok(())
}

#[test]
fn local_haplogroups_plan_stage_api_surface_stays_callable() {
    let _: fn(&Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_bam::stage_api::local_haplogroups_plan;
}
