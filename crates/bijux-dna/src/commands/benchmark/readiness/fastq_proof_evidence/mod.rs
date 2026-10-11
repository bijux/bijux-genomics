mod declarations;
mod schema;
mod validation;
use bijux_dna_bench_fastq::proofs::inputs::{ProofPaths, SchemaContext, ValidationContext};
use std::path::Path;
fn paths() -> ProofPaths<'static> {
    ProofPaths {
        expected_results:
            super::expected_benchmark_results::DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
        command_argv: super::rendered_command_argv::DEFAULT_RENDERED_COMMAND_ARGV_PATH,
        comparable_metrics: super::fastq_comparable_metrics::DEFAULT_FASTQ_COMPARABLE_METRICS_PATH,
    }
}
pub(super) fn schema(root: &Path) -> SchemaContext<'_> {
    SchemaContext {
        repo_root: root,
        declarations: &declarations::DeclarationAdapter,
        schema: &schema::SchemaAdapter,
        paths: paths(),
    }
}
pub(super) fn validation(root: &Path) -> ValidationContext<'_> {
    ValidationContext {
        repo_root: root,
        declarations: &declarations::DeclarationAdapter,
        validation: &validation::ValidationAdapter,
        paths: paths(),
    }
}
