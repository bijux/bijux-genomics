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
fn write_local_contamination_plan_materializes_governed_target_output() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-ready/bam.contamination");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::bam::write_local_contamination_plan()?;
    assert_eq!(plan_path, repo_root.join("target/local-ready/bam.contamination/plan.json"));
    assert!(plan_path.is_file(), "local-ready plan artifact must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.contamination"));
    assert_eq!(payload["tool_id"], serde_json::json!("verifybamid2"));
    assert_eq!(payload["resources"]["threads"], serde_json::json!(2));
    assert_eq!(payload["resources"]["mem_gb"], serde_json::json!(8));
    assert_eq!(payload["params"]["scope"], serde_json::json!("nuclear"));
    assert_eq!(
        payload["params"]["reference_panels"],
        serde_json::json!(["assets/reference/host/references/toy_human_contamination_panel.dat"])
    );
    assert_eq!(
        payload["params"]["assumptions"],
        serde_json::json!(
            "toy host reference with governed population-af panel for local contamination planning"
        )
    );
    let outputs = payload["io"]["outputs"]
        .as_array()
        .unwrap_or_else(|| panic!("plan outputs must serialize as an array"));
    let contamination_report = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("contamination_report"))
        .unwrap_or_else(|| {
            panic!(
                "contamination_report output missing from local-ready contamination plan payload"
            )
        });
    assert_eq!(
        contamination_report["path"],
        serde_json::json!("target/local-ready/bam.contamination/contamination.json")
    );
    assert!(
        payload["command"]["template"].as_array().is_some_and(|command| command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/toy/core-v1/bam/contamination_panel_screen.sam.bai")
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/reference/host/references/toy_host_reference.fasta")
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/reference/host/references/toy_human_contamination_panel.dat")
            })
        )),
        "local-ready contamination command must carry the governed BAI, reference, and panel paths"
    );
    Ok(())
}
