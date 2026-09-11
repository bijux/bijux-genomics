use crate::readiness::assessment::fail_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::declarations::DEFAULT_SLURM_DEPENDENCY_CHECK_REPORT_PATH;
use crate::readiness::declarations::DEFAULT_SLURM_DRY_RUN_ROOT;
use crate::readiness::declarations::DEFAULT_SLURM_SCRIPT_BODY_REPORT_PATH;
use crate::readiness::declarations::DEFAULT_SLURM_SHELL_SYNTAX_REPORT_PATH;
use crate::readiness::declarations::DEFAULT_SLURM_SUBMIT_MANIFEST_PATH;
use crate::readiness::ports::ReadinessDomain as BenchLocalDomain;
use bijux_dna_bench_model::evaluate_benchmark_local_hpc_slurm_readiness;
use bijux_dna_bench_model::BenchmarkLocalHpcSlurmObservation;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use std::path::PathBuf;

pub(in crate::readiness) fn evaluate_slurm_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    evaluate_fastq_slurm_scripts(context, checks);

    let bam_slurm_report = render_bam_slurm_scripts(context, checks);

    match context.backend.validate_slurm_script_bodies(
        repo_root,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT),
        PathBuf::from(DEFAULT_SLURM_SCRIPT_BODY_REPORT_PATH),
    ) {
        Ok(report) => checks.push(evaluate_benchmark_local_hpc_slurm_readiness(
            BenchmarkLocalHpcSlurmObservation::ScriptBodies(&report),
        )),
        Err(err) => checks.push(fail_check(
            94,
            "slurm_dry_run",
            "generated SLURM script bodies call real repo commands and contain no placeholders",
            Some(DEFAULT_SLURM_SCRIPT_BODY_REPORT_PATH.to_string()),
            format!("{err:#}"),
        )),
    }

    match context.backend.validate_slurm_shell_syntax(
        repo_root,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT),
        PathBuf::from(DEFAULT_SLURM_SHELL_SYNTAX_REPORT_PATH),
    ) {
        Ok(report) => checks.push(evaluate_benchmark_local_hpc_slurm_readiness(
            BenchmarkLocalHpcSlurmObservation::ShellSyntax(&report),
        )),
        Err(err) => checks.push(fail_check(
            95,
            "slurm_dry_run",
            "generated SLURM scripts pass bash syntax validation",
            Some(DEFAULT_SLURM_SHELL_SYNTAX_REPORT_PATH.to_string()),
            format!("{err:#}"),
        )),
    }

    let submit_manifest = match context.backend.render_slurm_submit_manifest(
        repo_root,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT),
        PathBuf::from(DEFAULT_SLURM_SUBMIT_MANIFEST_PATH),
    ) {
        Ok(report) => {
            checks.push(evaluate_benchmark_local_hpc_slurm_readiness(
                BenchmarkLocalHpcSlurmObservation::SubmitManifest(&report),
            ));
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                96,
                "slurm_dry_run",
                "SLURM dry-run submit manifest records complete job metadata",
                Some(DEFAULT_SLURM_SUBMIT_MANIFEST_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    };

    match context.backend.validate_slurm_dependencies(
        repo_root,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT),
        PathBuf::from(DEFAULT_SLURM_SUBMIT_MANIFEST_PATH),
        PathBuf::from(DEFAULT_SLURM_DEPENDENCY_CHECK_REPORT_PATH),
    ) {
        Ok(report) => checks.push(evaluate_benchmark_local_hpc_slurm_readiness(
            BenchmarkLocalHpcSlurmObservation::Dependencies(&report),
        )),
        Err(err) => checks.push(fail_check(
            97,
            "slurm_dry_run",
            "SLURM dependencies stay in exactly one source of truth",
            Some(DEFAULT_SLURM_DEPENDENCY_CHECK_REPORT_PATH.to_string()),
            format!("{err:#}"),
        )),
    }

    checks.push(evaluate_benchmark_local_hpc_slurm_readiness(
        BenchmarkLocalHpcSlurmObservation::RunPaths {
            bam: bam_slurm_report.as_ref(),
            submission: submit_manifest.as_ref(),
        },
    ));
}

pub(super) fn evaluate_fastq_slurm_scripts(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    match context.backend.render_local_slurm_scripts(
        repo_root,
        BenchLocalDomain::Fastq,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT).join("fastq"),
    ) {
        Ok(report) => checks.push(evaluate_benchmark_local_hpc_slurm_readiness(
            BenchmarkLocalHpcSlurmObservation::FastqScripts(&report),
        )),
        Err(err) => checks.push(fail_check(
            92,
            "slurm_dry_run",
            "FASTQ local benchmark stages render governed SLURM dry-run scripts",
            Some("artifacts/runs/bench/slurm-dry-run/fastq".to_string()),
            format!("{err:#}"),
        )),
    }
}

fn render_bam_slurm_scripts(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<bijux_dna_bench_model::BenchmarkLocalSlurmDryRunReport> {
    let repo_root = context.repo_root;
    match context.backend.render_local_slurm_scripts(
        repo_root,
        BenchLocalDomain::Bam,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT).join("bam"),
    ) {
        Ok(report) => {
            checks.push(evaluate_benchmark_local_hpc_slurm_readiness(
                BenchmarkLocalHpcSlurmObservation::BamScripts(&report),
            ));
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                93,
                "slurm_dry_run",
                "BAM local benchmark stages render governed SLURM dry-run scripts",
                Some("artifacts/runs/bench/slurm-dry-run/bam".to_string()),
                format!("{err:#}"),
            ));
            None
        }
    }
}
