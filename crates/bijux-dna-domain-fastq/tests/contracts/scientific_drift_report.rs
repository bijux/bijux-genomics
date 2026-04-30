use std::collections::BTreeMap;

use anyhow::Result;
use insta::Settings;

#[test]
fn scientific_drift_report_snapshot_stays_stable() -> Result<()> {
    let baseline = bijux_dna_domain_fastq::ScientificDriftSnapshotV1 {
        label: "baseline".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "fastp".to_string(),
        backend_version: Some("0.23.4".to_string()),
        defaults_fingerprint: Some("defaults-a".to_string()),
        metrics: BTreeMap::from([
            ("read_retention".to_string(), 0.91),
            ("mean_q".to_string(), 31.0),
            ("adapter_content_mean".to_string(), 0.08),
        ]),
        artifacts: BTreeMap::from([
            ("report_json".to_string(), "sha256:a".to_string()),
            ("trimmed_reads_r1".to_string(), "sha256:r1a".to_string()),
        ]),
        caveats: vec!["adapter-heavy samples bias retention deltas".to_string()],
    };
    let candidate = bijux_dna_domain_fastq::ScientificDriftSnapshotV1 {
        label: "candidate".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "cutadapt".to_string(),
        backend_version: Some("4.9".to_string()),
        defaults_fingerprint: Some("defaults-b".to_string()),
        metrics: BTreeMap::from([
            ("read_retention".to_string(), 0.88),
            ("mean_q".to_string(), 32.4),
            ("adapter_content_mean".to_string(), 0.01),
        ]),
        artifacts: BTreeMap::from([
            ("report_json".to_string(), "sha256:b".to_string()),
            ("trimmed_reads_r1".to_string(), "sha256:r1b".to_string()),
        ]),
        caveats: vec!["different adapter heuristics can reshape downstream QC".to_string()],
    };

    let report = bijux_dna_domain_fastq::build_fastq_scientific_drift_report(&baseline, &candidate);
    let mut settings = Settings::clone_current();
    settings.set_sort_maps(true);
    settings.bind(|| {
        insta::assert_json_snapshot!("fastq_scientific_drift_report", report);
    });
    Ok(())
}
