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
fn write_local_deplete_reference_contaminants_plan_materializes_governed_target_output(
) -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-ready/fastq.deplete_reference_contaminants");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path =
        bijux_dna_api::v1::api::fastq::write_local_deplete_reference_contaminants_plan()?;
    assert_eq!(
        plan_path,
        repo_root.join("target/local-ready/fastq.deplete_reference_contaminants/plan.json")
    );
    assert!(plan_path.is_file(), "local-ready plan artifact must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;
    assert_eq!(
        payload["stage_id"],
        serde_json::json!("fastq.deplete_reference_contaminants")
    );
    assert_eq!(payload["tool_id"], serde_json::json!("bowtie2"));
    assert_eq!(payload["resources"]["threads"], serde_json::json!(4));
    assert_eq!(payload["resources"]["mem_gb"], serde_json::json!(8));
    assert_eq!(
        payload["io"]["inputs"][0]["path"],
        serde_json::json!("assets/toy/core-v1/fastq/reads_1.fastq")
    );
    assert_eq!(
        payload["io"]["inputs"][1]["path"],
        serde_json::json!("assets/reference/contaminants/references/toy_contaminant_reference")
    );
    assert_eq!(payload["params"]["decoy_mode"], serde_json::json!("phix_and_spikeins"));
    assert_eq!(
        payload["effective_params"]["reference_catalog_id"],
        serde_json::json!("contaminant_reference")
    );
    assert!(
        payload["command"]["template"]
            .as_array()
            .is_some_and(|command| command.iter().any(|part| {
                part == "assets/reference/contaminants/references/toy_contaminant_reference"
            }) && command.iter().any(|part| {
                part
                    == "target/local-ready/fastq.deplete_reference_contaminants/bowtie2.contaminant.metrics.txt"
            })),
        "local-ready plan command must carry the governed Bowtie2 contaminant-depletion command"
    );
    Ok(())
}
