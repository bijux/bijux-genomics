use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const GOVERNED_COVERAGE_SAMPLE_ID: &str = "human_like_target_window_coverage";

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
fn write_local_coverage_smoke_summary_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("runs/bench/local-smoke/bam.coverage");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let summary_path = bijux_dna_api::v1::api::bam::write_local_coverage_smoke_summary()?;
    assert_eq!(summary_path, repo_root.join("runs/bench/local-smoke/bam.coverage/coverage.tsv"));
    assert!(summary_path.is_file(), "local-smoke BAM coverage TSV must exist");

    let body = std::fs::read_to_string(&summary_path)?;
    let mut lines = body.lines();
    let header = lines.next().ok_or_else(|| anyhow!("summary header missing"))?;
    let header_index = parse_header_index(header);
    for column in [
        "sample_id",
        "region_id",
        "contig",
        "mean_depth",
        "breadth_1x",
        "covered_bases",
        "coverage_regime",
        "row_expectation_matched",
        "case_expectation_matched",
        "coverage_tsv",
        "coverage_summary_json",
        "coverage_depth",
        "coverage_mosdepth_summary",
        "stage_metrics",
    ] {
        assert!(header_index.contains_key(column), "summary header must contain `{column}`");
    }

    let rows = lines.map(|line| parse_row(&header_index, line)).collect::<Result<Vec<_>>>()?;
    assert_eq!(rows.len(), 2, "governed local-smoke coverage summary must keep two region rows");

    let chr1_row = rows
        .iter()
        .find(|row| row["region_id"] == "chr1_window")
        .unwrap_or_else(|| panic!("chr1_window row missing from BAM coverage summary"));
    assert_eq!(chr1_row["sample_id"], GOVERNED_COVERAGE_SAMPLE_ID);
    assert_eq!(chr1_row["contig"], "chr1");
    assert_eq!(chr1_row["mean_depth"], "1.333333");
    assert_eq!(chr1_row["breadth_1x"], "1.000000");
    assert_eq!(chr1_row["covered_bases"], "6");
    assert_eq!(chr1_row["coverage_regime"], "low_pass");
    assert_eq!(chr1_row["row_expectation_matched"], "true");
    assert_eq!(chr1_row["case_expectation_matched"], "true");

    let chr2_row = rows
        .iter()
        .find(|row| row["region_id"] == "chr2_window")
        .unwrap_or_else(|| panic!("chr2_window row missing from BAM coverage summary"));
    assert_eq!(chr2_row["contig"], "chr2");
    assert_eq!(chr2_row["mean_depth"], "0.750000");
    assert_eq!(chr2_row["breadth_1x"], "0.750000");
    assert_eq!(chr2_row["covered_bases"], "3");

    let coverage_tsv = repo_root.join(&chr1_row["coverage_tsv"]);
    let coverage_summary_json = repo_root.join(&chr1_row["coverage_summary_json"]);
    let coverage_depth = repo_root.join(&chr1_row["coverage_depth"]);
    let coverage_summary_artifact = repo_root.join(&chr1_row["coverage_mosdepth_summary"]);
    let stage_metrics = repo_root.join(&chr1_row["stage_metrics"]);
    for path in [
        &coverage_tsv,
        &coverage_summary_json,
        &coverage_depth,
        &coverage_summary_artifact,
        &stage_metrics,
    ] {
        assert!(path.is_file(), "governed BAM coverage artifact must exist: {}", path.display());
    }

    let case_tsv = std::fs::read_to_string(&coverage_tsv)?;
    assert!(case_tsv.contains("chr1_window\tchr1\t1\t6\t6\t1.333333\t1.000000\t6"));
    assert!(case_tsv.contains("chr2_window\tchr2\t2\t5\t4\t0.750000\t0.750000\t3"));

    let coverage_summary: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&coverage_summary_json)?)?;
    assert_eq!(
        coverage_summary["schema_version"],
        serde_json::json!("bijux.bam.coverage_summary.v1")
    );
    assert_eq!(coverage_summary["stage_id"], serde_json::json!("bam.coverage"));
    assert_eq!(coverage_summary["coverage_regime"], serde_json::json!("low_pass"));
    assert_eq!(coverage_summary["depth_thresholds"], serde_json::json!([1, 5]));
    assert_eq!(coverage_summary["mean_depth"], serde_json::json!(1.1));

    let depth_body = std::fs::read_to_string(&coverage_depth)?;
    assert!(depth_body.contains("chr1\t1\t1"));
    assert!(depth_body.contains("chr1\t3\t2"));
    assert!(depth_body.contains("chr2\t5\t0"));

    let summary_artifact_body = std::fs::read_to_string(&coverage_summary_artifact)?;
    assert_eq!(summary_artifact_body, "total\t10\t9\t1.100000\n");

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.coverage.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["depth_thresholds"], serde_json::json!([1, 5]));
    assert_eq!(stage_metrics_json["observed_coverage_regime"], serde_json::json!("low_pass"));
    assert_eq!(stage_metrics_json["expected_region_count"], serde_json::json!(2));
    assert_eq!(stage_metrics_json["observed_region_count"], serde_json::json!(2));
    assert_eq!(stage_metrics_json["case_expectation_matched"], serde_json::json!(true));
    assert_eq!(stage_metrics_json["region_ids"], serde_json::json!(["chr1_window", "chr2_window"]));

    Ok(())
}

fn parse_header_index(header: &str) -> BTreeMap<String, usize> {
    header.split('\t').enumerate().map(|(index, column)| (column.to_string(), index)).collect()
}

fn parse_row(
    header_index: &BTreeMap<String, usize>,
    line: &str,
) -> Result<BTreeMap<String, String>> {
    let fields = line.split('\t').map(str::to_string).collect::<Vec<_>>();
    let mut row = BTreeMap::new();
    for (column, index) in header_index {
        let value =
            fields.get(*index).ok_or_else(|| anyhow!("summary row is missing `{column}`"))?;
        row.insert(column.clone(), value.clone());
    }
    Ok(row)
}
