use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
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
fn write_local_profile_reads_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/fastq.profile_reads");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_profile_reads_smoke_report()?;
    assert_eq!(
        report_path,
        repo_root.join("runs/bench/local-smoke/fastq.profile_reads/profile.json")
    );
    assert!(report_path.is_file(), "local profile summary must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.profile_reads"));
    assert_eq!(payload["case_count"], serde_json::json!(2));

    let cases = payload["cases"].as_array().ok_or_else(|| anyhow!("cases array missing"))?;
    let rows = cases.iter().map(case_map).collect::<Result<Vec<_>>>()?;

    let single_end = rows
        .iter()
        .find(|row| row["sample_id"] == "toy-se")
        .unwrap_or_else(|| panic!("toy-se profile case missing"));
    assert_eq!(single_end["layout"], "single_end");
    assert_eq!(single_end["reads_total"], "2");
    assert_eq!(single_end["bases_total"], "24");
    assert_eq!(single_end["mean_q"], "37.0");
    assert_eq!(single_end["gc_percent"], "50.0");

    let paired_end = rows
        .iter()
        .find(|row| row["sample_id"] == "toy-pe")
        .unwrap_or_else(|| panic!("toy-pe profile case missing"));
    assert_eq!(paired_end["layout"], "paired_end");
    assert_eq!(paired_end["reads_total"], "4");
    assert_eq!(paired_end["bases_total"], "48");
    assert_eq!(paired_end["mean_q"], "37.0");
    assert_eq!(paired_end["gc_percent"], "50.0");

    for row in rows {
        let report_json = repo_root.join(&row["report_json"]);
        let qc_tsv = repo_root.join(&row["qc_tsv"]);
        let qc_plots_dir = repo_root.join(&row["qc_plots_dir"]);
        assert!(report_json.is_file(), "governed profile report must exist");
        assert!(qc_tsv.is_file(), "governed profile TSV must exist");
        assert!(qc_plots_dir.is_dir(), "governed plots dir must exist");
        assert!(qc_plots_dir.join("length_histogram.json").is_file());
        assert!(qc_plots_dir.join("length_histogram.tsv").is_file());
    }

    Ok(())
}

fn case_map(value: &serde_json::Value) -> Result<BTreeMap<String, String>> {
    let object = value.as_object().ok_or_else(|| anyhow!("case row must be an object"))?;
    Ok(object
        .iter()
        .map(|(key, value)| {
            let mapped =
                value.as_str().map(ToString::to_string).unwrap_or_else(|| value.to_string());
            (key.clone(), mapped)
        })
        .collect())
}
