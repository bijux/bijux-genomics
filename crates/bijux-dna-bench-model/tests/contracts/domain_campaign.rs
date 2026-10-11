use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    validate_bam_benchmark_campaign_report, validate_fastq_benchmark_campaign_report,
    validate_hardening_benchmark_campaign_report, validate_vcf_benchmark_campaign_report,
    BamBenchmarkCampaignReport, DomainBenchmarkCampaignSummary, DomainBenchmarkGoalEntry,
    DomainBenchmarkMatrixRowRef, DomainBenchmarkScenario, FastqBenchmarkCampaignReport,
    HardeningBenchmarkCampaignReport, HardeningQueueEntry, VcfBenchmarkCampaignReport,
    BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1, FASTQ_BENCHMARK_CAMPAIGN_SCHEMA_V1,
    HARDENING_BENCHMARK_CAMPAIGN_SCHEMA_V1, VCF_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};

#[test]
fn domain_campaigns_validate_identity_goal_closure_and_summary() {
    assert!(validate_fastq_benchmark_campaign_report(&fastq_report()).is_ok());
    assert!(validate_bam_benchmark_campaign_report(&bam_report()).is_ok());
    assert!(validate_vcf_benchmark_campaign_report(&vcf_report()).is_ok());
    assert!(validate_hardening_benchmark_campaign_report(&hardening_report()).is_ok());

    let mut drifted = fastq_report();
    drifted.summary.total_rows = 0;
    assert!(validate_fastq_benchmark_campaign_report(&drifted)
        .is_err_and(|error| error.to_string().contains("summary drifted")));

    let mut wrong_domain = bam_report();
    wrong_domain.domain = "vcf".to_string();
    assert!(validate_bam_benchmark_campaign_report(&wrong_domain)
        .is_err_and(|error| error.to_string().contains("identity is invalid")));
}

#[test]
fn domain_campaigns_reject_duplicate_nested_identity_and_status_drift() {
    let mut duplicate = vcf_report();
    let scenario = duplicate.entries[0].scenarios[0].clone();
    duplicate.entries[0].scenarios.push(scenario);
    assert!(validate_vcf_benchmark_campaign_report(&duplicate)
        .is_err_and(|error| error.to_string().contains("scenario is invalid")));

    let mut status_drift = hardening_report();
    status_drift.entries[0].status = "requires-hardening".to_string();
    status_drift.summary.status_counts = BTreeMap::from([("requires-hardening".to_string(), 1)]);
    assert!(validate_hardening_benchmark_campaign_report(&status_drift)
        .is_err_and(|error| error.to_string().contains("does not match its evidence")));
}

#[test]
fn domain_campaigns_reject_nonexecuting_rows_and_foreign_queue_references() {
    let mut nonexecuting = fastq_report();
    nonexecuting.entries[0].matched_rows[0].repetitions = 0;
    assert!(validate_fastq_benchmark_campaign_report(&nonexecuting)
        .is_err_and(|error| error.to_string().contains("matrix row reference is invalid")));

    let mut foreign_reference = hardening_report();
    let admitted_row = foreign_reference.entries[0].matched_rows[0].row_id.clone();
    foreign_reference.entries[0].hardening_entries.push(HardeningQueueEntry {
        queue_id: "queue-foreign-reference".to_string(),
        severity: "warning".to_string(),
        failure_class: "downstream_contract".to_string(),
        recommendation: "admit every referenced row before publication".to_string(),
        affected_rows: vec![admitted_row, "foreign-row".to_string()],
        source_appraisers: vec!["downstream-compatibility".to_string()],
    });
    foreign_reference.entries[0].status = "requires-hardening".to_string();
    foreign_reference.summary.total_queue_entries = 1;
    foreign_reference.summary.status_counts =
        BTreeMap::from([("requires-hardening".to_string(), 1)]);
    assert!(validate_hardening_benchmark_campaign_report(&foreign_reference)
        .is_err_and(|error| error.to_string().contains("hardening entry is invalid")));
}

#[test]
fn domain_campaigns_preserve_refused_rows_without_execution_repetitions() {
    let mut refused = fastq_report();
    refused.entries[0].matched_rows[0].readiness_class = "refuse".to_string();
    refused.entries[0].matched_rows[0].repetitions = 0;
    refused.entries[0].status = "requires-hardening".to_string();
    refused.summary.status_counts = BTreeMap::from([("requires-hardening".to_string(), 1)]);
    assert!(validate_fastq_benchmark_campaign_report(&refused).is_ok());

    refused.entries[0].matched_rows[0].repetitions = 1;
    assert!(validate_fastq_benchmark_campaign_report(&refused)
        .is_err_and(|error| error.to_string().contains("matrix row reference is invalid")));
}

#[test]
fn domain_campaign_schemas_match_models_and_versions_fail_closed() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        (
            "fastq-benchmark-campaign-report.schema.json",
            serde_json::to_value(bijux_dna_bench_model::fastq_benchmark_campaign_report_schema())?,
        ),
        (
            "bam-benchmark-campaign-report.schema.json",
            serde_json::to_value(bijux_dna_bench_model::bam_benchmark_campaign_report_schema())?,
        ),
        (
            "vcf-benchmark-campaign-report.schema.json",
            serde_json::to_value(bijux_dna_bench_model::vcf_benchmark_campaign_report_schema())?,
        ),
        (
            "hardening-benchmark-campaign-report.schema.json",
            serde_json::to_value(
                bijux_dna_bench_model::hardening_benchmark_campaign_report_schema(),
            )?,
        ),
    ];
    for (name, generated) in schemas {
        let path = repository_root.join("benchmarks/contracts/schemas").join(name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "{name} drifted");
    }

    let mut fastq = serde_json::to_value(fastq_report())?;
    fastq["schema_version"] = serde_json::json!("bijux.hpc.fastq_benchmark_campaign.v2");
    assert!(serde_json::from_value::<FastqBenchmarkCampaignReport>(fastq).is_err());

    let mut bam = serde_json::to_value(bam_report())?;
    bam["schema_version"] = serde_json::json!("bijux.hpc.bam_benchmark_campaign.v2");
    assert!(serde_json::from_value::<BamBenchmarkCampaignReport>(bam).is_err());

    let mut vcf = serde_json::to_value(vcf_report())?;
    vcf["schema_version"] = serde_json::json!("bijux.hpc.vcf_benchmark_campaign.v2");
    assert!(serde_json::from_value::<VcfBenchmarkCampaignReport>(vcf).is_err());

    let mut hardening = serde_json::to_value(hardening_report())?;
    hardening["schema_version"] = serde_json::json!("bijux.hpc.hardening_benchmark_campaign.v2");
    assert!(serde_json::from_value::<HardeningBenchmarkCampaignReport>(hardening).is_err());
    Ok(())
}

fn fastq_report() -> FastqBenchmarkCampaignReport {
    FastqBenchmarkCampaignReport {
        schema_version: FASTQ_BENCHMARK_CAMPAIGN_SCHEMA_V1.to_string(),
        campaign_id: "campaign-fastq".to_string(),
        domain: "fastq".to_string(),
        selected_goals: vec!["G101".to_string()],
        summary: ready_summary(),
        entries: vec![ready_entry("G101", "fastq.validate_reads")],
    }
}

fn bam_report() -> BamBenchmarkCampaignReport {
    BamBenchmarkCampaignReport {
        schema_version: BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1.to_string(),
        campaign_id: "campaign-bam".to_string(),
        domain: "bam".to_string(),
        selected_goals: vec!["G121".to_string()],
        summary: ready_summary(),
        entries: vec![ready_entry("G121", "bam.validate")],
    }
}

fn vcf_report() -> VcfBenchmarkCampaignReport {
    VcfBenchmarkCampaignReport {
        schema_version: VCF_BENCHMARK_CAMPAIGN_SCHEMA_V1.to_string(),
        campaign_id: "campaign-vcf".to_string(),
        domain: "vcf".to_string(),
        selected_goals: vec!["G141".to_string()],
        summary: ready_summary(),
        entries: vec![ready_entry("G141", "vcf.validate")],
    }
}

fn hardening_report() -> HardeningBenchmarkCampaignReport {
    HardeningBenchmarkCampaignReport {
        schema_version: HARDENING_BENCHMARK_CAMPAIGN_SCHEMA_V1.to_string(),
        campaign_id: "campaign-all".to_string(),
        domain: "all".to_string(),
        selected_goals: vec!["G171".to_string()],
        summary: ready_summary(),
        entries: vec![ready_entry("G171", "fastq.validate_reads")],
    }
}

fn ready_summary() -> DomainBenchmarkCampaignSummary {
    DomainBenchmarkCampaignSummary {
        total_goals: 1,
        total_rows: 1,
        total_findings: 0,
        total_queue_entries: 0,
        status_counts: BTreeMap::from([("ready-for-benchmark-run".to_string(), 1)]),
    }
}

fn ready_entry(goal_id: &str, stage_id: &str) -> DomainBenchmarkGoalEntry {
    DomainBenchmarkGoalEntry {
        goal_id: goal_id.to_string(),
        title: format!("benchmark {stage_id}"),
        stage_ids: vec![stage_id.to_string()],
        scenarios: vec![DomainBenchmarkScenario {
            scenario_id: format!("{goal_id}-baseline").to_ascii_lowercase(),
            focus: "admitted baseline behavior".to_string(),
        }],
        matched_rows: vec![DomainBenchmarkMatrixRowRef {
            row_id: format!("{goal_id}-{stage_id}-tool").to_ascii_lowercase(),
            stage_id: stage_id.to_string(),
            tool_id: "tool-v1".to_string(),
            readiness_class: "ready".to_string(),
            repetitions: 3,
        }],
        appraisal_findings: Vec::new(),
        hardening_entries: Vec::new(),
        goal_checks: vec!["binding_ready=true".to_string()],
        status: "ready-for-benchmark-run".to_string(),
    }
}
