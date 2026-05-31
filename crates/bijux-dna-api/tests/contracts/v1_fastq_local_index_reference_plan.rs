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
fn write_local_index_reference_plan_materializes_governed_target_output() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-ready/fastq.index_reference");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::fastq::write_local_index_reference_plan()?;
    assert_eq!(plan_path, repo_root.join("target/local-ready/fastq.index_reference/plan.json"));
    assert!(plan_path.is_file(), "local-ready plan artifact must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.index_reference"));
    assert_eq!(payload["tool_id"], serde_json::json!("bowtie2_build"));
    assert_eq!(payload["resources"]["threads"], serde_json::json!(4));
    assert_eq!(payload["resources"]["mem_gb"], serde_json::json!(8));
    assert_eq!(
        payload["io"]["inputs"][0]["path"],
        serde_json::json!("assets/reference/contaminants/references/phix174.fasta")
    );
    assert!(
        payload["command"]["template"][2]
            .as_str()
            .is_some_and(|script| script.contains("bowtie2-build --threads 4")),
        "local-ready plan command must carry the governed bowtie2-build dry-run command"
    );
    Ok(())
}
