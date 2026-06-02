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
fn write_local_qc_pre_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/bam.qc_pre");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let report_path = bijux_dna_api::v1::api::bam::write_local_qc_pre_smoke_report()?;
    assert_eq!(report_path, repo_root.join("target/local-smoke/bam.qc_pre/qc_pre.json"));
    assert!(report_path.is_file(), "local-smoke BAM qc_pre summary must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.qc_pre"));
    assert_eq!(payload["case_count"], serde_json::json!(1));
    assert_eq!(payload["all_cases_matched"], serde_json::json!(true));

    let cases = payload["cases"].as_array().unwrap_or_else(|| panic!("cases array missing"));
    assert_eq!(cases.len(), 1);
    let case = &cases[0];
    assert_eq!(case["sample_id"], serde_json::json!("core-v1-duplicate-contigs"));
    assert_eq!(case["expectation_matched"], serde_json::json!(true));
    assert_eq!(case["total_reads"], serde_json::json!(3));
    assert_eq!(case["mapped_reads"], serde_json::json!(3));
    assert_eq!(case["unmapped_reads"], serde_json::json!(0));
    assert_eq!(case["duplicate_flagged_reads"], serde_json::json!(1));
    assert_eq!(
        case["contig_summary"],
        serde_json::json!([
            {
                "contig": "chr1",
                "length": 100,
                "mapped": 2,
                "unmapped": 0
            },
            {
                "contig": "chr2",
                "length": 80,
                "mapped": 1,
                "unmapped": 0
            }
        ])
    );
    assert_eq!(case["reference_mismatch"], serde_json::json!(false));

    let qc_pre_summary = repo_root.join(
        case["qc_pre_summary"].as_str().unwrap_or_else(|| panic!("qc_pre_summary path missing")),
    );
    let flagstat = repo_root
        .join(case["flagstat"].as_str().unwrap_or_else(|| panic!("flagstat path missing")));
    let idxstats = repo_root
        .join(case["idxstats"].as_str().unwrap_or_else(|| panic!("idxstats path missing")));
    let samtools_stats = repo_root.join(
        case["samtools_stats"].as_str().unwrap_or_else(|| panic!("samtools_stats path missing")),
    );
    let stage_metrics = repo_root.join(
        case["stage_metrics"].as_str().unwrap_or_else(|| panic!("stage_metrics path missing")),
    );
    assert!(qc_pre_summary.is_file(), "case qc_pre summary must exist");
    assert!(flagstat.is_file(), "case flagstat must exist");
    assert!(idxstats.is_file(), "case idxstats must exist");
    assert!(samtools_stats.is_file(), "case samtools stats must exist");
    assert!(stage_metrics.is_file(), "case stage metrics must exist");

    let case_summary: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&qc_pre_summary)?)?;
    assert_eq!(case_summary["stage_id"], serde_json::json!("bam.qc_pre"));
    assert_eq!(case_summary["total_reads"], serde_json::json!(3));
    assert_eq!(case_summary["duplicate_flagged_reads"], serde_json::json!(1));
    assert_eq!(
        case_summary["contig_summary"],
        serde_json::json!([
            {
                "contig": "chr1",
                "length": 100,
                "mapped": 2,
                "unmapped": 0
            },
            {
                "contig": "chr2",
                "length": 80,
                "mapped": 1,
                "unmapped": 0
            }
        ])
    );

    let stage_metrics_payload: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&stage_metrics)?)?;
    assert_eq!(stage_metrics_payload["stage_id"], serde_json::json!("bam.qc_pre"));
    assert_eq!(stage_metrics_payload["total_reads"], serde_json::json!(3));
    assert_eq!(stage_metrics_payload["mapped_reads"], serde_json::json!(3));
    assert_eq!(stage_metrics_payload["unmapped_reads"], serde_json::json!(0));
    assert_eq!(stage_metrics_payload["duplicate_flagged_reads"], serde_json::json!(1));
    assert_eq!(
        stage_metrics_payload["contig_summary"],
        serde_json::json!([
            {
                "contig": "chr1",
                "length": 100,
                "mapped": 2,
                "unmapped": 0
            },
            {
                "contig": "chr2",
                "length": 80,
                "mapped": 1,
                "unmapped": 0
            }
        ])
    );

    Ok(())
}
