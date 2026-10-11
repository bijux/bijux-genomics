use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_bam_proofs::retained_tools as owner;
#[cfg(all(test, feature = "bam_downstream"))]
use owner::BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION;
pub(crate) use owner::{
    BamAllRetainedToolsCompleteReport, DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH,
};
use std::env;
use std::path::{Path, PathBuf};
pub(crate) fn run_render_bam_all_retained_tools_complete(
    args: &parse::BenchReadinessRenderBamAllRetainedToolsCompleteArgs,
) -> Result<()> {
    let repo_root = env::current_dir().context("resolve current directory")?;
    let report = render_bam_all_retained_tools_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_all_retained_tools_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamAllRetainedToolsCompleteReport> {
    owner::render_bam_all_retained_tools_complete(
        &super::bam_retained_proofs::context(repo_root)?,
        &output_path,
    )
}
#[cfg(all(test, feature = "bam_downstream"))]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        render_bam_all_retained_tools_complete, BamAllRetainedToolsCompleteReport,
        BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION,
        DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH,
    };
    use crate::commands::benchmark::repo_locking::acquire_cwd_lock;

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    fn benchmark_readiness_test_lock(root: &Path) -> bijux_dna_infra::FileLock {
        bijux_dna_infra::FileLock::try_acquire(
            &root.join("artifacts/test-locks/benchmark-readiness-mutators.lock"),
        )
        .expect("acquire benchmark readiness test lock")
    }

    #[test]
    fn render_bam_all_retained_tools_complete_reports_governed_pass_state() {
        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _lock = benchmark_readiness_test_lock(&root);
        let report = render_bam_all_retained_tools_complete(
            &root,
            PathBuf::from(DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH),
        )
        .expect("render BAM retained-tools completion report");

        assert_governed_pass_state(&report);
        assert!(
            report
                .checks
                .iter()
                .find(|check| check.goal_id == 439)
                .expect("goal 439 check")
                .detail
                .contains("bam.genotyping"),
            "goal 439 detail must keep BAM genotyping explicit"
        );
    }

    fn assert_governed_pass_state(report: &BamAllRetainedToolsCompleteReport) {
        assert_eq!(report.schema_version, BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH);
        assert_eq!(report.checked_goal_count, 19);
        assert_eq!(report.passed_goal_count, 19);
        assert_eq!(report.failed_goal_count, 0);
        assert!(report.failing_goal_ids.is_empty());
        assert!(report.retained_row_count > 0);
        assert_eq!(report.retained_stage_count, 24);
        assert!(report.retained_tool_count > 0);
        assert_eq!(report.command_adapter_row_count, report.retained_row_count);
        assert_eq!(report.output_declaration_row_count, report.retained_row_count);
        assert_eq!(report.expected_result_row_count, report.retained_row_count);
        assert_eq!(report.rendered_command_row_count, report.retained_row_count);
        assert_eq!(report.parser_fixture_row_count, report.retained_row_count);
        assert_eq!(report.local_smoke_row_count, report.retained_row_count);
        assert_eq!(
            report.local_smoke_host_stage_row_count
                + report.local_smoke_container_row_count
                + report.local_smoke_external_artifact_row_count,
            report.retained_row_count
        );
        assert_eq!(report.report_map_row_count, report.retained_row_count);
        assert_eq!(report.active_row_consistency_surface_count, 6);
        assert_eq!(report.micro_smoke_family_count, 12);
        assert_eq!(report.science_threshold_stage_count, 15);
        assert!(report.ok);
        assert_eq!(report.checks.len(), 19);
        assert!(report.checks.iter().all(|check| check.ok));
    }
}
