#![allow(clippy::too_many_lines)]

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
fn local_deplete_reference_contaminants_plan_uses_governed_repo_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan =
        bijux_dna_planner_fastq::stage_api::local_deplete_reference_contaminants_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "fastq.deplete_reference_contaminants");
    assert_eq!(plan.tool_id.as_str(), "bowtie2");
    assert_eq!(plan.resources.threads, 4);
    assert_eq!(plan.resources.mem_gb, 8);
    assert_eq!(
        plan.out_dir,
        PathBuf::from("benchmarks/readiness/local-ready/fastq.deplete_reference_contaminants")
    );

    let input_r1 = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reads_r1")
        .unwrap_or_else(|| panic!("reads_r1 input missing from local-ready plan"));
    assert_eq!(input_r1.path, PathBuf::from("assets/toy/core-v1/fastq/reads_1.fastq"));

    let reference_index = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference_index")
        .unwrap_or_else(|| panic!("reference_index input missing from local-ready plan"));
    assert_eq!(
        reference_index.path,
        PathBuf::from("assets/reference/contaminants/references/toy_contaminant_reference")
    );

    let retained_reads = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "contaminant_screened_reads_r1")
        .unwrap_or_else(|| {
            panic!("contaminant_screened_reads_r1 output missing from local-ready plan")
        });
    assert_eq!(
        retained_reads.path,
        PathBuf::from(
            "benchmarks/readiness/local-ready/fastq.deplete_reference_contaminants/contaminant_screened.fastq.gz"
        )
    );

    let removed_reads = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "removed_contaminant_reads_r1")
        .unwrap_or_else(|| {
            panic!("removed_contaminant_reads_r1 output missing from local-ready plan")
        });
    assert_eq!(
        removed_reads.path,
        PathBuf::from(
            "benchmarks/readiness/local-ready/fastq.deplete_reference_contaminants/removed_contaminant.fastq.gz"
        )
    );

    let report_json = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "contaminant_screen_report_json")
        .unwrap_or_else(|| {
            panic!("contaminant_screen_report_json output missing from local-ready plan")
        });
    assert_eq!(
        report_json.path,
        PathBuf::from(
            "benchmarks/readiness/local-ready/fastq.deplete_reference_contaminants/contaminant_screen_report.json"
        )
    );

    assert_eq!(
        plan.params["reference_index"],
        serde_json::json!("assets/reference/contaminants/references/toy_contaminant_reference")
    );
    assert_eq!(plan.params["tool"], serde_json::json!("bowtie2"));
    assert_eq!(plan.params["threads"], serde_json::json!(4));
    assert_eq!(plan.params["decoy_mode"], serde_json::json!("phix_and_spikeins"));
    assert_eq!(
        plan.params["removed_reads_r1"],
        serde_json::json!(
            "benchmarks/readiness/local-ready/fastq.deplete_reference_contaminants/removed_contaminant.fastq.gz"
        )
    );
    assert_eq!(
        plan.effective_params["reference_catalog_id"],
        serde_json::json!("contaminant_reference")
    );
    assert_eq!(
        plan.effective_params["reference_index_backend"],
        serde_json::json!("bowtie2_build")
    );
    assert!(
        plan.command.template.iter().any(|part| {
            part == "assets/reference/contaminants/references/toy_contaminant_reference"
        }) && plan.command.template.iter().any(|part| {
            part
                == "benchmarks/readiness/local-ready/fastq.deplete_reference_contaminants/bowtie2.contaminant.metrics.txt"
        }) && plan.command.template.iter().any(|part| {
            part
                == "benchmarks/readiness/local-ready/fastq.deplete_reference_contaminants/removed_contaminant.fastq.gz"
        }),
        "local-ready plan command must materialize the governed contaminant Bowtie2 index and metrics path"
    );
    Ok(())
}

#[test]
fn local_deplete_reference_contaminants_plan_stage_api_surface_stays_callable() {
    let _: fn(&std::path::Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_deplete_reference_contaminants_plan;
}
