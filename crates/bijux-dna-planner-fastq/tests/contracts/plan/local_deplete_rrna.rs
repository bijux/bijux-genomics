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
fn local_deplete_rrna_plan_uses_governed_repo_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_fastq::stage_api::local_deplete_rrna_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "fastq.deplete_rrna");
    assert_eq!(plan.tool_id.as_str(), "sortmerna");
    assert_eq!(plan.resources.threads, 4);
    assert_eq!(plan.resources.mem_gb, 8);
    assert_eq!(plan.out_dir, PathBuf::from("benchmarks/readiness/local-ready/fastq.deplete_rrna"));
    for (name, expected) in [
        ("reads_r1", "assets/toy/core-v1/fastq/reads_1.fastq"),
        ("reads_r2", "assets/toy/core-v1/fastq/reads_2.fastq"),
        (
            "rrna_reference",
            "assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta",
        ),
    ] {
        assert_eq!(input_path(&plan, name), Path::new(expected));
    }
    for (name, expected) in [
        (
            "rrna_filtered_reads_r1",
            "benchmarks/readiness/local-ready/fastq.deplete_rrna/rrna_filtered_R1.fastq.gz",
        ),
        (
            "rrna_filtered_reads_r2",
            "benchmarks/readiness/local-ready/fastq.deplete_rrna/rrna_filtered_R2.fastq.gz",
        ),
        (
            "rrna_removed_reads_r1",
            "benchmarks/readiness/local-ready/fastq.deplete_rrna/removed_rrna_R1.fastq.gz",
        ),
        (
            "rrna_removed_reads_r2",
            "benchmarks/readiness/local-ready/fastq.deplete_rrna/removed_rrna_R2.fastq.gz",
        ),
    ] {
        assert_eq!(output_path(&plan, name), Path::new(expected));
    }

    for (key, expected) in [
        (
            "rrna_db",
            serde_json::json!(
                "assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta"
            ),
        ),
        ("input_r2", serde_json::json!("assets/toy/core-v1/fastq/reads_2.fastq")),
        (
            "removed_reads_r1",
            serde_json::json!(
                "benchmarks/readiness/local-ready/fastq.deplete_rrna/removed_rrna_R1.fastq.gz"
            ),
        ),
        (
            "removed_reads_r2",
            serde_json::json!(
                "benchmarks/readiness/local-ready/fastq.deplete_rrna/removed_rrna_R2.fastq.gz"
            ),
        ),
        ("tool", serde_json::json!("sortmerna")),
        ("threads", serde_json::json!(4)),
    ] {
        assert_eq!(plan.params[key], expected);
    }
    assert_eq!(plan.effective_params["emit_removed_reads"], serde_json::json!(true));
    assert!(
        plan.command.template[2].contains("sortmerna")
            && plan.command.template[2]
                .contains("assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta")
            && plan.command.template[2].contains(
                "benchmarks/readiness/local-ready/fastq.deplete_rrna/rrna_filtered_R1.fastq.gz"
            )
            && plan.command.template[2].contains(
                "benchmarks/readiness/local-ready/fastq.deplete_rrna/rrna_filtered_R2.fastq.gz"
            ),
        "local-ready plan command must materialize the governed SortMeRNA reference path"
    );
    Ok(())
}

#[test]
fn local_deplete_rrna_plan_stage_api_surface_stays_callable() {
    let _: fn(&std::path::Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_deplete_rrna_plan;
}
