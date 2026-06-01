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
fn write_local_genotyping_plan_materializes_governed_target_output() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-ready/bam.genotyping");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::bam::write_local_genotyping_plan()?;
    assert_eq!(plan_path, repo_root.join("target/local-ready/bam.genotyping/plan.json"));
    assert!(plan_path.is_file(), "local-ready plan artifact must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.genotyping"));
    assert_eq!(payload["tool_id"], serde_json::json!("angsd"));
    assert_eq!(payload["resources"]["threads"], serde_json::json!(2));
    assert_eq!(payload["resources"]["mem_gb"], serde_json::json!(8));
    assert_eq!(
        payload["params"]["reference"],
        serde_json::json!("assets/toy/core-v1/bam/genotyping_reference_chr1.fasta")
    );
    assert_eq!(
        payload["params"]["sites"],
        serde_json::json!("assets/toy/core-v1/vcf/genotyping_candidate_sites.vcf")
    );
    assert_eq!(
        payload["params"]["regions"],
        serde_json::json!("assets/toy/core-v1/bam/genotyping_target_regions.txt")
    );
    assert_eq!(
        payload["params"]["producer_contract"]["bcf"],
        serde_json::json!("target/local-ready/bam.genotyping/genotyping.bcf")
    );
    assert_eq!(
        payload["params"]["producer_contract"]["vcf"],
        serde_json::json!("target/local-ready/bam.genotyping/genotyping.vcf.gz")
    );
    assert_eq!(
        payload["params"]["sample_id"],
        serde_json::json!("core-v1-genotyping-panel-sites")
    );

    let outputs = payload["io"]["outputs"]
        .as_array()
        .unwrap_or_else(|| panic!("plan outputs must serialize as an array"));
    let bcf = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("genotyping_bcf"))
        .unwrap_or_else(|| panic!("genotyping_bcf output missing from local-ready payload"));
    assert_eq!(
        bcf["path"],
        serde_json::json!("target/local-ready/bam.genotyping/genotyping.bcf")
    );

    assert!(
        payload["command"]["template"].as_array().is_some_and(|command| command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/toy/core-v1/bam/genotyping_panel_sites.sam.bai")
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/toy/core-v1/bam/genotyping_reference_chr1.fasta")
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/toy/core-v1/vcf/genotyping_candidate_sites.vcf")
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/toy/core-v1/bam/genotyping_target_regions.txt")
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("target/local-ready/bam.genotyping/genotyping.bcf")
            })
        )),
        "local-ready genotyping command must carry the governed BAI, reference, sites, regions, and BCF output"
    );
    Ok(())
}
