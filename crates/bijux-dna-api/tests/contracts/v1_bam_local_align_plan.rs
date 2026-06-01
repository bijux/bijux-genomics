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
fn write_local_align_plan_materializes_governed_target_output() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-ready/bam.align");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::bam::write_local_align_plan()?;
    assert_eq!(plan_path, repo_root.join("target/local-ready/bam.align/plan.json"));
    assert!(plan_path.is_file(), "local-ready plan artifact must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.align"));
    assert_eq!(payload["tool_id"], serde_json::json!("bowtie2"));
    assert_eq!(payload["resources"]["threads"], serde_json::json!(4));
    assert_eq!(payload["resources"]["mem_gb"], serde_json::json!(8));
    assert_eq!(
        payload["io"]["inputs"][0]["path"],
        serde_json::json!("assets/toy/core-v1/fastq/reads_1.fastq")
    );
    assert_eq!(
        payload["io"]["inputs"][1]["path"],
        serde_json::json!("assets/reference/host/references/toy_host_reference.fasta")
    );
    assert_eq!(
        payload["params"]["reference_index"],
        serde_json::json!("assets/reference/host/references/toy_host_reference")
    );
    assert_eq!(
        payload["params"]["reference_dict"],
        serde_json::json!("assets/reference/host/references/toy_host_reference.dict")
    );
    let outputs = payload["io"]["outputs"]
        .as_array()
        .unwrap_or_else(|| panic!("plan outputs must serialize as an array"));
    let align_metrics = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("align_metrics"))
        .unwrap_or_else(|| panic!("align_metrics output missing from local-ready plan payload"));
    assert_eq!(
        align_metrics["path"],
        serde_json::json!("target/local-ready/bam.align/align.metrics.json")
    );
    let align_bam = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("align_bam"))
        .unwrap_or_else(|| panic!("align_bam output missing from local-ready plan payload"));
    assert_eq!(
        align_bam["path"],
        serde_json::json!("target/local-ready/bam.align/align.bam")
    );
    assert!(
        payload["command"]["template"]
            .as_array()
            .is_some_and(|command| command.iter().any(|part| {
                part.as_str().is_some_and(|shell| {
                    shell.contains("assets/reference/host/references/toy_host_reference.fasta")
                })
            }) && command.iter().any(|part| {
                part.as_str().is_some_and(|shell| {
                    shell.contains("-x assets/reference/host/references/toy_host_reference")
                })
            })),
        "local-ready plan command must carry the governed Bowtie2 FASTA and index-prefix paths"
    );
    Ok(())
}
