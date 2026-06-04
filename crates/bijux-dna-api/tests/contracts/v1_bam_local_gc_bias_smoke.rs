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
fn write_local_gc_bias_smoke_summary_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.gc_bias");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let summary_path = bijux_dna_api::v1::api::bam::write_local_gc_bias_smoke_summary()?;
    assert_eq!(summary_path, repo_root.join("target/local-smoke/bam.gc_bias/gc_bias.tsv"));
    assert!(summary_path.is_file(), "local-smoke BAM gc-bias TSV must exist");

    let body = std::fs::read_to_string(&summary_path)?;
    let mut lines = body.lines();
    let header = lines.next().ok_or_else(|| anyhow!("summary header missing"))?;
    assert_eq!(
        header,
        "sample_id\tgc_bin\tnormalized_coverage\twindows\tread_starts\tinsufficient_reference_reason\trow_expectation_matched\tcase_expectation_matched\tinput_bam\treference_fasta\tgc_bias_tsv\tgc_bias_summary_json\tgc_bias_metrics\tgc_bias_plot\tstage_metrics"
    );
    let header_index = parse_header_index(header);
    let rows = lines.map(|line| parse_row(&header_index, line)).collect::<Result<Vec<_>>>()?;
    assert_eq!(rows.len(), 3, "governed local-smoke gc-bias summary must keep three GC rows");

    let zero_gc_row = rows
        .iter()
        .find(|row| row["gc_bin"] == "0")
        .unwrap_or_else(|| panic!("0% GC row missing from BAM gc-bias summary"));
    assert_eq!(zero_gc_row["sample_id"], "human_like_gc_window_ladder");
    assert_eq!(zero_gc_row["normalized_coverage"], "0.750000");
    assert_eq!(zero_gc_row["windows"], "1");
    assert_eq!(zero_gc_row["read_starts"], "1");
    assert_eq!(zero_gc_row["insufficient_reference_reason"], "");
    assert_eq!(zero_gc_row["row_expectation_matched"], "true");
    assert_eq!(zero_gc_row["case_expectation_matched"], "true");
    assert_eq!(
        zero_gc_row["input_bam"],
        "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
    );
    assert_eq!(
        zero_gc_row["reference_fasta"],
        "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
    );

    let mid_gc_row = rows
        .iter()
        .find(|row| row["gc_bin"] == "50")
        .unwrap_or_else(|| panic!("50% GC row missing from BAM gc-bias summary"));
    assert_eq!(mid_gc_row["normalized_coverage"], "1.500000");
    assert_eq!(mid_gc_row["windows"], "1");
    assert_eq!(mid_gc_row["read_starts"], "2");
    assert_eq!(mid_gc_row["row_expectation_matched"], "true");
    assert_eq!(mid_gc_row["case_expectation_matched"], "true");

    let high_gc_row = rows
        .iter()
        .find(|row| row["gc_bin"] == "100")
        .unwrap_or_else(|| panic!("100% GC row missing from BAM gc-bias summary"));
    assert_eq!(high_gc_row["normalized_coverage"], "0.750000");
    assert_eq!(high_gc_row["windows"], "1");
    assert_eq!(high_gc_row["read_starts"], "1");
    assert_eq!(high_gc_row["row_expectation_matched"], "true");
    assert_eq!(high_gc_row["case_expectation_matched"], "true");

    let gc_bias_tsv = repo_root.join(&zero_gc_row["gc_bias_tsv"]);
    let gc_bias_summary_json = repo_root.join(&zero_gc_row["gc_bias_summary_json"]);
    let gc_bias_metrics = repo_root.join(&zero_gc_row["gc_bias_metrics"]);
    let gc_bias_plot = repo_root.join(&zero_gc_row["gc_bias_plot"]);
    let stage_metrics = repo_root.join(&zero_gc_row["stage_metrics"]);
    for path in
        [&gc_bias_tsv, &gc_bias_summary_json, &gc_bias_metrics, &gc_bias_plot, &stage_metrics]
    {
        assert!(path.is_file(), "governed BAM gc-bias artifact must exist: {}", path.display());
    }

    let case_tsv = std::fs::read_to_string(&gc_bias_tsv)?;
    assert!(case_tsv.contains("gc_bin\tnormalized_coverage\twindows\tread_starts"));
    assert!(case_tsv.contains("0\t0.750000\t1\t1"));
    assert!(case_tsv.contains("50\t1.500000\t1\t2"));
    assert!(case_tsv.contains("100\t0.750000\t1\t1"));

    let summary_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&gc_bias_summary_json)?)?;
    assert_eq!(summary_json["schema_version"], serde_json::json!("bijux.bam.gc_bias.v1"));
    assert_eq!(summary_json["stage_id"], serde_json::json!("bam.gc_bias"));
    assert_eq!(summary_json["window_size"], serde_json::json!(10));
    assert_eq!(summary_json["total_clusters"], serde_json::json!(4));
    assert_eq!(summary_json["aligned_reads"], serde_json::json!(4));
    assert_eq!(summary_json["windows"], serde_json::json!(3));
    assert_eq!(summary_json["read_starts"], serde_json::json!(4));
    assert_eq!(summary_json["report_present"], serde_json::json!(true));
    assert_eq!(summary_json["plot_present"], serde_json::json!(true));
    assert_eq!(summary_json["at_dropout"], serde_json::json!(25.0));
    assert_eq!(summary_json["gc_dropout"], serde_json::json!(25.0));
    assert_eq!(summary_json["gc_bias_score"], serde_json::json!(0.25));
    assert_eq!(summary_json["insufficient_reference_reason"], serde_json::Value::Null);

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.gc_bias.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["window_size"], serde_json::json!(10));
    assert_eq!(stage_metrics_json["expected_row_count"], serde_json::json!(3));
    assert_eq!(stage_metrics_json["observed_row_count"], serde_json::json!(3));
    assert_eq!(stage_metrics_json["gc_bias_score"], serde_json::json!(0.25));
    assert_eq!(stage_metrics_json["at_dropout"], serde_json::json!(25.0));
    assert_eq!(stage_metrics_json["gc_dropout"], serde_json::json!(25.0));
    assert_eq!(stage_metrics_json["insufficient_reference_reason"], serde_json::Value::Null);
    assert_eq!(stage_metrics_json["observed_gc_bins"], serde_json::json!([0, 50, 100]));
    assert_eq!(stage_metrics_json["row_expectation_matched"], serde_json::json!(true));
    assert_eq!(stage_metrics_json["case_expectation_matched"], serde_json::json!(true));

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
