//! Owner: benchmark model local HPC readiness policy.
use crate::{
    BenchmarkLocalHpcSubmissionReadinessGoalCheck as GoalCheck,
    BenchmarkLocalSlurmDependencyCheckReport, BenchmarkLocalSlurmDryRunReport,
    BenchmarkLocalSlurmScriptBodyReport, BenchmarkLocalSlurmShellSyntaxReport,
    BenchmarkLocalSlurmSubmitManifest,
};

/// Borrow exact observations from the local submission-readiness workflow.
/// These observations describe generated scripts and their checks, not scheduler execution.
#[derive(Debug, Clone, Copy)]
pub enum BenchmarkLocalHpcSlurmObservation<'a> {
    FastqScripts(&'a BenchmarkLocalSlurmDryRunReport),
    BamScripts(&'a BenchmarkLocalSlurmDryRunReport),
    ScriptBodies(&'a BenchmarkLocalSlurmScriptBodyReport),
    ShellSyntax(&'a BenchmarkLocalSlurmShellSyntaxReport),
    SubmitManifest(&'a BenchmarkLocalSlurmSubmitManifest),
    Dependencies(&'a BenchmarkLocalSlurmDependencyCheckReport),
    RunPaths {
        bam: Option<&'a BenchmarkLocalSlurmDryRunReport>,
        submission: Option<&'a BenchmarkLocalSlurmSubmitManifest>,
    },
}

impl BenchmarkLocalHpcSlurmObservation<'_> {
    fn admit(self) -> anyhow::Result<()> {
        match self {
            Self::FastqScripts(report) => {
                crate::validate_benchmark_local_slurm_dry_run(report)?;
                anyhow::ensure!(
                    report.domain == "fastq",
                    "FASTQ readiness received another domain"
                );
            }
            Self::BamScripts(report) => {
                crate::validate_benchmark_local_slurm_dry_run(report)?;
                anyhow::ensure!(report.domain == "bam", "BAM readiness received another domain");
            }
            Self::ScriptBodies(report) => {
                crate::validate_benchmark_local_slurm_script_body_report(report)?;
            }
            Self::ShellSyntax(report) => {
                crate::validate_benchmark_local_slurm_shell_syntax(report)?;
            }
            Self::SubmitManifest(report) => {
                crate::validate_benchmark_local_slurm_submit_manifest(report)?;
            }
            Self::Dependencies(report) => {
                crate::validate_benchmark_local_slurm_dependency_check(report)?;
            }
            Self::RunPaths { bam, submission } => {
                if let Some(report) = bam {
                    Self::BamScripts(report).admit()?;
                }
                if let Some(report) = submission {
                    Self::SubmitManifest(report).admit()?;
                }
            }
        }
        Ok(())
    }
}

/// Evaluate a governed local Slurm readiness observation after admitting its complete report.
/// Malformed observations become failed checks with the same stable goal/category identity.
#[must_use]
pub fn evaluate_benchmark_local_hpc_slurm_readiness(
    observation: BenchmarkLocalHpcSlurmObservation<'_>,
) -> GoalCheck {
    let admission = observation.admit();
    let mut check = match observation {
        BenchmarkLocalHpcSlurmObservation::FastqScripts(report) => {
            evaluate_fastq_slurm_goal(report)
        }
        BenchmarkLocalHpcSlurmObservation::BamScripts(report) => evaluate_bam_slurm_goal(report),
        BenchmarkLocalHpcSlurmObservation::ScriptBodies(report) => {
            evaluate_slurm_script_body_goal(report)
        }
        BenchmarkLocalHpcSlurmObservation::ShellSyntax(report) => {
            evaluate_slurm_shell_syntax_goal(report)
        }
        BenchmarkLocalHpcSlurmObservation::SubmitManifest(report) => {
            evaluate_slurm_submit_manifest_goal(report)
        }
        BenchmarkLocalHpcSlurmObservation::Dependencies(report) => {
            evaluate_slurm_dependency_goal(report)
        }
        BenchmarkLocalHpcSlurmObservation::RunPaths { bam, submission } => {
            evaluate_slurm_run_path_goal(bam, submission)
        }
    };
    if let Err(error) = admission {
        check.ok = false;
        check.detail = format!("invalid local Slurm observation: {error:#}");
    }
    check
}

fn evaluate_fastq_slurm_goal(report: &BenchmarkLocalSlurmDryRunReport) -> GoalCheck {
    if report.script_count == 27 {
        ok_check(
            92,
            "slurm_dry_run",
            "FASTQ local benchmark stages render governed SLURM dry-run scripts",
            Some(report.output_root.clone()),
            format!("FASTQ SLURM dry-run rendered {} scripts", report.script_count),
        )
    } else {
        fail_check(
            92,
            "slurm_dry_run",
            "FASTQ local benchmark stages render governed SLURM dry-run scripts",
            Some(report.output_root.clone()),
            format!("expected 27 FASTQ scripts but found {}", report.script_count),
        )
    }
}

fn evaluate_bam_slurm_goal(report: &BenchmarkLocalSlurmDryRunReport) -> GoalCheck {
    if report.script_count == 24 {
        ok_check(
            93,
            "slurm_dry_run",
            "BAM local benchmark stages render governed SLURM dry-run scripts",
            Some(report.output_root.clone()),
            format!("BAM SLURM dry-run rendered {} scripts", report.script_count),
        )
    } else {
        fail_check(
            93,
            "slurm_dry_run",
            "BAM local benchmark stages render governed SLURM dry-run scripts",
            Some(report.output_root.clone()),
            format!("expected 24 BAM scripts but found {}", report.script_count),
        )
    }
}

fn evaluate_slurm_script_body_goal(report: &BenchmarkLocalSlurmScriptBodyReport) -> GoalCheck {
    if report.ok {
        ok_check(
            94,
            "slurm_dry_run",
            "generated SLURM script bodies call real repo commands and contain no placeholders",
            Some(report.report_path.clone()),
            format!("SLURM script body validation passed across {} scripts", report.script_count),
        )
    } else {
        fail_check(
            94,
            "slurm_dry_run",
            "generated SLURM script bodies call real repo commands and contain no placeholders",
            Some(report.report_path.clone()),
            format!(
                "ok={} script_count={} findings_count={}",
                report.ok, report.script_count, report.findings_count
            ),
        )
    }
}

fn evaluate_slurm_shell_syntax_goal(report: &BenchmarkLocalSlurmShellSyntaxReport) -> GoalCheck {
    if report.ok {
        ok_check(
            95,
            "slurm_dry_run",
            "generated SLURM scripts pass bash syntax validation",
            Some(report.report_path.clone()),
            format!("SLURM shell syntax validation passed across {} scripts", report.script_count),
        )
    } else {
        fail_check(
            95,
            "slurm_dry_run",
            "generated SLURM scripts pass bash syntax validation",
            Some(report.report_path.clone()),
            format!(
                "ok={} script_count={} findings_count={}",
                report.ok, report.script_count, report.findings_count
            ),
        )
    }
}

fn evaluate_slurm_submit_manifest_goal(report: &BenchmarkLocalSlurmSubmitManifest) -> GoalCheck {
    let required_fields_are_present = report.jobs.iter().all(|job| {
        !job.job_name.trim().is_empty()
            && !job.domain.trim().is_empty()
            && !job.tool_id.trim().is_empty()
            && !job.script_path.trim().is_empty()
            && !job.logs.stdout_path.trim().is_empty()
            && !job.logs.stderr_path.trim().is_empty()
    });
    if report.job_count == 51 && required_fields_are_present {
        ok_check(
            96,
            "slurm_dry_run",
            "SLURM dry-run submit manifest records complete job metadata",
            Some(report.manifest_path.clone()),
            format!("submit manifest records {} jobs", report.job_count),
        )
    } else {
        fail_check(
            96,
            "slurm_dry_run",
            "SLURM dry-run submit manifest records complete job metadata",
            Some(report.manifest_path.clone()),
            format!(
                "job_count={} required_fields_are_present={required_fields_are_present}",
                report.job_count
            ),
        )
    }
}

fn evaluate_slurm_dependency_goal(report: &BenchmarkLocalSlurmDependencyCheckReport) -> GoalCheck {
    if report.ok && report.findings_count == 0 {
        ok_check(
            97,
            "slurm_dry_run",
            "SLURM dependencies stay in exactly one source of truth",
            Some(report.report_path.clone()),
            format!("dependency ownership is clean across {} dry-run jobs", report.job_count),
        )
    } else {
        fail_check(
            97,
            "slurm_dry_run",
            "SLURM dependencies stay in exactly one source of truth",
            Some(report.report_path.clone()),
            format!(
                "ok={} findings_count={} job_count={}",
                report.ok, report.findings_count, report.job_count
            ),
        )
    }
}

fn evaluate_slurm_run_path_goal(
    bam_slurm_report: Option<&BenchmarkLocalSlurmDryRunReport>,
    submit_manifest: Option<&BenchmarkLocalSlurmSubmitManifest>,
) -> GoalCheck {
    let Some(bam_slurm_report) = bam_slurm_report else {
        return fail_check(
            98,
            "slurm_dry_run",
            "SLURM dry-run scripts resolve predictable log and result paths",
            Some("artifacts/runs/bench/slurm-dry-run/runs/local-benchmark-dry-run".to_string()),
            "BAM SLURM script rendering failed earlier".to_string(),
        );
    };
    let Some(submit_manifest) = submit_manifest else {
        return fail_check(
            98,
            "slurm_dry_run",
            "SLURM dry-run scripts resolve predictable log and result paths",
            Some("artifacts/runs/bench/slurm-dry-run/runs/local-benchmark-dry-run".to_string()),
            "SLURM submit manifest rendering failed earlier".to_string(),
        );
    };

    let path_fields_are_present = bam_slurm_report.scripts.iter().all(|script| {
        script.stdout_path.contains("local-benchmark-dry-run")
            && script.stderr_path.contains("local-benchmark-dry-run")
            && script.result_root.contains("local-benchmark-dry-run")
            && script.stage_result_manifest_path.contains("local-benchmark-dry-run")
    }) && submit_manifest.jobs.iter().all(|job| {
        job.logs.stdout_path.contains("local-benchmark-dry-run")
            && job.logs.stderr_path.contains("local-benchmark-dry-run")
            && job.result_root.contains("local-benchmark-dry-run")
    });
    if path_fields_are_present {
        ok_check(
            98,
            "slurm_dry_run",
            "SLURM dry-run scripts resolve predictable log and result paths",
            Some("artifacts/runs/bench/slurm-dry-run/runs/local-benchmark-dry-run".to_string()),
            "SLURM dry-run scripts and submit manifest both carry governed run-path fields"
                .to_string(),
        )
    } else {
        fail_check(
            98,
            "slurm_dry_run",
            "SLURM dry-run scripts resolve predictable log and result paths",
            Some("artifacts/runs/bench/slurm-dry-run/runs/local-benchmark-dry-run".to_string()),
            "one or more generated SLURM path fields are missing the governed run-id layout"
                .to_string(),
        )
    }
}

fn ok_check(
    goal_id: u32,
    category: &str,
    surface: impl Into<String>,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> GoalCheck {
    GoalCheck {
        goal_id,
        category: category.to_string(),
        surface: surface.into(),
        output_path,
        ok: true,
        detail: detail.into(),
    }
}

fn fail_check(
    goal_id: u32,
    category: &str,
    surface: impl Into<String>,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> GoalCheck {
    GoalCheck {
        goal_id,
        category: category.to_string(),
        surface: surface.into(),
        output_path,
        ok: false,
        detail: detail.into(),
    }
}
