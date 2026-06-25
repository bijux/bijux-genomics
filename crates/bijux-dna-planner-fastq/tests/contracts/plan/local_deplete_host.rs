use anyhow::Result;
use std::path::{Path, PathBuf};

use bijux_dna_stage_contract::StagePlanV1;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn input_path<'a>(plan: &'a StagePlanV1, name: &str) -> &'a Path {
    plan.io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .unwrap_or_else(|| panic!("{name} input missing from local-ready plan"))
        .path
        .as_path()
}

fn output_path<'a>(plan: &'a StagePlanV1, name: &str) -> &'a Path {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .unwrap_or_else(|| panic!("{name} output missing from local-ready plan"))
        .path
        .as_path()
}

#[test]
fn local_deplete_host_plan_uses_governed_repo_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_fastq::stage_api::local_deplete_host_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "fastq.deplete_host");
    assert_eq!(plan.tool_id.as_str(), "bowtie2");
    assert_eq!(plan.resources.threads, 4);
    assert_eq!(plan.resources.mem_gb, 8);
    assert_eq!(plan.out_dir, PathBuf::from("benchmarks/readiness/local-ready/fastq.deplete_host"));
    for (name, expected) in [
        ("reads_r1", "assets/toy/core-v1/fastq/reads_1.fastq"),
        ("reads_r2", "assets/toy/core-v1/fastq/reads_2.fastq"),
        ("reference_index", "assets/reference/host/references/toy_host_reference"),
    ] {
        assert_eq!(input_path(&plan, name), Path::new(expected));
    }
    for (name, expected) in [
        (
            "host_depleted_reads_r1",
            "benchmarks/readiness/local-ready/fastq.deplete_host/host_depleted_R1.fastq.gz",
        ),
        (
            "host_depleted_reads_r2",
            "benchmarks/readiness/local-ready/fastq.deplete_host/host_depleted_R2.fastq.gz",
        ),
        (
            "removed_host_reads_r1",
            "benchmarks/readiness/local-ready/fastq.deplete_host/removed_host_R1.fastq.gz",
        ),
        (
            "removed_host_reads_r2",
            "benchmarks/readiness/local-ready/fastq.deplete_host/removed_host_R2.fastq.gz",
        ),
        (
            "host_depletion_report_json",
            "benchmarks/readiness/local-ready/fastq.deplete_host/host_depletion_report.json",
        ),
    ] {
        assert_eq!(output_path(&plan, name), Path::new(expected));
    }

    for (key, expected) in [
        (
            "reference_index",
            serde_json::json!("assets/reference/host/references/toy_host_reference"),
        ),
        ("input_r2", serde_json::json!("assets/toy/core-v1/fastq/reads_2.fastq")),
        ("tool", serde_json::json!("bowtie2")),
        ("threads", serde_json::json!(4)),
        (
            "removed_host_r1",
            serde_json::json!(
                "benchmarks/readiness/local-ready/fastq.deplete_host/removed_host_R1.fastq.gz"
            ),
        ),
        (
            "removed_host_r2",
            serde_json::json!(
                "benchmarks/readiness/local-ready/fastq.deplete_host/removed_host_R2.fastq.gz"
            ),
        ),
    ] {
        assert_eq!(plan.params[key], expected);
    }
    assert_eq!(plan.effective_params["emit_removed_reads"], serde_json::json!(true));
    assert_eq!(plan.effective_params["reference_catalog_id"], serde_json::json!("host_reference"));
    let command = &plan.command.template;
    assert!(
        command.iter().any(|part| part == "assets/reference/host/references/toy_host_reference")
            && command.iter().any(|part| {
                part
                    == "benchmarks/readiness/local-ready/fastq.deplete_host/bowtie2.host.metrics.txt"
            })
            && command.iter().any(|part| {
                part
                    == "benchmarks/readiness/local-ready/fastq.deplete_host/host_depleted_R%.fastq.gz"
            })
            && command.iter().any(|part| {
                part
                    == "benchmarks/readiness/local-ready/fastq.deplete_host/removed_host_R%.fastq.gz"
            }),
        "local-ready plan command must materialize the governed Bowtie2 host index and metrics path"
    );
    Ok(())
}

#[test]
fn local_deplete_host_plan_stage_api_surface_stays_callable() {
    let _: fn(&std::path::Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_deplete_host_plan;
}
