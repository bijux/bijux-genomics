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
fn write_local_profile_read_lengths_smoke_summary_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.profile_read_lengths");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let summary_path =
        bijux_dna_api::v1::api::fastq::write_local_profile_read_lengths_smoke_summary()?;
    assert_eq!(
        summary_path,
        repo_root.join("target/local-smoke/fastq.profile_read_lengths/read_lengths.tsv")
    );
    assert!(summary_path.is_file(), "local-smoke read-length summary must exist");

    let body = std::fs::read_to_string(&summary_path)?;
    let mut lines = body.lines();
    let header = lines.next().ok_or_else(|| anyhow!("summary header missing"))?;
    let header_index = parse_header_index(header);
    for column in ["sample_id", "min_len", "max_len", "mean_len", "median_len", "read_count"] {
        assert!(header_index.contains_key(column), "summary header must contain `{column}`");
    }

    let rows = lines.map(|line| parse_row(&header_index, line)).collect::<Result<Vec<_>>>()?;
    assert_eq!(rows.len(), 2, "governed local-smoke summary must keep SE and PE coverage");

    let single_end = rows
        .iter()
        .find(|row| row["sample_id"] == "toy-se")
        .unwrap_or_else(|| panic!("toy-se row missing from read-length summary"));
    assert_eq!(single_end["min_len"], "12");
    assert_eq!(single_end["max_len"], "12");
    assert_eq!(single_end["mean_len"], "12");
    assert_eq!(single_end["median_len"], "12");
    assert_eq!(single_end["read_count"], "2");
    assert_eq!(single_end["layout"], "single_end");

    let paired_end = rows
        .iter()
        .find(|row| row["sample_id"] == "toy-pe")
        .unwrap_or_else(|| panic!("toy-pe row missing from read-length summary"));
    assert_eq!(paired_end["min_len"], "12");
    assert_eq!(paired_end["max_len"], "12");
    assert_eq!(paired_end["mean_len"], "12");
    assert_eq!(paired_end["median_len"], "12");
    assert_eq!(paired_end["read_count"], "4");
    assert_eq!(paired_end["layout"], "paired_end");

    for row in &rows {
        let report_json = repo_root.join(&row["report_json"]);
        let length_distribution_tsv = repo_root.join(&row["length_distribution_tsv"]);
        let length_distribution_json = repo_root.join(&row["length_distribution_json"]);
        assert!(report_json.is_file(), "profile report must exist");
        assert!(length_distribution_tsv.is_file(), "length distribution TSV must exist");
        assert!(length_distribution_json.is_file(), "length distribution JSON must exist");
    }

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
