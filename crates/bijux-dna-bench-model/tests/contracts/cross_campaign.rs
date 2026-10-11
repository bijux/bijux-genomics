use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    validate_cross_benchmark_campaign_report, AppraisalFinding, CrossBenchmarkCampaignReport,
    CrossBenchmarkCampaignSummary, CrossBenchmarkGoalEntry, CrossMatrixRowRef, CrossScenario,
    HardeningQueueEntry, CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};

#[test]
fn cross_campaign_validates_goal_closure_and_summary() {
    assert!(validate_cross_benchmark_campaign_report(&report()).is_ok());

    let mut drifted = report();
    drifted.summary.total_rows = 0;
    assert!(validate_cross_benchmark_campaign_report(&drifted)
        .is_err_and(|error| error.to_string().contains("summary drifted")));

    let mut mismatched = report();
    mismatched.entries[0].goal_id = "G999".to_string();
    assert!(validate_cross_benchmark_campaign_report(&mismatched)
        .is_err_and(|error| error.to_string().contains("selected goals")));
}

#[test]
fn cross_campaign_rejects_duplicate_nested_identities_and_unknown_statuses() {
    let mut duplicate_row = report();
    let row = duplicate_row.entries[0].matched_rows[0].clone();
    duplicate_row.entries[0].matched_rows.push(row);
    duplicate_row.summary.total_rows += 1;
    assert!(validate_cross_benchmark_campaign_report(&duplicate_row)
        .is_err_and(|error| error.to_string().contains("row reference")));

    let mut duplicate_scenario = report();
    let scenario = duplicate_scenario.entries[0].scenarios[0].clone();
    duplicate_scenario.entries[0].scenarios.push(scenario);
    assert!(validate_cross_benchmark_campaign_report(&duplicate_scenario)
        .is_err_and(|error| error.to_string().contains("scenario")));

    let mut unknown_status = report();
    unknown_status.entries[0].status = "completed".to_string();
    unknown_status.summary.status_counts = BTreeMap::from([("completed".to_string(), 1)]);
    assert!(validate_cross_benchmark_campaign_report(&unknown_status)
        .is_err_and(|error| error.to_string().contains("goal entry")));

    let mut evidence_mismatch = report();
    evidence_mismatch.entries[0].hardening_entries.clear();
    evidence_mismatch.summary.total_queue_entries = 0;
    assert!(validate_cross_benchmark_campaign_report(&evidence_mismatch)
        .is_err_and(|error| error.to_string().contains("does not match its evidence")));
}

#[test]
fn cross_campaign_schema_matches_model_and_version_fails_closed() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated =
        serde_json::to_value(bijux_dna_bench_model::cross_benchmark_campaign_report_schema())?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/cross-benchmark-campaign-report.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "cross-benchmark campaign schema drifted");

    let mut value = serde_json::to_value(report())?;
    value["schema_version"] = serde_json::json!("bijux.hpc.cross_benchmark_campaign.v2");
    assert!(serde_json::from_value::<CrossBenchmarkCampaignReport>(value).is_err());
    Ok(())
}

fn report() -> CrossBenchmarkCampaignReport {
    CrossBenchmarkCampaignReport {
        schema_version: CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1.to_string(),
        campaign_id: "campaign-cross".to_string(),
        domain: "cross".to_string(),
        selected_goals: vec!["G161".to_string()],
        summary: CrossBenchmarkCampaignSummary {
            total_goals: 1,
            total_rows: 1,
            total_findings: 1,
            total_queue_entries: 1,
            status_counts: BTreeMap::from([("requires-hardening".to_string(), 1)]),
        },
        entries: vec![CrossBenchmarkGoalEntry {
            goal_id: "G161".to_string(),
            title: "benchmark FASTQ-to-BAM modern workflow".to_string(),
            stage_ids: vec!["fastq.trim_reads=>bam.align".to_string()],
            scenarios: vec![CrossScenario {
                scenario_id: "modern-fastq-bam-validate-trim-align".to_string(),
                focus: "validate, trim, align, summary, and coverage handoff".to_string(),
            }],
            matched_rows: vec![CrossMatrixRowRef {
                row_id: "cross-fastq-trim-to-bam-align-cutadapt-bwa-mem2".to_string(),
                stage_id: "fastq.trim_reads=>bam.align".to_string(),
                tool_id: "cutadapt=>bwa-mem2".to_string(),
                readiness_class: "degraded".to_string(),
                repetitions: 5,
            }],
            appraisal_findings: vec![AppraisalFinding {
                appraiser_id: "scientific-validity".to_string(),
                row_id: "cross-fastq-trim-to-bam-align-cutadapt-bwa-mem2".to_string(),
                severity: "warning".to_string(),
                confidence: "high".to_string(),
                failure_class: "scientific-readiness-degraded".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "scientific readiness is degraded".to_string(),
                recommendation: "close the declared evidence gap".to_string(),
            }],
            hardening_entries: vec![HardeningQueueEntry {
                queue_id: "hardening-0007".to_string(),
                severity: "warning".to_string(),
                failure_class: "scientific-readiness-degraded".to_string(),
                recommendation: "close the declared evidence gap".to_string(),
                affected_rows: vec!["cross-fastq-trim-to-bam-align-cutadapt-bwa-mem2".to_string()],
                source_appraisers: vec!["scientific-validity".to_string()],
            }],
            goal_checks: vec!["modern_bridge_rows=1".to_string()],
            status: "requires-hardening".to_string(),
        }],
    }
}
