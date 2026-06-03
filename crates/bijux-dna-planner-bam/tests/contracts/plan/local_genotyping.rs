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
    fs::copy(
        repo_root.join("domain/bam/tools/angsd.yaml"),
        tool_dir.join("angsd.yaml"),
    )?;
    let runtime_dir = temp.path().join("configs/runtime/profiles");
    fs::create_dir_all(&runtime_dir)?;
    fs::copy(
        repo_root.join("configs/runtime/profiles/local.toml"),
        runtime_dir.join("local.toml"),
    )?;
    Ok(temp)
}

fn write_local_genotyping_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-genotyping.toml"), body)?;
    Ok(())
}

#[test]
fn local_genotyping_plan_uses_governed_bam_reference_and_sites_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_bam::stage_api::local_genotyping_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "bam.genotyping");
    assert_eq!(plan.tool_id.as_str(), "angsd");
    assert_eq!(plan.resources.threads, 2);
    assert_eq!(plan.resources.mem_gb, 8);
    assert_eq!(plan.out_dir, PathBuf::from("target/local-ready/bam.genotyping"));

    let bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "bam")
        .unwrap_or_else(|| panic!("bam input missing from local-ready genotyping plan"));
    assert_eq!(bam.path, PathBuf::from("assets/toy/core-v1/bam/genotyping_panel_sites.sam"));

    let bai = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "bam_bai")
        .unwrap_or_else(|| panic!("bam_bai input missing from local-ready genotyping plan"));
    assert_eq!(
        bai.path,
        PathBuf::from("assets/toy/core-v1/bam/genotyping_panel_sites.sam.bai")
    );

    let reference = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference")
        .unwrap_or_else(|| panic!("reference input missing from local-ready genotyping plan"));
    assert_eq!(
        reference.path,
        PathBuf::from("assets/toy/core-v1/bam/genotyping_reference_chr1.fasta")
    );

    let sites = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "sites")
        .unwrap_or_else(|| panic!("sites input missing from local-ready genotyping plan"));
    assert_eq!(sites.path, PathBuf::from("assets/toy/core-v1/vcf/genotyping_candidate_sites.vcf"));

    let regions = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "regions")
        .unwrap_or_else(|| panic!("regions input missing from local-ready genotyping plan"));
    assert_eq!(regions.path, PathBuf::from("assets/toy/core-v1/bam/genotyping_target_regions.txt"));

    let bcf = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "genotyping_bcf")
        .unwrap_or_else(|| panic!("genotyping_bcf output missing from local-ready plan"));
    assert_eq!(bcf.path, PathBuf::from("target/local-ready/bam.genotyping/genotyping.bcf"));

    let vcf = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "genotyping_vcf")
        .unwrap_or_else(|| panic!("genotyping_vcf output missing from local-ready plan"));
    assert_eq!(vcf.path, PathBuf::from("target/local-ready/bam.genotyping/genotyping.vcf.gz"));

    assert_eq!(
        plan.params["reference"],
        serde_json::json!("assets/toy/core-v1/bam/genotyping_reference_chr1.fasta")
    );
    assert_eq!(
        plan.params["sites"],
        serde_json::json!("assets/toy/core-v1/vcf/genotyping_candidate_sites.vcf")
    );
    assert_eq!(
        plan.params["regions"],
        serde_json::json!("assets/toy/core-v1/bam/genotyping_target_regions.txt")
    );
    assert_eq!(
        plan.params["producer_contract"]["bcf"],
        serde_json::json!("target/local-ready/bam.genotyping/genotyping.bcf")
    );
    assert_eq!(
        plan.params["producer_contract"]["vcf"],
        serde_json::json!("target/local-ready/bam.genotyping/genotyping.vcf.gz")
    );
    assert_eq!(
        plan.params["sample_id"],
        serde_json::json!("core-v1-genotyping-panel-sites")
    );
    assert_eq!(plan.params["tool"], serde_json::json!("angsd"));
    assert_eq!(plan.effective_params["caller"], serde_json::json!("angsd"));
    assert_eq!(plan.effective_params["min_posterior"], serde_json::json!(0.9));
    assert_eq!(plan.effective_params["min_call_rate"], serde_json::json!(0.5));

    let command = plan
        .command
        .template
        .iter()
        .last()
        .unwrap_or_else(|| panic!("bam.genotyping command template must contain a shell body"));
    assert!(
        command.contains("assets/toy/core-v1/bam/genotyping_panel_sites.sam.bai")
            && command.contains("assets/toy/core-v1/bam/genotyping_reference_chr1.fasta")
            && command.contains("assets/toy/core-v1/vcf/genotyping_candidate_sites.vcf")
            && command.contains("assets/toy/core-v1/bam/genotyping_target_regions.txt")
            && command.contains("target/local-ready/bam.genotyping/genotyping.bcf")
            && command.contains("target/local-ready/bam.genotyping/genotyping.vcf.gz"),
        "local-ready genotyping command must carry the governed BAI, reference, sites, regions, BCF, and VCF outputs"
    );

    Ok(())
}

#[test]
fn local_genotyping_plan_stage_api_surface_stays_callable() {
    let _: fn(&Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_bam::stage_api::local_genotyping_plan;
}

#[test]
fn local_genotyping_plan_rejects_empty_sample_ids() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_genotyping_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_genotyping.v1"
bam = "{bam}"
bai = "{bai}"
reference_fasta = "{reference}"
sites_vcf = "{sites}"
regions = "{regions}"
tool_id = "angsd"
sample_id = " "
min_posterior = 0.9
min_call_rate = 0.5
threads = 2
output_dir = "target/local-ready/bam.genotyping"
"#,
            bam = repo_root
                .join("assets/toy/core-v1/bam/genotyping_panel_sites.sam")
                .display(),
            bai = repo_root
                .join("assets/toy/core-v1/bam/genotyping_panel_sites.sam.bai")
                .display(),
            reference = repo_root
                .join("assets/toy/core-v1/bam/genotyping_reference_chr1.fasta")
                .display(),
            sites = repo_root
                .join("assets/toy/core-v1/vcf/genotyping_candidate_sites.vcf")
                .display(),
            regions = repo_root
                .join("assets/toy/core-v1/bam/genotyping_target_regions.txt")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_genotyping_plan(temp.path())
        .expect_err("empty sample_id must be rejected before genotyping plan construction");
    assert_eq!(error.to_string(), "local-ready bam.genotyping sample_id must not be empty");
    Ok(())
}
