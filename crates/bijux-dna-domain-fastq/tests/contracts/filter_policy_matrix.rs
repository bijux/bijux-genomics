use anyhow::Result;
use std::path::{Path, PathBuf};

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow::anyhow!("workspace root"))
}

#[test]
fn governed_filter_policy_matrix_covers_quality_complexity_polyg_and_damage() {
    let matrix = bijux_dna_domain_fastq::governed_filter_policy_matrix();
    let stage_ids = matrix.iter().map(|entry| entry.stage_id.as_str()).collect::<Vec<_>>();
    assert_eq!(
        stage_ids,
        vec![
            "fastq.filter_reads",
            "fastq.filter_low_complexity",
            "fastq.trim_polyg_tails",
            "fastq.trim_terminal_damage",
        ]
    );
    assert!(
        matrix.iter().all(|entry| !entry.changed_metrics.is_empty()),
        "every governed filter entry must publish changed metrics"
    );
    assert!(
        matrix.iter().all(|entry| !entry.scientific_caveats.is_empty()),
        "every governed filter entry must publish scientific caveats"
    );
}

#[test]
fn filter_policy_docs_track_governed_stage_ids() -> Result<()> {
    let docs = std::fs::read_to_string(
        workspace_root()?.join("domain/fastq/docs/FILTER_POLICY_MATRIX.md"),
    )?;
    for stage_id in [
        "fastq.filter_reads",
        "fastq.filter_low_complexity",
        "fastq.trim_polyg_tails",
        "fastq.trim_terminal_damage",
    ] {
        assert!(docs.contains(stage_id), "FILTER_POLICY_MATRIX.md must mention {stage_id}");
    }
    Ok(())
}
