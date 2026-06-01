#![cfg(feature = "bam_downstream")]

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
    let output_dir = repo_root.join("target/local-ready/bam.haplogroups");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::bam::write_local_haplogroups_plan()?;
    assert_eq!(plan_path, repo_root.join("target/local-ready/bam.haplogroups/plan.json"));
    assert!(plan_path.is_file(), "local-ready plan artifact must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.haplogroups"));
    assert_eq!(payload["tool_id"], serde_json::json!("yleaf"));
    assert_eq!(payload["resources"]["threads"], serde_json::json!(2));
    assert_eq!(payload["resources"]["mem_gb"], serde_json::json!(8));
    assert_eq!(payload["params"]["reference_panel_id"], serde_json::json!("toy-human-y-hg38"));
    assert_eq!(
        payload["params"]["reference_panel"],
        serde_json::json!("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
    );
    assert_eq!(
        payload["params"]["reference_fasta"],
        serde_json::json!("assets/reference/host/references/toy_human_y_reference.fasta")
    );
    assert_eq!(payload["params"]["reference_build"], serde_json::json!("hg38"));
    assert_eq!(
        payload["params"]["coverage_gate"],
        serde_json::json!({ "min_coverage": 2.0 })
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
        serde_json::json!("target/local-ready/bam.haplogroups/haplogroups.json")
    );

    assert!(
        payload["command"]["template"].as_array().is_some_and(|command| command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam.bai")
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("target/local-ready/bam.haplogroups/haplogroups")
            })
        )),
        "local-ready haplogroups command must carry the governed BAI, panel, and output prefix"
    );
    Ok(())
}
