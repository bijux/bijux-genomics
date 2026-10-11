use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::context::ReadinessContext;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_edna_fixture_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    let edna_report = match context.backend.validate_edna_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(context.paths.edna_fixture),
    ) {
        Ok(report) => Some(report),
        Err(err) => {
            checks.push(fail_check(
                68,
                "corpora",
                "corpus-02 eDNA fixture validates mock-community FASTQ inputs",
                Some(context.paths.edna_fixture.to_string()),
                format!("{err:#}"),
            ));
            checks.push(fail_check(
                70,
                "corpora",
                "corpus-02 expected taxonomy output validates sample-taxon truth rows",
                Some(
                    "benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/expected_taxa.tsv"
                        .to_string(),
                ),
                "corpus-02 eDNA fixture validation failed earlier".to_string(),
            ));
            None
        }
    };
    if let Some(report) = edna_report {
        if report.valid && report.sample_count == 2 {
            checks.push(ok_check(
                68,
                "corpora",
                "corpus-02 eDNA fixture validates mock-community FASTQ inputs",
                Some(report.manifest_path.clone()),
                format!("corpus-02 eDNA fixture validates {} samples", report.sample_count),
            ));
        } else {
            checks.push(fail_check(
                68,
                "corpora",
                "corpus-02 eDNA fixture validates mock-community FASTQ inputs",
                Some(report.manifest_path.clone()),
                format!(
                    "expected 2 samples and valid fixture but found sample_count={} valid={}",
                    report.sample_count, report.valid
                ),
            ));
        }
        if report.expected_taxa_count == 3
            && report.expected_taxa_output_row_count == 6
            && !report.expected_taxa_path.trim().is_empty()
        {
            checks.push(ok_check(
                70,
                "corpora",
                "corpus-02 expected taxonomy output validates sample-taxon truth rows",
                Some(report.expected_taxa_path.clone()),
                format!(
                    "expected taxonomy contract carries {} taxa across {} rows",
                    report.expected_taxa_count, report.expected_taxa_output_row_count
                ),
            ));
        } else {
            checks.push(fail_check(
                70,
                "corpora",
                "corpus-02 expected taxonomy output validates sample-taxon truth rows",
                Some(report.expected_taxa_path.clone()),
                format!(
                    "expected_taxa_count={} expected_taxa_output_row_count={}",
                    report.expected_taxa_count, report.expected_taxa_output_row_count
                ),
            ));
        }
    }
}
