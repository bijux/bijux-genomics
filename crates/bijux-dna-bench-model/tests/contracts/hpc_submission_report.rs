use anyhow::{Context, Result};
use bijux_dna_bench_model::{validate_hpc_submission_report, SlurmSubmissionReport};
use serde_json::{json, Value};

fn receipt() -> Value {
    let job = json!({
        "job_name":"trim", "stage":"fastq.trim", "tool":"fastp", "sample":"sample",
        "array_task":null, "planned_job_id":"planned-trim", "scheduler_job_id":"mock-0001",
        "dependency_scheduler_ids":[], "script_path":"scripts/trim.sh",
        "log_path":"logs/trim.log", "out_path":"logs/trim.out", "err_path":"logs/trim.err",
        "results_path":"results/trim.age", "code_path":"code/trim.age"
    });
    let mut dependent = job.clone();
    dependent["job_name"] = json!("qc");
    dependent["planned_job_id"] = json!("planned-qc");
    dependent["scheduler_job_id"] = json!("mock-0002");
    dependent["dependency_scheduler_ids"] = json!(["mock-0001"]);
    json!({"schema_version":"bijux.hpc.slurm.submission.v1", "mode":"mock",
        "campaign_id":"campaign", "domain":"fastq", "submitted_at":"1750000000",
        "jobs":[job, dependent]})
}

#[test]
fn submission_report_preserves_wire_and_admits_ordered_receipts() -> Result<()> {
    let value = receipt();
    let report: SlurmSubmissionReport =
        serde_json::from_value(value.clone()).context("decode valid mock receipt")?;
    validate_hpc_submission_report(&report).context("admit ordered receipts")?;
    assert_eq!(serde_json::to_value(report).context("serialize receipt")?, value);
    let mut real = value;
    real["mode"] = json!("real");
    real["jobs"][0]["scheduler_job_id"] = json!("1001");
    real["jobs"][1]["scheduler_job_id"] = json!("1002");
    real["jobs"][1]["dependency_scheduler_ids"] = json!(["1001"]);
    validate_hpc_submission_report(&serde_json::from_value(real).context("decode real receipt")?)
        .context("admit real receipts")?;
    Ok(())
}

#[test]
fn submission_report_rejects_foreign_wire_and_invalid_receipt_closure() -> Result<()> {
    for (pointer, replacement) in
        [("/schema_version", json!("bijux.hpc.slurm.submission.v9")), ("/mode", json!("complete"))]
    {
        let mut value = receipt();
        *value.pointer_mut(pointer).context("existing mutation field")? = replacement;
        assert!(serde_json::from_value::<SlurmSubmissionReport>(value).is_err());
    }
    for pointer in ["", "/jobs/0"] {
        let mut value = receipt();
        value
            .pointer_mut(pointer)
            .context("existing receipt object")?
            .as_object_mut()
            .context("existing receipt object")?
            .insert("extra".into(), json!(true));
        assert!(serde_json::from_value::<SlurmSubmissionReport>(value).is_err());
    }
    for (pointer, replacement) in [
        ("/jobs", json!([])),
        ("/campaign_id", json!(" ")),
        ("/jobs/0/script_path", json!("")),
        ("/jobs/1/job_name", json!("trim")),
        ("/jobs/1/planned_job_id", json!("planned-trim")),
        ("/jobs/1/scheduler_job_id", json!("mock-0001")),
        ("/jobs/1/scheduler_job_id", json!("123")),
        ("/jobs/0/dependency_scheduler_ids", json!(["mock-0002"])),
        ("/jobs/0/dependency_scheduler_ids", json!(["mock-0001"])),
        ("/jobs/1/dependency_scheduler_ids", json!(["unknown"])),
        ("/jobs/1/dependency_scheduler_ids", json!(["mock-0001", "mock-0001"])),
    ] {
        let mut value = receipt();
        *value.pointer_mut(pointer).context("existing mutation field")? = replacement;
        let report = serde_json::from_value(value).context("decode semantic mutation")?;
        assert!(validate_hpc_submission_report(&report).is_err(), "accepted mutation at {pointer}");
    }
    Ok(())
}

#[test]
fn submission_request_and_backend_receipts_reject_invalid_identity_before_publication() -> Result<()>
{
    use bijux_dna_bench_model::{
        plan_hpc_submission, validate_hpc_scheduler_receipt, validate_hpc_submission_request,
        HpcSubmissionMode, HpcSubmissionSelector,
    };
    let mut report = crate::hpc_campaign_fixture::campaign();
    report.planned_jobs[0].depends_on.clear();
    let plan = plan_hpc_submission(&report.planned_jobs, &HpcSubmissionSelector::All)?;
    validate_hpc_submission_request(&report, &plan, "1700000000")?;
    assert!(validate_hpc_submission_request(&report, &plan, " ").is_err());
    drop(plan);
    report.planned_jobs[0].outputs.log.clear();
    let plan = plan_hpc_submission(&report.planned_jobs, &HpcSubmissionSelector::All)?;
    assert!(validate_hpc_submission_request(&report, &plan, "1700000000").is_err());
    drop(plan);
    let mut report = crate::hpc_campaign_fixture::campaign();
    report.planned_jobs[0].depends_on.clear();
    let mut repeated = report.planned_jobs[0].clone();
    repeated.job_name = "distinct-name".into();
    report.planned_jobs.push(repeated);
    let plan = plan_hpc_submission(&report.planned_jobs, &HpcSubmissionSelector::All)?;
    assert!(validate_hpc_submission_request(&report, &plan, "1700000000").is_err());
    validate_hpc_scheduler_receipt(HpcSubmissionMode::Real, "1234", &[])?;
    validate_hpc_scheduler_receipt(HpcSubmissionMode::Mock, "mock-0001", &[])?;
    for (mode, identity, preceding) in [
        (HpcSubmissionMode::Real, "1234", vec!["1234".into()]),
        (HpcSubmissionMode::Real, "mock-0001", vec![]),
        (HpcSubmissionMode::Mock, "1234", vec![]),
        (HpcSubmissionMode::Real, "1234;foreign", vec![]),
        (HpcSubmissionMode::Real, "", vec![]),
    ] {
        assert!(validate_hpc_scheduler_receipt(mode, identity, &preceding).is_err());
    }
    Ok(())
}
