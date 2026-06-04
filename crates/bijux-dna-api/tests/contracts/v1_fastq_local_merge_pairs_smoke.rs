use anyhow::{anyhow, Result};
use flate2::read::MultiGzDecoder;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
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
fn write_local_merge_pairs_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.merge_pairs");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::fastq::write_local_merge_pairs_smoke_report()?;
    assert_eq!(report_path, repo_root.join("target/local-smoke/fastq.merge_pairs/report.json"));
    assert!(report_path.is_file(), "local merge summary must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    let row = value_map(&payload)?;
    assert_eq!(row["stage_id"], "fastq.merge_pairs");
    assert_eq!(row["sample_id"], "human_like_pe_merge_overlap");
    assert_eq!(row["planned_tool_id"], "pear");
    assert_eq!(row["report_tool_id"], "bijux");
    assert_eq!(row["merge_overlap"], "8");
    assert_eq!(row["min_length"], "12");
    assert_eq!(row["input_pair_count"], "2");
    assert_eq!(row["merged_count"], "1");
    assert_eq!(row["unmerged_r1_count"], "1");
    assert_eq!(row["unmerged_r2_count"], "1");
    assert_eq!(row["discarded_count"], "0");

    let merged = read_gzip_fastq_sequences(&repo_root.join(&row["merged_fastq_gz"]))?;
    assert_eq!(merged, vec!["ACGTACGTGGGGTTAA".to_string()]);

    let unmerged_r1 = read_gzip_fastq_sequences(&repo_root.join(&row["unmerged_r1_fastq_gz"]))?;
    let unmerged_r2 = read_gzip_fastq_sequences(&repo_root.join(&row["unmerged_r2_fastq_gz"]))?;
    assert_eq!(unmerged_r1, vec!["GGGGAAAACCCC".to_string()]);
    assert_eq!(unmerged_r2, vec!["ATATATATATAT".to_string()]);

    let case_report_path = repo_root.join(&row["case_report_json"]);
    assert!(case_report_path.is_file(), "governed merge case report must exist");
    let case_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&case_report_path)?)?;
    assert_eq!(case_report["tool_id"], serde_json::json!("bijux"));
    assert_eq!(case_report["reads_r1"], serde_json::json!(2));
    assert_eq!(case_report["reads_r2"], serde_json::json!(2));
    assert_eq!(case_report["reads_merged"], serde_json::json!(1));
    assert_eq!(case_report["reads_unmerged"], serde_json::json!(1));
    let input_pair_count = case_report["reads_r1"]
        .as_u64()
        .ok_or_else(|| anyhow!("reads_r1 must be an integer"))?
        .min(
            case_report["reads_r2"]
                .as_u64()
                .ok_or_else(|| anyhow!("reads_r2 must be an integer"))?,
        );
    let merged_pair_count = case_report["reads_merged"]
        .as_u64()
        .ok_or_else(|| anyhow!("reads_merged must be an integer"))?
        .min(input_pair_count);
    let unmerged_pair_count = case_report["reads_unmerged"]
        .as_u64()
        .ok_or_else(|| anyhow!("reads_unmerged must be an integer"))?
        .min(input_pair_count.saturating_sub(merged_pair_count));
    let discarded_pair_count =
        input_pair_count.saturating_sub(merged_pair_count + unmerged_pair_count);
    assert_eq!(payload["input_pair_count"], serde_json::json!(input_pair_count));
    assert_eq!(payload["merged_count"], serde_json::json!(merged_pair_count));
    assert_eq!(payload["unmerged_r1_count"], serde_json::json!(unmerged_pair_count));
    assert_eq!(payload["unmerged_r2_count"], serde_json::json!(unmerged_pair_count));
    assert_eq!(payload["discarded_count"], serde_json::json!(discarded_pair_count));

    Ok(())
}

fn value_map(value: &serde_json::Value) -> Result<BTreeMap<String, String>> {
    let object = value.as_object().ok_or_else(|| anyhow!("report must be a JSON object"))?;
    Ok(object
        .iter()
        .map(|(key, value)| {
            let mapped =
                value.as_str().map(ToString::to_string).unwrap_or_else(|| value.to_string());
            (key.clone(), mapped)
        })
        .collect())
}

fn read_gzip_fastq_sequences(path: &Path) -> Result<Vec<String>> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(MultiGzDecoder::new(file));
    let mut lines = reader.lines();
    let mut sequences = Vec::new();
    loop {
        let Some(header) = lines.next() else {
            break;
        };
        let _header = header?;
        let sequence = lines.next().ok_or_else(|| anyhow!("missing sequence line"))??;
        let _plus = lines.next().ok_or_else(|| anyhow!("missing plus line"))??;
        let _quality = lines.next().ok_or_else(|| anyhow!("missing quality line"))??;
        sequences.push(sequence);
    }
    Ok(sequences)
}
