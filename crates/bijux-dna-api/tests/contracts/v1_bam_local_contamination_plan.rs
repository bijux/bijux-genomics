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
    assert_eq!(
        payload["params"]["sample_id"],
        serde_json::json!("human_like_contamination_panel_screen")
    );
    assert_eq!(payload["params"]["tool"], serde_json::json!("verifybamid2"));
    let inputs = payload["io"]["inputs"]
        .as_array()
        .unwrap_or_else(|| panic!("plan inputs must serialize as an array"));
    let bam = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("bam"))
        .unwrap_or_else(|| panic!("bam input missing from local-ready contamination plan payload"));
    assert_eq!(
        bam["path"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam"
        )
    );
    let bai = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("bam_bai"))
        .unwrap_or_else(|| {
            panic!("bam_bai input missing from local-ready contamination plan payload")
        });
    assert_eq!(
        bai["path"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam.bai"
        )
    );
    let reference = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("reference"))
        .unwrap_or_else(|| {
            panic!("reference input missing from local-ready contamination plan payload")
        });
    assert_eq!(
        reference["path"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
        )
    );
    let reference_panel = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("reference_panel"))
        .unwrap_or_else(|| {
            panic!("reference_panel input missing from local-ready contamination plan payload")
        });
    assert_eq!(
        reference_panel["path"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_contamination_panel.dat"
        )
    );
    assert_eq!(payload["params"]["scope"], serde_json::json!("nuclear"));
    assert_eq!(payload["params"]["prior"], serde_json::json!(0.02));
    assert_eq!(payload["params"]["sex_specific"], serde_json::json!(false));
    assert_eq!(payload["params"]["chromosome_system"], serde_json::json!("xy"));
    assert_eq!(payload["params"]["minimum_mean_coverage"], serde_json::json!(0.5));
    assert_eq!(payload["params"]["emit_confidence_caveats"], serde_json::json!(true));
    assert_eq!(
        payload["params"]["reference_panels"],
        serde_json::json!([
            "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_contamination_panel.dat"
        ])
    );
    assert_eq!(
        payload["params"]["assumptions"],
        serde_json::json!(
            "governed BAM corpus contamination panel with shared corpus reference for local contamination planning"
        )
    );
    assert_eq!(
        payload["params"]["required_reference_digest"],
        serde_json::json!("c2dc7ed50c21f1cf9663d03e215f6e0f25e8296ab5cded9efd941703cadbd07c")
    );
    assert_eq!(payload["params"]["tool_scope"], serde_json::json!("nuclear"));
    assert_eq!(payload["effective_params"]["chromosome_system"], serde_json::json!("xy"));
    assert_eq!(payload["effective_params"]["minimum_mean_coverage"], serde_json::json!(0.5));
    assert_eq!(payload["effective_params"]["emit_confidence_caveats"], serde_json::json!(true));
    assert_eq!(
        payload["effective_params"]["assumptions"],
        serde_json::json!(
            "governed BAM corpus contamination panel with shared corpus reference for local contamination planning"
        )
    );
    assert_eq!(
        payload["effective_params"]["required_reference_digest"],
        serde_json::json!("c2dc7ed50c21f1cf9663d03e215f6e0f25e8296ab5cded9efd941703cadbd07c")
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
    let contamination_summary = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("summary"))
        .unwrap_or_else(|| {
            panic!("summary output missing from local-ready contamination plan payload")
        });
    assert_eq!(
        contamination_summary["path"],
        serde_json::json!("target/local-ready/bam.contamination/contamination.summary.json")
    );
    let stage_metrics = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("stage_metrics"))
        .unwrap_or_else(|| {
            panic!("stage_metrics output missing from local-ready contamination plan payload")
        });
    assert_eq!(
        stage_metrics["path"],
        serde_json::json!("target/local-ready/bam.contamination/stage.metrics.json")
    );
    assert!(
        payload["command"]["template"].as_array().is_some_and(|command| command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam.bai"
                )
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains(
                    "tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
                )
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains(
                    "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_contamination_panel.dat"
                )
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("target/local-ready/bam.contamination/contamination.summary.json")
            })
        )),
        "local-ready contamination command must carry the governed BAI, reference, panel, and summary-output paths"
    );
    Ok(())
}
