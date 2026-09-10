use anyhow::Result;
use bijux_dna_bench_model::{
    plan_hpc_submission, HpcSubmissionSelector as Selector, PlannedJob, PlannedOutputs,
    ResourceTemplate,
};

fn job(name: &str, stage: &str, tool: &str, sample: &str, dependencies: &[&str]) -> PlannedJob {
    PlannedJob {
        job_id: format!("id-{name}"),
        job_name: name.into(),
        stage: stage.into(),
        tool: tool.into(),
        sample: sample.into(),
        corpus_size_class: None,
        array_task: None,
        depends_on: dependencies.iter().map(|name| (*name).into()).collect(),
        resource_template: "small".into(),
        resources: ResourceTemplate {
            cpus: 1,
            mem_gb: 2,
            walltime: "00:10:00".into(),
            scratch_gb: 1,
        },
        outputs: PlannedOutputs {
            scratch_dir: "scratch".into(),
            log: "log".into(),
            out: "out".into(),
            err: "err".into(),
            results: "results".into(),
            code: "code".into(),
        },
    }
}

#[test]
fn hpc_submission_filters_tool_and_sample_together_and_borrows_current_jobs() -> Result<()> {
    let jobs = vec![
        job("a", "fastq.validate", "check", "one", &[]),
        job("b", "fastq.validate", "check", "two", &[]),
        job("c", "fastq.validate", "other", "two", &[]),
        job("d", "bam.align", "align", "two", &[]),
    ];
    for (tool, sample, expected) in [
        (None, None, vec!["a", "b", "c"]),
        (Some("check"), None, vec!["a", "b"]),
        (None, Some("two"), vec!["b", "c"]),
        (Some("check"), Some("two"), vec!["b"]),
    ] {
        let selector = Selector::Stage {
            stage: "fastq.validate".into(),
            tool: tool.map(str::to_string),
            sample: sample.map(str::to_string),
        };
        let plan = plan_hpc_submission(&jobs, &selector)?;
        assert_eq!(plan.jobs().iter().map(|job| job.name).collect::<Vec<_>>(), expected);
        for selected in plan.jobs() {
            assert!(jobs.iter().any(|original| std::ptr::eq(original, selected.planned)));
        }
    }
    assert!(plan_hpc_submission(
        &jobs,
        &Selector::Stage {
            stage: "fastq.validate".into(),
            tool: Some("absent".into()),
            sample: Some("two".into())
        }
    )
    .is_err());
    assert!(plan_hpc_submission(&jobs, &Selector::Cross { domains: vec!["fastq".into()] }).is_err());
    assert_eq!(
        plan_hpc_submission(
            &jobs,
            &Selector::Cross { domains: vec!["fastq".into(), "bam".into()] }
        )?
        .jobs()
        .len(),
        4
    );
    Ok(())
}

#[test]
fn hpc_submission_preserves_scoped_prerequisites_and_exact_ordered_receipts() -> Result<()> {
    let jobs = vec![
        job("first", "fastq.validate", "check", "one", &[]),
        job("second", "bam.align", "align", "one", &["first"]),
        job("third", "bam.sort", "sort", "one", &[]),
    ];
    let all = plan_hpc_submission(&jobs, &Selector::All)?;
    assert!(all.scheduler_dependencies(0, &[])?.is_empty());
    assert_eq!(all.scheduler_dependencies(1, &["17".into()])?, ["17"]);
    assert_eq!(all.scheduler_dependencies(2, &["17".into(), "29".into()])?, ["29"]);
    assert!(all.scheduler_dependencies(2, &["17".into()]).is_err());
    assert!(all.scheduler_dependencies(1, &[String::new()]).is_err());
    assert!(all.scheduler_dependencies(3, &["17".into(), "29".into(), "30".into()]).is_err());
    let scoped = plan_hpc_submission(&jobs, &Selector::Domain { domain: "bam".into() })?;
    assert_eq!(scoped.jobs()[0].depends_on_names, ["first"]);
    assert!(scoped.scheduler_dependencies(0, &[])?.is_empty());
    assert_eq!(scoped.scheduler_dependencies(1, &["41".into()])?, ["41"]);
    Ok(())
}

#[test]
fn hpc_submission_rejects_ambiguous_and_unresolved_selected_dependencies() {
    let first = job("first", "fastq.validate", "check", "one", &[]);
    let second = job("second", "bam.align", "align", "one", &["first"]);
    for invalid in [
        vec![first.clone(), first.clone()],
        vec![job("", "fastq.validate", "check", "one", &[])],
        vec![job("first", "fastq.validate", "check", "one", &["missing"])],
        vec![job("first", "fastq.validate", "check", "one", &["first"])],
        vec![second.clone(), first.clone()],
        vec![job("first", "fastq.validate", "check", "one", &["second"]), second],
        Vec::new(),
    ] {
        assert!(plan_hpc_submission(&invalid, &Selector::All).is_err());
    }
}
