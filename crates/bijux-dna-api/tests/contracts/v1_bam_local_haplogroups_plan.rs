#![cfg(feature = "bam_downstream")]
#![allow(clippy::too_many_lines)]

use anyhow::Result;
use std::path::{Path, PathBuf};

struct RepoRootOverrideGuard {
    previous: Option<std::ffi::OsString>,
}

impl RepoRootOverrideGuard {
    fn install(root: &Path) -> Self {
        let previous = std::env::var_os("BIJUX_REPO_ROOT");
        std::env::set_var("BIJUX_REPO_ROOT", root);
        Self { previous }
    }
}

impl Drop for RepoRootOverrideGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            std::env::set_var("BIJUX_REPO_ROOT", previous);
        } else {
            std::env::remove_var("BIJUX_REPO_ROOT");
        }
    }
}

fn repo_root() -> Result<PathBuf> {
    crate::support::repo_root()
}

#[test]
fn write_local_haplogroups_plan_materializes_governed_target_output() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("benchmarks/readiness/local-ready/bam.haplogroups");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::bam::write_local_haplogroups_plan()?;
    assert_eq!(
        plan_path,
        repo_root.join("benchmarks/readiness/local-ready/bam.haplogroups/plan.json")
    );
    assert!(plan_path.is_file(), "local-ready plan artifact must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.haplogroups"));
    assert_eq!(payload["tool_id"], serde_json::json!("yleaf"));
    assert_eq!(payload["resources"]["threads"], serde_json::json!(2));
    assert_eq!(payload["resources"]["mem_gb"], serde_json::json!(8));
    assert_eq!(payload["params"]["reference_panel_id"], serde_json::json!("adna-y-hg38-mini"));
    assert_eq!(
        payload["params"]["reference_panel"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_y_haplogroup_panel.tsv"
        )
    );
    assert_eq!(
        payload["params"]["reference_fasta"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta"
        )
    );
    assert_eq!(payload["params"]["reference_build"], serde_json::json!("hg38"));
    assert_eq!(payload["params"]["population_scope"], serde_json::json!("adna_y_haplogroup_panel"));
    assert_eq!(payload["params"]["coverage_gate"], serde_json::json!({ "min_coverage": 2.0 }));
    assert_eq!(payload["params"]["sample_id"], serde_json::json!("adna_y_haplogroup_panel"));

    let inputs = payload["io"]["inputs"]
        .as_array()
        .unwrap_or_else(|| panic!("plan inputs must serialize as an array"));
    let bam = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("bam"))
        .unwrap_or_else(|| panic!("bam input missing from local-ready haplogroups plan payload"));
    assert_eq!(
        bam["path"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_y_haplogroup_panel.sam"
        )
    );
    let input_index = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("bam_bai"))
        .unwrap_or_else(|| {
            panic!("bam_bai input missing from local-ready haplogroups plan payload")
        });
    assert_eq!(
        input_index["path"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_y_haplogroup_panel.sam.bai"
        )
    );
    let reference = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("reference"))
        .unwrap_or_else(|| {
            panic!("reference input missing from local-ready haplogroups plan payload")
        });
    assert_eq!(
        reference["path"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta"
        )
    );
    let reference_panel = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("reference_panel"))
        .unwrap_or_else(|| {
            panic!("reference_panel input missing from local-ready haplogroups plan payload")
        });
    assert_eq!(
        reference_panel["path"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_y_haplogroup_panel.tsv"
        )
    );

    let outputs = payload["io"]["outputs"]
        .as_array()
        .unwrap_or_else(|| panic!("plan outputs must serialize as an array"));
    let haplogroups_report = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("haplogroups"))
        .unwrap_or_else(|| {
            panic!("haplogroups output missing from local-ready haplogroups plan payload")
        });
    assert_eq!(
        haplogroups_report["path"],
        serde_json::json!("benchmarks/readiness/local-ready/bam.haplogroups/haplogroups.json")
    );
    let summary = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("summary"))
        .unwrap_or_else(|| {
            panic!("summary output missing from local-ready haplogroups plan payload")
        });
    assert_eq!(
        summary["path"],
        serde_json::json!(
            "benchmarks/readiness/local-ready/bam.haplogroups/haplogroups.summary.json"
        )
    );
    let stage_metrics = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("stage_metrics"))
        .unwrap_or_else(|| {
            panic!("stage_metrics output missing from local-ready haplogroups plan payload")
        });
    assert_eq!(
        stage_metrics["path"],
        serde_json::json!("benchmarks/readiness/local-ready/bam.haplogroups/stage.metrics.json")
    );

    assert!(
        payload["command"]["template"].as_array().is_some_and(|command| command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains(
                    "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_y_haplogroup_panel.sam.bai"
                )
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains(
                    "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_y_haplogroup_panel.tsv"
                )
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("benchmarks/readiness/local-ready/bam.haplogroups/haplogroups")
            })
        )),
        "local-ready haplogroups command must carry the governed BAI, panel, and output prefix"
    );
    Ok(())
}

#[test]
fn write_local_haplogroups_plan_preserves_governed_command_metadata() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("benchmarks/readiness/local-ready/bam.haplogroups");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::bam::write_local_haplogroups_plan()?;
    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;

    assert_eq!(
        payload["out_dir"],
        serde_json::json!("benchmarks/readiness/local-ready/bam.haplogroups")
    );
    assert_eq!(payload["effective_params"]["min_coverage"], serde_json::json!(2.0));
    assert_eq!(payload["effective_params"]["reference_build"], serde_json::json!("hg38"));
    assert_eq!(
        payload["effective_params"]["population_scope"],
        serde_json::json!("adna_y_haplogroup_panel")
    );
    assert_eq!(
        payload["effective_params"]["reference_panel"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_y_haplogroup_panel.tsv"
        )
    );
    assert_eq!(
        payload["effective_params"]["refuse_without_population_context"],
        serde_json::json!(true)
    );

    let command = payload["command"]["template"]
        .as_array()
        .and_then(|template| template.last())
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("local-ready haplogroups plan must serialize a shell command"));
    assert!(
        command.contains(
            "yleaf -bam benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_y_haplogroup_panel.sam"
        )
            && command.contains("--reference_genome hg38")
            && command.contains(
                "benchmarks/readiness/local-ready/bam.haplogroups/haplogroups.summary.json"
            )
            && command.contains("\"population_scope\":\"adna_y_haplogroup_panel\"")
            && command.contains("\"min_coverage\":2.0")
            && command.contains("\"assignment_output_prefix\":\"benchmarks/readiness/local-ready/bam.haplogroups/haplogroups\""),
        "local-ready haplogroups command must preserve the governed tool, reference build, summary, population scope, coverage gate, and assignment output prefix"
    );

    Ok(())
}
