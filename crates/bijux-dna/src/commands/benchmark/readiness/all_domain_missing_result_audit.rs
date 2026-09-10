use super::{
    all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows,
    all_domain_output_declarations::collect_all_domain_output_declaration_rows,
    all_domain_rendered_commands::collect_all_domain_rendered_command_rows,
};
use crate::commands::benchmark::path_resolution::BenchmarkPathResolver;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
#[cfg(test)]
use bijux_dna_api::v1::api::bench::{
    AllDomainMissingResultStatus, ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION,
    BAM_REMOVED_RESULT_ID, FASTQ_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    AllDomainMissingResultTestReport, DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_render_all_domain_missing_result_audit(
    args: &parse::BenchReadinessRenderAllDomainMissingResultTestArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let report = render_all_domain_missing_result_audit(
        &repo_root,
        args.output.clone().unwrap_or_else(|| {
            benchmark_paths.benchmark_readiness_root().join("missing-result-test-all-domains.json")
        }),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_missing_result_audit(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainMissingResultTestReport> {
    let expected = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let commands = collect_all_domain_rendered_command_rows(repo_root)?;
    let outputs = collect_all_domain_output_declaration_rows(repo_root)?;
    bijux_dna_api::v1::api::bench::render_all_domain_missing_result_audit(
        repo_root,
        &output_path,
        &expected,
        &commands,
        &outputs,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::env;
    use std::path::PathBuf;

    use super::{
        render_all_domain_missing_result_audit, AllDomainMissingResultStatus,
        ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION, BAM_REMOVED_RESULT_ID,
        DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH, FASTQ_REMOVED_RESULT_ID,
        VCF_REMOVED_RESULT_ID,
    };
    use crate::commands::benchmark::repo_locking::{acquire_cwd_lock, RepoProcessLock};

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    struct CurrentDirGuard {
        previous: PathBuf,
    }

    impl CurrentDirGuard {
        fn change_to(path: &std::path::Path) -> Self {
            let previous = env::current_dir().expect("current dir");
            env::set_current_dir(path).expect("set current dir");
            Self { previous }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            env::set_current_dir(&self.previous).expect("restore current dir");
        }
    }

    #[test]
    fn all_domain_missing_result_audit_tracks_removed_rows_per_domain() {
        let _cwd_lock = acquire_cwd_lock();
        let root = repo_root();
        let _repo_lock =
            RepoProcessLock::acquire(&root, "benchmark-readiness-mutators").expect("repo lock");
        let _cwd_guard = CurrentDirGuard::change_to(&root);
        let report = render_all_domain_missing_result_audit(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH),
        )
        .expect("render all-domain missing-result test");

        assert_eq!(report.schema_version, ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH);
        assert_eq!(
            report.fake_result_root,
            "artifacts/benchmark-smoke/simulated-runs/readiness-probes/all-domains/missing-result-test"
        );
        assert_eq!(
            report.present_result_row_count + report.missing_result_row_count,
            report.expected_row_count
        );
        assert_eq!(report.missing_result_row_count, 3);
        assert!(report.passes_behavior_test);
        assert_eq!(report.domain_counts.len(), 3);
        assert_eq!(report.domain_counts.values().sum::<usize>(), report.expected_row_count);
        for domain in ["fastq", "bam", "vcf"] {
            assert!(report.domain_counts.get(domain).is_some_and(|count| *count > 0));
        }

        let removed_ids =
            report.removed_result_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
        assert_eq!(
            removed_ids,
            [FASTQ_REMOVED_RESULT_ID, BAM_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID]
                .into_iter()
                .collect()
        );

        let missing_rows = report
            .rows
            .iter()
            .filter(|row| row.result_status == AllDomainMissingResultStatus::MissingResult)
            .collect::<Vec<_>>();
        assert_eq!(missing_rows.len(), 3);
        assert!(missing_rows.iter().all(|row| row.observed_output_artifact_ids.is_empty()));
        assert!(report.rows.iter().all(|row| {
            row.result_status == AllDomainMissingResultStatus::MissingResult
                || !row.observed_output_artifact_ids.is_empty()
        }));
    }
}
