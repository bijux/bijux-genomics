//! Owner: bijux-dna-bench-model
//! Analysis requirement checks for benchmark suites.

use crate::diagnostics::BenchError;
use crate::model::BenchmarkSuiteSpec;

pub(crate) fn validate_suite_analysis_requirements(
    suite: &BenchmarkSuiteSpec,
) -> Result<(), BenchError> {
    if suite.analysis_requirements.require_bootstrap
        && suite.replicate_policy.count < suite.analysis_requirements.min_replicates_for_bootstrap
    {
        return Err(BenchError::InvalidPolicy(format!(
            "suite requires bootstrap with at least {} replicates",
            suite.analysis_requirements.min_replicates_for_bootstrap
        )));
    }
    if suite.analysis_requirements.require_outlier_detection && suite.replicate_policy.count < 3 {
        return Err(BenchError::InvalidPolicy(
            "suite requires outlier detection with at least 3 replicates".to_string(),
        ));
    }
    Ok(())
}
