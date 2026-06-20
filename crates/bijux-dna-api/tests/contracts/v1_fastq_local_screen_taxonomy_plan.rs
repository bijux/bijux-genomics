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
fn write_local_screen_taxonomy_plan_materializes_governed_target_output() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("benchmarks/readiness/local-ready/fastq.screen_taxonomy");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::fastq::write_local_screen_taxonomy_plan()?;
    assert_eq!(
        plan_path,
        repo_root.join("benchmarks/readiness/local-ready/fastq.screen_taxonomy/plan.json")
    );
    assert!(plan_path.is_file(), "local-ready taxonomy plan artifact must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;
    let inputs = payload["io"]["inputs"]
        .as_array()
        .unwrap_or_else(|| panic!("plan inputs missing"));
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.screen_taxonomy"));
    assert_eq!(payload["tool_id"], serde_json::json!("kraken2"));
    assert_eq!(payload["resources"]["threads"], serde_json::json!(4));
    assert_eq!(payload["resources"]["mem_gb"], serde_json::json!(16));
    assert_eq!(
        inputs
            .iter()
            .find(|artifact| artifact["name"] == "reads_r1")
            .unwrap_or_else(|| panic!("reads_r1 input missing"))["path"],
        serde_json::json!("assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq")
    );
    assert_eq!(
        inputs
            .iter()
            .find(|artifact| artifact["name"] == "reads_r2")
            .unwrap_or_else(|| panic!("reads_r2 input missing"))["path"],
        serde_json::json!("assets/toy/corpus-02-edna-mini/fastq/mock_community_reads_R2.fastq")
    );
    assert_eq!(
        inputs
            .iter()
            .find(|artifact| artifact["name"] == "taxonomy_database_root")
            .unwrap_or_else(|| panic!("taxonomy_database_root input missing"))["path"],
        serde_json::json!("assets/reference/taxonomy/references/mock_community_taxonomy")
    );
    assert_eq!(
        payload["params"]["database_root"],
        serde_json::json!("assets/reference/taxonomy/references/mock_community_taxonomy")
    );
    assert_eq!(payload["effective_params"]["emit_unclassified"], serde_json::json!(true));
    assert_eq!(payload["effective_params"]["paired_mode"], serde_json::json!("paired_end"));
    assert_eq!(
        payload["params"]["unclassified_reads_r1"],
        serde_json::json!(
            "benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.unclassified_reads_1.fastq"
        )
    );
    assert_eq!(
        payload["params"]["unclassified_reads_r2"],
        serde_json::json!(
            "benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.unclassified_reads_2.fastq"
        )
    );
    assert!(
        payload["command"]["template"]
            .as_array()
            .and_then(|command| command.get(2))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|script| {
                script.contains("--db 'assets/reference/taxonomy/references/mock_community_taxonomy/kraken2'")
                    && script.contains("'benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.report.tsv'")
                    && script.contains("'benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.classifications.native.tsv'")
                    && script.contains("--paired 'assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq' 'assets/toy/corpus-02-edna-mini/fastq/mock_community_reads_R2.fastq'")
                    && script.contains("--unclassified-out 'benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.unclassified_reads_#.fastq'")
            }),
        "local-ready taxonomy plan command must carry the governed taxonomy database root and report path"
    );
    Ok(())
}
