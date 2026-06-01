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
fn write_local_mapping_summary_smoke_summary_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.mapping_summary");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let summary_path = bijux_dna_api::v1::api::bam::write_local_mapping_summary_smoke_summary()?;
    assert_eq!(
        summary_path,
        repo_root.join("target/local-smoke/bam.mapping_summary/mapping_summary.tsv")
    );
    assert!(summary_path.is_file(), "local-smoke BAM mapping summary TSV must exist");

    let body = std::fs::read_to_string(&summary_path)?;
    let mut lines = body.lines();
    let header = lines.next().ok_or_else(|| anyhow!("summary header missing"))?;
    let header_index = parse_header_index(header);
    for column in [
        "sample_id",
        "total_reads",
        "mapped_reads",
        "mapping_fraction",
        "reference_name",
        "expectation_matched",
        "mapping_summary_json",
        "flagstat",
        "idxstats",
        "samtools_stats",
        "stage_metrics",
    ] {
        assert!(header_index.contains_key(column), "summary header must contain `{column}`");
    }

    let rows = lines.map(|line| parse_row(&header_index, line)).collect::<Result<Vec<_>>>()?;
    assert_eq!(rows.len(), 1, "governed local-smoke summary must keep exactly one BAM case");

    let row = rows
        .iter()
        .find(|row| row["sample_id"] == "core-v1-partial-mapping")
        .unwrap_or_else(|| panic!("core-v1-partial-mapping row missing from BAM mapping summary"));
    assert_eq!(row["total_reads"], "3");
    assert_eq!(row["mapped_reads"], "2");
    assert_eq!(row["mapping_fraction"], "0.666667");
    assert_eq!(row["reference_name"], "chr1");
    assert_eq!(row["expectation_matched"], "true");

    let mapping_summary_json = repo_root.join(&row["mapping_summary_json"]);
    let flagstat = repo_root.join(&row["flagstat"]);
    let idxstats = repo_root.join(&row["idxstats"]);
    let samtools_stats = repo_root.join(&row["samtools_stats"]);
    let stage_metrics = repo_root.join(&row["stage_metrics"]);
    assert!(mapping_summary_json.is_file(), "case mapping summary JSON must exist");
    assert!(flagstat.is_file(), "case flagstat must exist");
    assert!(idxstats.is_file(), "case idxstats must exist");
    assert!(samtools_stats.is_file(), "case samtools stats must exist");
    assert!(stage_metrics.is_file(), "case stage metrics must exist");

    let mapping_summary: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mapping_summary_json)?)?;
    assert_eq!(
        mapping_summary["schema_version"],
        serde_json::json!("bijux.bam.mapping_summary.v1")
    );
    assert_eq!(
        mapping_summary["flagstat"]["total_reads"],
        serde_json::json!(3)
    );
    assert_eq!(
        mapping_summary["flagstat"]["mapped_reads"],
        serde_json::json!(2)
    );

    let stage_metrics_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(
        stage_metrics_json["schema_version"],
        serde_json::json!("bijux.bam.mapping_summary.local_smoke.metrics.v1")
    );
    assert_eq!(stage_metrics_json["reference_name"], serde_json::json!("chr1"));

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
