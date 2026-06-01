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
fn write_local_correct_errors_smoke_plan_materializes_governed_target_output() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.correct_errors");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::fastq::write_local_correct_errors_smoke_plan()?;
    assert_eq!(plan_path, repo_root.join("target/local-smoke/fastq.correct_errors/plan.json"));
    assert!(plan_path.is_file(), "local correct-errors dry-run plan must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.correct_errors"));
    assert_eq!(payload["tool_id"], serde_json::json!("rcorrector"));
    assert_eq!(payload["resources"]["threads"], serde_json::json!(1));
    assert_eq!(
        payload["out_dir"],
        serde_json::json!("target/local-smoke/fastq.correct_errors/paired-dry-run/rcorrector")
    );
    assert_eq!(
        payload["io"]["inputs"][0]["path"],
        serde_json::json!("assets/toy/core-v1/fastq/reads_1.fastq")
    );
    assert_eq!(
        payload["io"]["inputs"][1]["path"],
        serde_json::json!("assets/toy/core-v1/fastq/reads_2.fastq")
    );
    assert_eq!(payload["params"]["tool"], serde_json::json!("rcorrector"));
    assert_eq!(payload["params"]["threads"], serde_json::json!(1));
    assert_eq!(payload["params"]["quality_encoding"], serde_json::json!("phred33"));
    assert_eq!(payload["params"]["conservative_mode"], serde_json::json!(false));
    assert_eq!(payload["effective_params"]["paired_mode"], serde_json::json!("paired_end"));
    assert_eq!(
        payload["effective_params"]["correction_engine"],
        serde_json::json!("rcorrector")
    );
    assert!(
        payload["command"]["template"]
            .as_array()
            .is_some_and(|command| command.iter().any(|part| part.as_str().is_some_and(|part| {
                part.contains("run_rcorrector.pl")
            })) && command.iter().any(|part| part.as_str().is_some_and(|part| {
                part.contains("assets/toy/core-v1/fastq/reads_1.fastq")
                    && part.contains("assets/toy/core-v1/fastq/reads_2.fastq")
            }))),
        "local correct-errors dry-run plan must carry the governed rcorrector command"
    );

    Ok(())
}
