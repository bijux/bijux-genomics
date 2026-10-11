//! Owner: bijux-dna-bench-workflows
//! Reject contradictory smoke inputs before artifact publication.
use super::run_normalize_primers_stage;
use anyhow::{Context, Result};
use serde_json::json;

#[test]
fn rejects_paired_or_mate_bearing_single_end_smoke_before_copying() -> Result<()> {
    for (layout, mate) in [("paired_end", None), ("single_end", Some("mate.fastq.gz"))] {
        let repository = tempfile::tempdir()?;
        let source = repository
            .path()
            .join("artifacts/runs/bench/local-smoke/fastq.normalize_primers/report.json");
        std::fs::create_dir_all(source.parent().context("report parent")?)?;
        std::fs::write(
            &source,
            serde_json::to_vec(&json!({
                "schema_version": "bijux.fastq.normalize_primers.local_smoke.report.v2",
                "stage_id": "fastq.normalize_primers", "case_count": 1,
                "cases": [{
                    "sample_id": "amplicon-16s-se", "tool_id": "cutadapt", "layout": layout,
                    "primer_set_id": "16S_universal_v1", "marker_id": "16S",
                    "orientation_policy": "normalize_to_forward_primer",
                    "input_reads": 3, "matched_reads": 2, "unmatched_reads": 1, "output_reads": 3,
                    "normalized_reads_r1": "reads.fastq.gz", "normalized_reads_r2": mate,
                    "report_json": "report.json", "primer_orientation_report": "orientation.tsv",
                    "primer_stats_json": "stats.json", "used_fallback": false
                }]
            }))?,
        )?;
        let output = repository.path().join("workflow");
        let error = run_normalize_primers_stage(repository.path(), &output)
            .err()
            .context("contradictory layout was accepted")?;
        assert!(error.to_string().contains("requires single-ended smoke evidence"));
        assert!(!output.exists(), "input rejection must precede artifact copies");
    }
    Ok(())
}
