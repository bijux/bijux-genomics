#![cfg(feature = "bam_downstream")]

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn stage_api_temp_repo() -> Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    let repo_root = repo_root();
    let tool_dir = temp.path().join("domain/bam/tools");
    fs::create_dir_all(&tool_dir)?;
    fs::copy(repo_root.join("domain/bam/tools/yleaf.yaml"), tool_dir.join("yleaf.yaml"))?;
    let runtime_dir = temp.path().join("configs/runtime/profiles");
    fs::create_dir_all(&runtime_dir)?;
    fs::copy(
        repo_root.join("configs/runtime/profiles/local.toml"),
        runtime_dir.join("local.toml"),
    )?;
    Ok(temp)
}

fn write_local_haplogroups_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-haplogroups.toml"), body)?;
    Ok(())
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

    assert_eq!(plan.params["reference_panel_id"], serde_json::json!("toy-human-y-hg38"));
    assert_eq!(
        plan.params["reference_panel"],
        serde_json::json!("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
    );
    assert_eq!(
        plan.params["reference_fasta"],
        serde_json::json!("assets/reference/host/references/toy_human_y_reference.fasta")
    );
    assert_eq!(plan.params["reference_build"], serde_json::json!("hg38"));
    assert_eq!(plan.params["population_scope"], serde_json::json!("human_y_haplogroup_panel"));
    assert_eq!(plan.params["coverage_gate"], serde_json::json!({ "min_coverage": 2.0 }));
    assert_eq!(plan.params["sample_id"], serde_json::json!("core-v1-haplogroups-y-panel-screen"));
    assert_eq!(plan.params["tool"], serde_json::json!("yleaf"));
    assert_eq!(plan.effective_params["min_coverage"], serde_json::json!(2.0));

    let command =
        plan.command.template.iter().last().unwrap_or_else(|| {
            panic!("bam.haplogroups command template must contain a shell body")
        });
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

#[test]
fn local_haplogroups_plan_rejects_empty_sample_ids() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_haplogroups_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_haplogroups.v1"
bam = "{bam}"
bai = "{bai}"
reference_fasta = "{reference}"
reference_panel_id = "toy-human-y-hg38"
reference_panel = "{panel}"
tool_id = "yleaf"
sample_id = " "
reference_build = "hg38"
population_scope = "human_y_haplogroup_panel"
min_coverage = 2.0
refuse_without_population_context = true
threads = 2
output_dir = "target/local-ready/bam.haplogroups"
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam").display(),
            bai = repo_root
                .join("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam.bai")
                .display(),
            reference = repo_root
                .join("assets/reference/host/references/toy_human_y_reference.fasta")
                .display(),
            panel = repo_root
                .join("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_haplogroups_plan(temp.path())
        .expect_err("empty sample_id must be rejected before haplogroups plan construction");
    assert_eq!(error.to_string(), "local-ready bam.haplogroups sample_id must not be empty");
    Ok(())
}

#[test]
fn local_haplogroups_plan_requires_reference_panel_ids() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_haplogroups_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_haplogroups.v1"
bam = "{bam}"
bai = "{bai}"
reference_fasta = "{reference}"
reference_panel_id = " "
reference_panel = "{panel}"
tool_id = "yleaf"
sample_id = "missing-panel-id"
reference_build = "hg38"
population_scope = "human_y_haplogroup_panel"
min_coverage = 2.0
refuse_without_population_context = true
threads = 2
output_dir = "target/local-ready/bam.haplogroups"
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam").display(),
            bai = repo_root
                .join("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam.bai")
                .display(),
            reference = repo_root
                .join("assets/reference/host/references/toy_human_y_reference.fasta")
                .display(),
            panel = repo_root
                .join("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_haplogroups_plan(temp.path())
        .expect_err("blank reference_panel_id must be rejected for governed haplogroups planning");
    assert_eq!(
        error.to_string(),
        "local-ready bam.haplogroups reference_panel_id must not be empty"
    );
    Ok(())
}

#[test]
fn local_haplogroups_plan_requires_reference_builds() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_haplogroups_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_haplogroups.v1"
bam = "{bam}"
bai = "{bai}"
reference_fasta = "{reference}"
reference_panel_id = "toy-human-y-hg38"
reference_panel = "{panel}"
tool_id = "yleaf"
sample_id = "missing-reference-build"
reference_build = " "
population_scope = "human_y_haplogroup_panel"
min_coverage = 2.0
refuse_without_population_context = true
threads = 2
output_dir = "target/local-ready/bam.haplogroups"
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam").display(),
            bai = repo_root
                .join("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam.bai")
                .display(),
            reference = repo_root
                .join("assets/reference/host/references/toy_human_y_reference.fasta")
                .display(),
            panel = repo_root
                .join("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_haplogroups_plan(temp.path())
        .expect_err("blank reference_build must be rejected for governed haplogroups planning");
    assert_eq!(error.to_string(), "local-ready bam.haplogroups reference_build must not be empty");
    Ok(())
}

#[test]
fn local_haplogroups_plan_requires_population_scopes() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_haplogroups_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_haplogroups.v1"
bam = "{bam}"
bai = "{bai}"
reference_fasta = "{reference}"
reference_panel_id = "toy-human-y-hg38"
reference_panel = "{panel}"
tool_id = "yleaf"
sample_id = "missing-population-scope"
reference_build = "hg38"
population_scope = " "
min_coverage = 2.0
refuse_without_population_context = true
threads = 2
output_dir = "target/local-ready/bam.haplogroups"
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam").display(),
            bai = repo_root
                .join("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam.bai")
                .display(),
            reference = repo_root
                .join("assets/reference/host/references/toy_human_y_reference.fasta")
                .display(),
            panel = repo_root
                .join("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_haplogroups_plan(temp.path())
        .expect_err("blank population_scope must be rejected for governed haplogroups planning");
    assert_eq!(error.to_string(), "local-ready bam.haplogroups population_scope must not be empty");
    Ok(())
}

#[test]
fn local_haplogroups_plan_requires_positive_minimum_coverage() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_haplogroups_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_haplogroups.v1"
bam = "{bam}"
bai = "{bai}"
reference_fasta = "{reference}"
reference_panel_id = "toy-human-y-hg38"
reference_panel = "{panel}"
tool_id = "yleaf"
sample_id = "non-positive-coverage-gate"
reference_build = "hg38"
population_scope = "human_y_haplogroup_panel"
min_coverage = 0.0
refuse_without_population_context = true
threads = 2
output_dir = "target/local-ready/bam.haplogroups"
"#,
            bam = repo_root.join("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam").display(),
            bai = repo_root
                .join("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam.bai")
                .display(),
            reference = repo_root
                .join("assets/reference/host/references/toy_human_y_reference.fasta")
                .display(),
            panel = repo_root
                .join("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_haplogroups_plan(temp.path())
        .expect_err("non-positive min_coverage must be rejected for governed haplogroups planning");
    assert_eq!(
        error.to_string(),
        "local-ready bam.haplogroups min_coverage must be finite and greater than zero"
    );
    Ok(())
}
