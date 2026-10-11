use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::context::ReadinessContext;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_amplicon_fixture_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    let amplicon_report = match context.backend.validate_amplicon_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(context.paths.amplicon_fixture),
    ) {
        Ok(report) => Some(report),
        Err(err) => {
            for (goal_id, surface, output_path) in [
                (
                    71,
                    "corpus-03 amplicon fixture validates FASTQ sample and control ownership",
                    context.paths.amplicon_fixture.to_string(),
                ),
                (
                    72,
                    "corpus-03 primer metadata validates governed primer normalization inputs",
                    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/primers.tsv".to_string(),
                ),
                (
                    73,
                    "corpus-03 expected ASV table validates sample-aware amplicon truth rows",
                    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/expected_asvs.tsv".to_string(),
                ),
                (
                    74,
                    "corpus-03 chimera controls validate governed positive-control expectations",
                    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_expectations.tsv"
                        .to_string(),
                ),
            ] {
                checks.push(fail_check(
                    goal_id,
                    "corpora",
                    surface,
                    Some(output_path),
                    format!("{err:#}"),
                ));
            }
            None
        }
    };
    if let Some(report) = amplicon_report {
        assess_amplicon_fixture(checks, &report);
    }
}

fn assess_amplicon_fixture(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &crate::readiness::ports::AmpliconFixtureObservation,
) {
    if report.valid && report.sample_count >= 3 && report.control_count >= 1 {
        checks.push(ok_check(
            71,
            "corpora",
            "corpus-03 amplicon fixture validates FASTQ sample and control ownership",
            Some(report.manifest_path.clone()),
            format!(
                "amplicon fixture validates {} samples and {} controls",
                report.sample_count, report.control_count
            ),
        ));
    } else {
        checks.push(fail_check(
            71,
            "corpora",
            "corpus-03 amplicon fixture validates FASTQ sample and control ownership",
            Some(report.manifest_path.clone()),
            format!(
                "sample_count={} control_count={} valid={}",
                report.sample_count, report.control_count, report.valid
            ),
        ));
    }
    if report.primer_table_row_count >= 1 && !report.primers_tsv_path.trim().is_empty() {
        checks.push(ok_check(
            72,
            "corpora",
            "corpus-03 primer metadata validates governed primer normalization inputs",
            Some(report.primers_tsv_path.clone()),
            format!("primer metadata table contains {} rows", report.primer_table_row_count),
        ));
    } else {
        checks.push(fail_check(
            72,
            "corpora",
            "corpus-03 primer metadata validates governed primer normalization inputs",
            Some(report.primers_tsv_path.clone()),
            format!("primer_table_row_count={}", report.primer_table_row_count),
        ));
    }
    if report.expected_asv_row_count >= 1 && !report.expected_asvs_path.trim().is_empty() {
        checks.push(ok_check(
            73,
            "corpora",
            "corpus-03 expected ASV table validates sample-aware amplicon truth rows",
            Some(report.expected_asvs_path.clone()),
            format!("expected ASV table contains {} rows", report.expected_asv_row_count),
        ));
    } else {
        checks.push(fail_check(
            73,
            "corpora",
            "corpus-03 expected ASV table validates sample-aware amplicon truth rows",
            Some(report.expected_asvs_path.clone()),
            format!("expected_asv_row_count={}", report.expected_asv_row_count),
        ));
    }
    if report.chimera_expectation_row_count >= 1
        && !report.chimera_controls_fasta_path.trim().is_empty()
        && !report.chimera_expectations_path.trim().is_empty()
    {
        checks.push(ok_check(
            74,
            "corpora",
            "corpus-03 chimera controls validate governed positive-control expectations",
            Some(report.chimera_expectations_path.clone()),
            format!("chimera expectations contain {} rows", report.chimera_expectation_row_count),
        ));
    } else {
        checks.push(fail_check(
            74,
            "corpora",
            "corpus-03 chimera controls validate governed positive-control expectations",
            Some(report.chimera_expectations_path.clone()),
            format!("chimera_expectation_row_count={}", report.chimera_expectation_row_count),
        ));
    }
}
