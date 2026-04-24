//! Owner: bijux-dna-bench
//! Dataset diversity and stratification checks for benchmark suites.

use crate::diagnostics::BenchError;
use crate::model::BenchmarkSuiteSpec;

pub(crate) fn validate_suite_diversity(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    if suite.datasets.is_empty() || suite.stages.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "suite must include datasets and stages".to_string(),
        ));
    }
    if suite.datasets.iter().any(|dataset| dataset.hash.trim().is_empty()) {
        return Err(BenchError::InvalidPolicy("suite datasets must include hash".to_string()));
    }
    if suite.datasets.len() < suite.diversity.min_dataset_count {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must include at least {} datasets",
            suite.diversity.min_dataset_count
        )));
    }
    let mut classes = std::collections::BTreeSet::new();
    let mut layouts = std::collections::BTreeSet::new();
    for dataset in &suite.datasets {
        classes.insert(dataset.class_label.as_str());
        layouts.insert(dataset.read_layout.as_str());
    }
    if classes.len() < suite.diversity.min_classes {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must include at least {} dataset classes",
            suite.diversity.min_classes
        )));
    }
    if layouts.len() < suite.diversity.min_read_layouts {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must include at least {} read layouts",
            suite.diversity.min_read_layouts
        )));
    }
    for requirement in &suite.stratifications {
        let values: std::collections::BTreeSet<&str> = match requirement.key.as_str() {
            "dataset_class" => {
                suite.datasets.iter().map(|dataset| dataset.class_label.as_str()).collect()
            }
            "read_layout" => {
                suite.datasets.iter().map(|dataset| dataset.read_layout.as_str()).collect()
            }
            _ => {
                return Err(BenchError::InvalidPolicy(format!(
                    "unsupported stratification key {}",
                    requirement.key
                )))
            }
        };
        for required in &requirement.required_values {
            if !values.contains(required.as_str()) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite missing required stratification value {} for {}",
                    required, requirement.key
                )));
            }
        }
    }
    Ok(())
}
